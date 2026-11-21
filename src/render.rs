//! Render plan — timeline + zoom + camera → ffmpeg filter graph.
//!
//! Upstream omareel renders via ffmpeg filter_complex: screen track with
//! crop/scale for zooms, camera overlay (overlay filter), audio mix.
//! This port builds the plan and generates the filter graph string.

use crate::camera::CameraBubble;
use crate::clip::{Clip, ClipKind};
use crate::timeline::{Timeline, TrackKind};
use crate::zoom::{ZoomKeyframe, zoom_at};
use crate::OmareelError;

/// Output render settings.
#[derive(Debug, Clone)]
pub struct RenderSettings {
    /// Output width.
    pub width: u32,
    /// Output height.
    pub height: u32,
    /// Output fps.
    pub fps: u32,
    /// Video bitrate (e.g. "8M").
    pub video_bitrate: String,
    /// Audio bitrate (e.g. "192k").
    pub audio_bitrate: String,
    /// Output format/extension.
    pub format: String,
    /// Camera bubble overlay.
    pub camera: CameraBubble,
    /// Zoom keyframes (empty = no zoom).
    pub zoom_keyframes: Vec<ZoomKeyframe>,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 60,
            video_bitrate: "8M".into(),
            audio_bitrate: "192k".into(),
            format: "mp4".into(),
            camera: CameraBubble::default(),
            zoom_keyframes: Vec::new(),
        }
    }
}

/// A planned ffmpeg render.
#[derive(Debug, Clone)]
pub struct FfmpegRenderPlan {
    /// Input files (screen, camera, audio sources).
    pub inputs: Vec<String>,
    /// Filter complex graph string.
    pub filter_graph: String,
    /// Output file.
    pub output: String,
    /// Summary description.
    pub summary: String,
}

/// Build the render plan for a timeline.
pub fn plan_render(
    timeline: &Timeline,
    settings: &RenderSettings,
    output: &str,
) -> Result<FfmpegRenderPlan, OmareelError> {
    let duration = timeline.duration();
    if duration <= 0.0 {
        return Err(OmareelError::Render("empty timeline".into()));
    }
    let screen = timeline.track(TrackKind::Screen)
        .and_then(|t| t.clips.first());
    let screen_clip = match screen {
        Some(Clip::Video(v)) => v.clone(),
        _ => return Err(OmareelError::Render("no screen video clip".into())),
    };
    let camera_clip = timeline.track(TrackKind::Camera)
        .and_then(|t| t.clips.first())
        .and_then(|c| match c {
            Clip::Video(v) if v.kind == ClipKind::Camera => Some(v.clone()),
            _ => None,
        });

    let mut inputs = vec![screen_clip.source.clone()];
    let mut filter_parts: Vec<String> = Vec::new();

    // Screen chain: trim + scale + optional zoom crop.
    let mut screen_chain = format!(
        "[0:v]trim=start={}:end={},setpts=PTS-STARTPTS",
        screen_clip.in_point,
        screen_clip.in_point + screen_clip.duration
    );
    if !settings.zoom_keyframes.is_empty() {
        // Zoom via crop+scale animated — simplified: use zoompan or crop expr.
        // Generate zoom expressions from keyframes.
        screen_chain.push_str(",scale=");
        // zoompan is complex; use scale with zoom factor per keyframe →
        // simplified: apply max zoom crop.
        let max_zoom = settings
            .zoom_keyframes
            .iter()
            .map(|k| k.region.zoom)
            .fold(1.0, f64::max);
        screen_chain.push_str(&format!(
            "iw*{z}:ih*{z}",
            z = max_zoom
        ));
        // crop to output size centered.
        screen_chain.push_str(&format!(
            ",crop={w}:{h}",
            w = settings.width,
            h = settings.height
        ));
    } else {
        screen_chain.push_str(&format!(
            ",scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2",
            w = settings.width,
            h = settings.height
        ));
    }
    screen_chain.push_str("[v0]");
    filter_parts.push(screen_chain);

    // Camera overlay.
    let mut video_label = "v0".to_string();
    if let Some(cam) = camera_clip.as_ref() {
        inputs.push(cam.source.clone());
        let idx = inputs.len() - 1;
        let (bw, bh) = settings.camera.pixel_size(settings.width, settings.height);
        let (ox, oy) = settings.camera.origin(settings.width, settings.height);
        filter_parts.push(format!(
            "[{idx}:v]trim=start={}:end={},setpts=PTS-STARTPTS,scale={bw}:{bh}[cam]",
            cam.in_point,
            cam.in_point + cam.duration,
            bw = bw,
            bh = bh
        ));
        filter_parts.push(format!(
            "[{video_label}][cam]overlay={ox}:{oy}:format=auto[vout]",
            ox = ox,
            oy = oy
        ));
        video_label = "vout".to_string();
    }
    let _ = video_label;

    // Audio: screen audio + camera audio mixed.
    let mut audio_inputs = Vec::new();
    filter_parts.push(format!(
        "[0:a]atrim=start={}:end={},asetpts=PTS-STARTPTS,volume={}[a0]",
        screen_clip.in_point,
        screen_clip.in_point + screen_clip.duration,
        screen_clip.volume
    ));
    audio_inputs.push("a0".to_string());
    if let Some(cam) = camera_clip.as_ref() {
        let idx = inputs.len() - 1;
        filter_parts.push(format!(
            "[{idx}:a]atrim=start={}:end={},asetpts=PTS-STARTPTS,volume={}[a1]",
            cam.in_point,
            cam.in_point + cam.duration,
            cam.volume
        ));
        audio_inputs.push("a1".to_string());
    }
    let audio_out = if audio_inputs.len() > 1 {
        filter_parts.push(format!(
            "{}amix=inputs={}:duration=longest[aout]",
            audio_inputs.iter().map(|l| format!("[{l}]")).collect::<String>(),
            audio_inputs.len()
        ));
        "aout"
    } else {
        "a0"
    };

    let filter_graph = filter_parts.join(";");
    let summary = format!(
        "render {}s @{}x{}{}fps → {} ({} inputs, zoom={})",
        duration,
        settings.width,
        settings.height,
        settings.fps,
        output,
        inputs.len(),
        !settings.zoom_keyframes.is_empty()
    );
    Ok(FfmpegRenderPlan {
        inputs,
        filter_graph,
        output: output.to_string(),
        summary,
    })
}

