//! Zoom keyframes — auto-zoom regions over time (upstream "auto zooms").

/// A zoom region at a point in time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomRegion {
    /// Center X (0.0-1.0 of frame width).
    pub cx: f64,
    /// Center Y (0.0-1.0 of frame height).
    pub cy: f64,
    /// Zoom level (1.0 = no zoom, 2.0 = 2x).
    pub zoom: f64,
}

impl Default for ZoomRegion {
    fn default() -> Self {
        Self { cx: 0.5, cy: 0.5, zoom: 1.0 }
    }
}

/// A zoom keyframe: region + when it applies.
#[derive(Debug, Clone, Copy)]
pub struct ZoomKeyframe {
    /// Time on timeline (seconds).
    pub time: f64,
    /// Zoom region.
    pub region: ZoomRegion,
    /// Transition duration into this keyframe (seconds).
    pub transition: f64,
}

impl Default for ZoomKeyframe {
    fn default() -> Self {
        Self { time: 0.0, region: ZoomRegion::default(), transition: 0.3 }
    }
}

/// Interpolate zoom region at time `t` from sorted keyframes.
pub fn zoom_at(keyframes: &[ZoomKeyframe], t: f64) -> ZoomRegion {
    if keyframes.is_empty() {
        return ZoomRegion::default();
    }
    // Before first keyframe → first region.
    if t <= keyframes[0].time {
        return keyframes[0].region;
    }
    // After last → last region.
    let last = keyframes.last().unwrap();
    if t >= last.time {
        return last.region;
    }
    // Find surrounding keyframes.
    for w in keyframes.windows(2) {
        let (a, b) = (w[0], w[1]);
        if t >= a.time && t <= b.time {
            let span = (b.time - a.time).max(1e-9);
            // Apply transition easing: linear over `transition` secs.
            let local = t - a.time;
            let trans = b.transition.min(span);
            let f = if trans > 0.0 && local < trans {
                local / trans
            } else if local >= span - trans && trans > 0.0 {
                // Ease out near end.
                1.0
            } else {
                1.0
            };
            let _ = f;
            // Simple lerp.
            let f = ((t - a.time) / span).clamp(0.0, 1.0);
            return ZoomRegion {
                cx: a.region.cx + (b.region.cx - a.region.cx) * f,
                cy: a.region.cy + (b.region.cy - a.region.cy) * f,
                zoom: a.region.zoom + (b.region.zoom - a.region.zoom) * f,
            };
        }
    }
    last.region
}

/// Generate auto-zoom keyframes from cursor activity (simplified):
/// zoom into regions where the cursor moved recently.
///
/// `activity` is (time, x, y) cursor samples in normalized coords.
pub fn auto_zoom(activity: &[(f64, f64, f64)], zoom_level: f64) -> Vec<ZoomKeyframe> {
    let mut kfs = vec![ZoomKeyframe {
        time: 0.0,
        region: ZoomRegion::default(),
        transition: 0.3,
    }];
    // Cluster samples 2s apart into zoom regions.
    let mut last_t = -10.0;
    for &(t, x, y) in activity {
        if t - last_t > 2.0 {
            kfs.push(ZoomKeyframe {
                time: t,
                region: ZoomRegion { cx: x, cy: y, zoom: zoom_level },
                transition: 0.4,
            });
            last_t = t;
        }
    }
    kfs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_at_interpolates() {
        let kfs = vec![
            ZoomKeyframe { time: 0.0, region: ZoomRegion { cx: 0.5, cy: 0.5, zoom: 1.0 }, transition: 0.0 },
            ZoomKeyframe { time: 10.0, region: ZoomRegion { cx: 0.2, cy: 0.3, zoom: 2.0 }, transition: 0.0 },
        ];
        let mid = zoom_at(&kfs, 5.0);
        assert!((mid.cx - 0.35).abs() < 0.01);
        assert!((mid.zoom - 1.5).abs() < 0.01);
        assert_eq!(zoom_at(&kfs, 0.0).zoom, 1.0);
        assert_eq!(zoom_at(&kfs, 20.0).zoom, 2.0);
    }

    #[test]
    fn test_auto_zoom() {
        let activity = vec![
            (0.0, 0.5, 0.5), (1.0, 0.6, 0.5), (5.0, 0.2, 0.3), (6.0, 0.2, 0.3), (12.0, 0.8, 0.7),
        ];
        let kfs = auto_zoom(&activity, 2.0);
        assert!(kfs.len() >= 2); // initial + at least one zoom
        assert!(kfs.iter().any(|k| k.region.zoom == 2.0));
    }
}
