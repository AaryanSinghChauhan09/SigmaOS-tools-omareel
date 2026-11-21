//! Camera bubble — webcam overlay position/size.

/// Position of the camera bubble on the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPosition {
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left.
    BottomLeft,
    /// Bottom-right (default for Omarchy).
    BottomRight,
}

/// Camera bubble settings.
#[derive(Debug, Clone)]
pub struct CameraBubble {
    /// Whether the bubble is visible.
    pub visible: bool,
    /// Corner position.
    pub position: CameraPosition,
    /// Bubble size as fraction of frame height (0.1-0.3 typical).
    pub size_frac: f64,
    /// Corner margin in pixels.
    pub margin: u32,
    /// Whether bubble has rounded corners/shadow.
    pub rounded: bool,
}

impl Default for CameraBubble {
    fn default() -> Self {
        Self {
            visible: true,
            position: CameraPosition::BottomRight,
            size_frac: 0.18,
            margin: 24,
            rounded: true,
        }
    }
}

impl CameraBubble {
    /// Bubble width/height in pixels for a frame size.
    pub fn pixel_size(&self, frame_w: u32, frame_h: u32) -> (u32, u32) {
        let side = (frame_h as f64 * self.size_frac) as u32;
        // 4:3 aspect typical webcam.
        (side * 4 / 3, side)
    }

    /// Top-left corner of bubble in frame pixels.
    pub fn origin(&self, frame_w: u32, frame_h: u32) -> (u32, u32) {
        let (bw, bh) = self.pixel_size(frame_w, frame_h);
        let m = self.margin;
        match self.position {
            CameraPosition::TopLeft => (m, m),
            CameraPosition::TopRight => (frame_w.saturating_sub(bw + m), m),
            CameraPosition::BottomLeft => (m, frame_h.saturating_sub(bh + m)),
            CameraPosition::BottomRight => {
                (frame_w.saturating_sub(bw + m), frame_h.saturating_sub(bh + m))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bubble_position() {
        let b = CameraBubble::default();
        let (x, y) = b.origin(1920, 1080);
        // Bottom-right: x near right edge, y near bottom.
        assert!(x > 1400);
        assert!(y > 800);
    }

    #[test]
    fn test_size() {
        let b = CameraBubble::default();
        let (w, h) = b.pixel_size(1920, 1080);
        assert_eq!(h, (1080.0 * 0.18) as u32);
        assert_eq!(w, h * 4 / 3);
    }
}
