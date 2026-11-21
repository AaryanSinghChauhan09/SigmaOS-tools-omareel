//! Integration tests for sigmaos-tools-omareel.

use sigmaos_tools_omareel::*;

fn tl() -> Timeline {
    let mut t = Timeline::new();
    t.add_clip(TrackKind::Screen, Clip::Video(VideoClip {
        id: 1, source: "s.mp4".into(), start: 0.0, duration: 20.0,
        in_point: 2.0, kind: ClipKind::Screen, volume: 1.0,
    })).unwrap();
    t
}

#[test]
fn test_timeline_duration() {
    assert_eq!(tl().duration(), 20.0);
}

#[test]
fn test_render_plan_basic() {
    let plan = plan_render(&tl(), &RenderSettings::default(), "out.mp4").unwrap();
    assert!(plan.filter_graph.contains("trim"));
    assert!(plan.filter_graph.contains("amix") == false); // single audio
}

#[test]
fn test_zoom_interpolation() {
    let kfs = vec![
        ZoomKeyframe { time: 0.0, region: ZoomRegion { cx: 0.5, cy: 0.5, zoom: 1.0 }, transition: 0.0 },
        ZoomKeyframe { time: 10.0, region: ZoomRegion { cx: 0.3, cy: 0.4, zoom: 2.5 }, transition: 0.0 },
    ];
    let z = zoom_at(&kfs, 5.0);
    assert!((z.zoom - 1.75).abs() < 0.01);
}

#[test]
fn test_camera_bubble() {
    let b = CameraBubble::default();
    let (x, y) = b.origin(1920, 1080);
    assert!(x > 1500 && y > 800);
}

#[test]
fn test_cursor_activity() {
    let mut c = SyntheticCursor::default();
    c.mode = CursorMode::OnActivity;
    c.move_to(0.5, 0.5, 1.0);
    assert!(c.visible_at(1.2));
    assert!(!c.visible_at(3.0));
}
