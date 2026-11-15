use sigmaos_tools_omareel::{
    RecordingConfig, RecordingFrame, RecordingSession, RecordingStats, SessionState, VideoFormat,
};

#[test]
fn test_recording_config_creation_aaa() {
    // Arrange + Act
    let config = RecordingConfig::new("/tmp/test.mp4", VideoFormat::Mp4).unwrap();
    // Assert
    assert_eq!(config.fps, 30);
    assert_eq!(config.width, 1920);
    assert_eq!(config.format, VideoFormat::Mp4);
}

#[test]
fn test_recording_session_lifecycle_aaa() {
    // Arrange
    let config = RecordingConfig::new("/tmp/test.mp4", VideoFormat::Mp4).unwrap();
    let mut session = RecordingSession::new(config);
    // Act
    session.start().unwrap();
    session
        .add_frame(RecordingFrame {
            sequence: 0,
            timestamp_ms: 33,
            compressed_size_bytes: 1024,
        })
        .unwrap();
    session
        .add_frame(RecordingFrame {
            sequence: 1,
            timestamp_ms: 66,
            compressed_size_bytes: 1024,
        })
        .unwrap();
    let stats = session.stop().unwrap();
    // Assert
    assert_eq!(stats.total_frames, 2);
    assert_eq!(session.state, SessionState::Stopped);
}

#[test]
fn test_stats_average_fps_aaa() {
    // Arrange
    let stats = RecordingStats {
        total_frames: 60,
        duration_ms: 2000,
        output_path: std::path::PathBuf::from("/tmp/t.mp4"),
        format: VideoFormat::Mp4,
        total_bytes: 1_000_000,
    };
    // Act
    let fps = stats.average_fps();
    // Assert
    assert!((fps - 30.0).abs() < 0.1);
}
