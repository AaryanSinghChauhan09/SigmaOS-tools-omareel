//! omareel — screen recording editor CLI (render-plan front-end).

use sigmaos_tools_omareel::{Clip, ClipKind, RenderSettings, Timeline, TrackKind, VideoClip, plan_render};

fn main() {
    let mut tl = Timeline::new();
    tl.add_clip(TrackKind::Screen, Clip::Video(VideoClip {
        id: 1, source: "screen.mp4".into(), start: 0.0, duration: 30.0,
        in_point: 0.0, kind: ClipKind::Screen, volume: 1.0,
    })).unwrap();
    match plan_render(&tl, &RenderSettings::default(), "out.mp4") {
        Ok(p) => {
            println!("omareel: {}", p.summary);
            println!("inputs: {:?}", p.inputs);
            println!("filter: {}", p.filter_graph);
        }
        Err(e) => { eprintln!("omareel: {e}"); std::process::exit(1); }
    }
}
