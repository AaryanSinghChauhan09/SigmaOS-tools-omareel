//! SigmaOS tools-omareel component.
//!
//! Rust implementation of `omacom/omareel`: screen recordings that look
//! great — a screen recorder/editor with synthetic cursor, auto-zooms,
//! and camera bubble (upstream: Rust + C++/Qt).
//!
//! This port implements the editing model upstream's `omareel_core`
//! crates provide: clip timelines (video/audio/camera tracks), zoom
//! keyframes, and the render plan (ffmpeg filter graph) — the logic the
//! editor UI manipulates.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod camera;
pub mod clip;
pub mod cursor;
pub mod error;
pub mod render;
pub mod timeline;
pub mod zoom;

pub use camera::{CameraBubble, CameraPosition};
pub use clip::{AudioClip, Clip, ClipKind, VideoClip};
pub use cursor::{CursorMode, CursorStyle, SyntheticCursor};
pub use error::OmareelError;
pub use render::{FfmpegRenderPlan, RenderSettings, plan_render, zoom_at_time};
pub use timeline::{Timeline, Track, TrackKind};
pub use zoom::{ZoomKeyframe, ZoomRegion, auto_zoom, zoom_at};
