//! Timeline preview — render the edit timeline as ASCII/JSON.
//!
//! Upstream omareel shows a multi-track timeline UI; this port renders
//! the same model to text: tracks as rows, clips as positioned blocks,
//! zoom keyframes as markers, and a playhead line.

use crate::clip::Clip;
use crate::timeline::{Timeline, TrackKind};

/// Preview options.
#[derive(Debug, Clone)]
pub struct PreviewOpts {
    /// Width in characters (time axis resolution).
    pub width: usize,
    /// Seconds per cell.
    pub secs_per_cell: f64,
}

impl Default for PreviewOpts {
    fn default() -> Self {
        Self { width: 80, secs_per_cell: 1.0 }
    }
}

/// Render the timeline as ASCII lines.
pub fn render_timeline(timeline: &Timeline, playhead: f64, opts: &PreviewOpts) -> Vec<String> {
    let duration = timeline.duration().max(opts.width as f64 * opts.secs_per_cell);
    let mut lines = Vec::new();
    // Time ruler.
    lines.push(ruler(duration, opts));
    for track in &timeline.tracks {
        lines.push(render_track(track, playhead, duration, opts));
    }
    // Playhead.
    let pos = (playhead / opts.secs_per_cell) as usize;
    if pos < opts.width {
        let mut l = String::new();
        for _ in 0..pos {
            l.push(' ');
        }
        l.push('▼');
        lines.push(l);
    }
    lines
}

fn ruler(duration: f64, opts: &PreviewOpts) -> String {
    let mut s = String::new();
    for i in 0..opts.width {
        let t = i as f64 * opts.secs_per_cell;
        if (t as u64) % 10 == 0 && (t as u64) % opts.secs_per_cell as u64 == 0 {
            s.push_str(&format!("{}", (t as u64) % 10));
        } else if t < duration {
            s.push('─');
        } else {
            s.push(' ');
        }
    }
    s
}

fn render_track(track: &crate::timeline::Track, playhead: f64, duration: f64, opts: &PreviewOpts) -> String {
    let label = match track.kind {
        TrackKind::Screen => "screen",
        TrackKind::Camera => "camera",
        TrackKind::Audio => "audio ",
        TrackKind::Zoom => "zoom  ",
    };
    let mut cells = vec![' '; opts.width];
    for clip in &track.clips {
        let start_cell = (clip.start() / opts.secs_per_cell) as usize;
        let end_cell = ((clip.end() / opts.secs_per_cell).ceil() as usize).min(opts.width);
        let glyph = match clip {
            Clip::Video(v) => match v.kind {
                crate::clip::ClipKind::Screen => '█',
                crate::clip::ClipKind::Camera => '▓',
                crate::clip::ClipKind::Audio => '░',
            },
            Clip::Audio(_) => '▒',
        };
        for c in cells.iter_mut().take(end_cell).skip(start_cell) {
            *c = glyph;
        }
    }
    // Playhead marker.
    let pos = (playhead / opts.secs_per_cell) as usize;
    if pos < opts.width && !cells.is_empty() {
        cells[pos] = '│';
    }
    format!("{}│{}", label, cells.iter().collect::<String>())
}

/// Timeline summary as JSON-ish text.
pub fn summary_json(timeline: &Timeline) -> String {
    let duration = timeline.duration();
    let mut out = format!("{{\"duration\":{:.1},\"tracks\":[", duration);
    for (i, t) in timeline.tracks.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let kind = match t.kind {
            TrackKind::Screen => "screen",
            TrackKind::Camera => "camera",
            TrackKind::Audio => "audio",
            TrackKind::Zoom => "zoom",
        };
        out.push_str(&format!("{{\"kind\":\"{}\",\"clips\":{}}}", kind, t.clips.len()));
    }
    out.push_str("]}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::{Clip, ClipKind, VideoClip};

    fn tl() -> Timeline {
        let mut t = Timeline::new();
        t.add_clip(TrackKind::Screen, Clip::Video(VideoClip {
            id: 1, source: "s.mp4".into(), start: 0.0, duration: 10.0,
            in_point: 0.0, kind: ClipKind::Screen, volume: 1.0,
        })).unwrap();
        t
    }

    #[test]
    fn test_render() {
        let lines = render_timeline(&tl(), 5.0, &PreviewOpts::default());
        assert!(lines.iter().any(|l| l.contains("screen")));
        assert!(lines.iter().any(|l| l.contains('█')));
        assert!(lines.iter().any(|l| l.contains('│')));
    }

    #[test]
    fn test_summary() {
        let s = summary_json(&tl());
        assert!(s.contains("\"duration\":10.0"));
        assert!(s.contains("screen"));
    }
}
