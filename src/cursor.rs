//! Synthetic cursor — draw a mouse cursor into the recording.
//!
//! Upstream omareel can inject a synthetic cursor so viewers see where
//! the user clicked/hovered. This models cursor state and its overlay
//! placement per frame.

/// Cursor visual style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    /// Arrow pointer.
    Arrow,
    /// Hand (clickable).
    Hand,
    /// I-beam (text).
    IBeam,
    /// Hidden.
    Hidden,
}

/// Cursor visibility mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMode {
    /// Always show cursor.
    Always,
    /// Show only during activity (clicks/scrolls).
    OnActivity,
    /// Never show.
    Never,
}

/// Cursor state at a point in time.
#[derive(Debug, Clone)]
pub struct SyntheticCursor {
    /// Current position in normalized frame coords (0-1).
    pub x: f64,
    pub y: f64,
    /// Current style.
    pub style: CursorStyle,
    /// Mode.
    pub mode: CursorMode,
    /// Last activity timestamp (for OnActivity fade).
    pub last_activity: f64,
}

impl Default for SyntheticCursor {
    fn default() -> Self {
        Self {
            x: 0.5,
            y: 0.5,
            style: CursorStyle::Arrow,
            mode: CursorMode::Always,
            last_activity: 0.0,
        }
    }
}

impl SyntheticCursor {
    /// Whether cursor should be drawn at time `t`.
    pub fn visible_at(&self, t: f64) -> bool {
        match self.mode {
            CursorMode::Always => true,
            CursorMode::Never => false,
            CursorMode::OnActivity => t - self.last_activity < 1.5,
        }
    }

    /// Update cursor position; marks activity.
    pub fn move_to(&mut self, x: f64, y: f64, t: f64) {
        self.x = x.clamp(0.0, 1.0);
        self.y = y.clamp(0.0, 1.0);
        self.last_activity = t;
    }

    /// Pixel position for a frame size.
    pub fn pixel_pos(&self, w: u32, h: u32) -> (u32, u32) {
        ((self.x * w as f64) as u32, (self.y * h as f64) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_visibility() {
        let mut c = SyntheticCursor::default();
        c.mode = CursorMode::OnActivity;
        c.move_to(0.2, 0.3, 10.0);
        assert!(c.visible_at(10.5));
        assert!(!c.visible_at(12.0)); // >1.5s later
    }

    #[test]
    fn test_pixel_pos() {
        let mut c = SyntheticCursor::default();
        c.move_to(0.5, 0.25, 0.0);
        assert_eq!(c.pixel_pos(100, 200), (50, 50));
    }
}
