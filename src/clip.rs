//! Clip model — video/audio segments on a timeline.

/// Kind of media clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipKind {
    /// Screen recording video.
    Screen,
    /// Webcam video.
    Camera,
    /// Audio track.
    Audio,
}

/// A video clip (screen or camera).
#[derive(Debug, Clone)]
pub struct VideoClip {
    /// Clip id.
    pub id: u64,
    /// Source file path.
    pub source: String,
    /// Start position on the timeline (seconds).
    pub start: f64,
    /// Duration (seconds).
    pub duration: f64,
    /// Source in-point (seconds into source file).
    pub in_point: f64,
    /// Clip kind (screen vs camera).
    pub kind: ClipKind,
    /// Volume 0.0-1.0 (camera clips can be muted).
    pub volume: f32,
}

/// An audio clip.
#[derive(Debug, Clone)]
pub struct AudioClip {
    /// Clip id.
    pub id: u64,
    /// Source file path.
    pub source: String,
    /// Start on timeline.
    pub start: f64,
    /// Duration.
    pub duration: f64,
    /// Volume.
    pub volume: f32,
}

/// Either clip type.
#[derive(Debug, Clone)]
pub enum Clip {
    /// Video clip.
    Video(VideoClip),
    /// Audio clip.
    Audio(AudioClip),
}

impl Clip {
    /// End time on timeline.
    pub fn end(&self) -> f64 {
        match self {
            Self::Video(v) => v.start + v.duration,
            Self::Audio(a) => a.start + a.duration,
        }
    }
    /// Start time on timeline.
    pub fn start(&self) -> f64 {
        match self {
            Self::Video(v) => v.start,
            Self::Audio(a) => a.start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_bounds() {
        let c = Clip::Video(VideoClip {
            id: 1, source: "s.mp4".into(), start: 5.0, duration: 10.0,
            in_point: 0.0, kind: ClipKind::Screen, volume: 1.0,
        });
        assert_eq!(c.start(), 5.0);
        assert_eq!(c.end(), 15.0);
    }
}