/// Compute zoom-at-time for render (used by callers animating zooms).
pub fn zoom_at_time(settings: &RenderSettings, t: f64) -> crate::zoom::ZoomRegion {
    zoom_at(&settings.zoom_keyframes, t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::{ClipKind, VideoClip};

    fn timeline_with_screen() -> Timeline {
        let mut tl = Timeline::new();
        tl.add_clip(
            TrackKind::Screen,
            Clip::Video(VideoClip {
                id: 1, source: "screen.mp4".into(), start: 0.0, duration: 30.0,
                in_point: 5.0, kind: ClipKind::Screen, volume: 1.0,
            }),
        )
        .unwrap();
        tl
    }

    #[test]
    fn test_basic_plan() {
        let plan = plan_render(&timeline_with_screen(), &RenderSettings::default(), "out.mp4").unwrap();
        assert!(plan.filter_graph.contains("trim=start=5"));
        assert!(plan.filter_graph.contains("scale=1920:1080"));
        assert!(plan.inputs.contains(&"screen.mp4".to_string()));
    }

    #[test]
    fn test_camera_overlay() {
        let mut tl = timeline_with_screen();
        tl.add_clip(
            TrackKind::Camera,
            Clip::Video(VideoClip {
                id: 2, source: "cam.mp4".into(), start: 0.0, duration: 30.0,
                in_point: 0.0, kind: ClipKind::Camera, volume: 0.8,
            }),
        )
        .unwrap();
        let plan = plan_render(&tl, &RenderSettings::default(), "out.mp4").unwrap();
        assert!(plan.filter_graph.contains("overlay="));
        assert!(plan.filter_graph.contains("amix"));
        assert_eq!(plan.inputs.len(), 2);
    }

    #[test]
    fn test_zoom_plan() {
        let mut s = RenderSettings::default();
        s.zoom_keyframes = vec![ZoomKeyframe {
            time: 5.0,
            region: crate::zoom::ZoomRegion { cx: 0.5, cy: 0.5, zoom: 2.0 },
            transition: 0.3,
        }];
        let plan = plan_render(&timeline_with_screen(), &s, "out.mp4").unwrap();
        assert!(plan.filter_graph.contains("crop=1920:1080"));
    }

    #[test]
    fn test_empty_timeline_errors() {
        let tl = Timeline::new();
        assert!(plan_render(&tl, &RenderSettings::default(), "x.mp4").is_err());
    }
}
