//! Timeline model — tracks of clips + total duration.

use crate::clip::Clip;
use crate::OmareelError;

/// Track kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    /// Screen video track.
    Screen,
    /// Camera overlay track.
    Camera,
    /// Audio track.
    Audio,
    /// Zoom keyframes (not a media track — annotation layer).
    Zoom,
}

/// A timeline track.
#[derive(Debug, Clone)]
pub struct Track {
    /// Track kind.
    pub kind: TrackKind,
    /// Clips on this track (sorted by start).
    pub clips: Vec<Clip>,
}

/// The editing timeline.
#[derive(Debug, Clone, Default)]
pub struct Timeline {
    /// Tracks in layer order (bottom → top).
    pub tracks: Vec<Track>,
}

impl Timeline {
    /// New empty timeline with default tracks.
    pub fn new() -> Self {
        Self {
            tracks: vec![
                Track { kind: TrackKind::Screen, clips: Vec::new() },
                Track { kind: TrackKind::Camera, clips: Vec::new() },
                Track { kind: TrackKind::Audio, clips: Vec::new() },
            ],
        }
    }

    /// Add a clip to a track kind.
    pub fn add_clip(&mut self, kind: TrackKind, clip: Clip) -> Result<(), OmareelError> {
        let track = self
            .tracks
            .iter_mut()
            .find(|t| t.kind == kind)
            .ok_or_else(|| OmareelError::Timeline(format!("no {kind:?} track")))?;
        // Check overlap within track.
        for c in &track.clips {
            if clip.start() < c.end() && c.start() < clip.end() {
                return Err(OmareelError::Timeline(format!(
                    "clip overlaps existing clip on {:?} track",
                    kind
                )));
            }
        }
        track.clips.push(clip);
        track.clips.sort_by(|a, b| a.start().partial_cmp(&b.start()).unwrap());
        Ok(())
    }

    /// Total timeline duration (max clip end across tracks).
    pub fn duration(&self) -> f64 {
        self.tracks
            .iter()
            .flat_map(|t| t.clips.iter().map(|c| c.end()))
            .fold(0.0, f64::max)
    }

    /// Get track by kind.
    pub fn track(&self, kind: TrackKind) -> Option<&Track> {
        self.tracks.iter().find(|t| t.kind == kind)
    }

}

impl Default for TrackKind {
    fn default() -> Self {
        TrackKind::Screen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::{ClipKind, VideoClip};

    #[test]
    fn test_add_clip_and_duration() {
        let mut tl = Timeline::new();
        tl.add_clip(
            TrackKind::Screen,
            Clip::Video(VideoClip {
                id: 1, source: "a.mp4".into(), start: 0.0, duration: 10.0,
                in_point: 0.0, kind: ClipKind::Screen, volume: 1.0,
            }),
        )
        .unwrap();
        assert_eq!(tl.duration(), 10.0);
    }

    #[test]
    fn test_overlap_rejected() {
        let mut tl = Timeline::new();
        for (id, start) in [(1, 0.0), (2, 5.0)] {
            tl.add_clip(
                TrackKind::Screen,
                Clip::Video(VideoClip {
                    id, source: "x.mp4".into(), start, duration: 10.0,
                    in_point: 0.0, kind: ClipKind::Screen, volume: 1.0,
                }),
            )
            .unwrap_or(());
        }
        // Second clip at 5.0 overlaps first (0-10); should have failed.
        let screen = tl.track(TrackKind::Screen).unwrap();
        assert_eq!(screen.clips.len(), 1);
    }
}
