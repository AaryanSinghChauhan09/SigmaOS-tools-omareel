#![forbid(unsafe_code)]
//! omareel - Screen recording engine for SigmaOS

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum VideoFormat {
    Mp4,
    Webm,
    Gif,
}

impl VideoFormat {
    pub fn extension(&self) -> &str {
        match self {
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Gif => "gif",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    pub output_path: PathBuf,
    pub format: VideoFormat,
    pub include_audio: bool,
    pub include_cursor: bool,
}

impl RecordingConfig {
    pub fn new(output: &str, format: VideoFormat) -> Result<Self, String> {
        if output.is_empty() {
            return Err("Output path cannot be empty".to_string());
        }
        Ok(Self {
            fps: 30,
            width: 1920,
            height: 1080,
            output_path: PathBuf::from(output),
            format,
            include_audio: false,
            include_cursor: true,
        })
    }

    pub fn with_fps(mut self, fps: u32) -> Result<Self, String> {
        if fps == 0 || fps > 120 {
            return Err(format!("FPS must be 1-120, got {}", fps));
        }
        self.fps = fps;
        Ok(self)
    }
}

#[derive(Debug, Clone)]
pub struct RecordingFrame {
    pub sequence: u64,
    pub timestamp_ms: u64,
    pub compressed_size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct RecordingStats {
    pub total_frames: u64,
    pub duration_ms: u64,
    pub output_path: PathBuf,
    pub format: VideoFormat,
    pub total_bytes: u64,
}

impl RecordingStats {
    pub fn average_fps(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        self.total_frames as f64 / (self.duration_ms as f64 / 1000.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Idle,
    Recording,
    Paused,
    Stopped,
}

pub struct RecordingSession {
    pub config: RecordingConfig,
    pub state: SessionState,
    frames: Vec<RecordingFrame>,
}

impl RecordingSession {
    pub fn new(config: RecordingConfig) -> Self {
        Self {
            config,
            state: SessionState::Idle,
            frames: Vec::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.state != SessionState::Idle {
            return Err(format!("Cannot start: session is {:?}", self.state));
        }
        self.state = SessionState::Recording;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        if self.state != SessionState::Recording {
            return Err("Cannot pause: not recording".to_string());
        }
        self.state = SessionState::Paused;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        if self.state != SessionState::Paused {
            return Err("Cannot resume: not paused".to_string());
        }
        self.state = SessionState::Recording;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<RecordingStats, String> {
        if self.state == SessionState::Idle || self.state == SessionState::Stopped {
            return Err(format!("Cannot stop: session is {:?}", self.state));
        }
        self.state = SessionState::Stopped;
        let duration_ms = self.frames.last().map(|f| f.timestamp_ms).unwrap_or(0);
        let total_bytes: u64 = self.frames.iter().map(|f| f.compressed_size_bytes).sum();
        Ok(RecordingStats {
            total_frames: self.frames.len() as u64,
            duration_ms,
            output_path: self.config.output_path.clone(),
            format: self.config.format.clone(),
            total_bytes,
        })
    }

    pub fn add_frame(&mut self, frame: RecordingFrame) -> Result<(), String> {
        if self.state != SessionState::Recording {
            return Err("Cannot add frame: not recording".to_string());
        }
        self.frames.push(frame);
        Ok(())
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}


#[cfg(test)]
mod oma_reel_tests {
    use super::*;

    #[test]
    fn test_invalid_fps_error_aaa() {
        // Arrange
        let config = RecordingConfig::new("/tmp/test.mp4", VideoFormat::Mp4).unwrap();
        // Act
        let err_low = config.clone().with_fps(0);
        let err_high = config.with_fps(241);
        // Assert
        assert!(err_low.is_err());
        assert!(err_high.is_err());
    }

    #[test]
    fn test_session_state_transitions_aaa() {
        // Arrange
        let config = RecordingConfig::new("/tmp/out.gif", VideoFormat::Gif).unwrap();
        let mut session = RecordingSession::new(config);
        assert_eq!(session.state, SessionState::Idle);
        // Act
        session.start().unwrap();
        assert_eq!(session.state, SessionState::Recording);
        session.pause().unwrap();
        assert_eq!(session.state, SessionState::Paused);
        session.resume().unwrap();
        assert_eq!(session.state, SessionState::Recording);
        let _ = session.stop().unwrap();
        // Assert
        assert_eq!(session.state, SessionState::Stopped);
    }

    #[test]
    fn test_video_format_extensions_aaa() {
        // Arrange & Act & Assert
        assert_eq!(VideoFormat::Mp4.extension(), "mp4");
        assert_eq!(VideoFormat::Gif.extension(), "gif");
        assert_eq!(VideoFormat::Webm.extension(), "webm");
    }
}
