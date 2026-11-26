use eframe::egui::{Pos2, Rect, Vec2};

const SCROLL_ZOOM_SENSITIVITY: f32 = 0.002;
const SCROLL_ZOOM_STEP_MIN: f32 = 0.8;
const SCROLL_ZOOM_STEP_MAX: f32 = 1.25;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub origin: Pos2,
    pub zoom: f32,
}

impl Viewport {
    pub fn new(rect: Rect, pan: Vec2, zoom: f32) -> Self {
        Self {
            origin: rect.center() + pan,
            zoom,
        }
    }

    pub fn to_screen(self, world: Vec2) -> Pos2 {
        self.origin + world * self.zoom
    }

    pub fn to_world(self, screen: Pos2) -> Vec2 {
        (screen - self.origin) / self.zoom
    }

    pub fn grid_offset(&self, rect: Rect) -> Vec2 {
        self.origin - rect.min
    }
}

pub fn sanitize_zoom(zoom: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if zoom.is_finite() {
        zoom.clamp(min, max)
    } else {
        fallback.clamp(min, max)
    }
}

pub fn sanitize_pan(pan: Vec2) -> Vec2 {
    if pan.is_finite() {
        pan
    } else {
        Vec2::ZERO
    }
}

pub fn zoom_after_scroll(current: f32, scroll: f32, min: f32, max: f32) -> f32 {
    let safe_current = sanitize_zoom(current, min, max, 1.0);
    if !scroll.is_finite() {
        return safe_current;
    }
    let factor =
        (1.0 + scroll * SCROLL_ZOOM_SENSITIVITY).clamp(SCROLL_ZOOM_STEP_MIN, SCROLL_ZOOM_STEP_MAX);
    (safe_current * factor).clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{pos2, vec2};

    #[test]
    fn world_and_screen_coordinates_round_trip() {
        let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(400.0, 300.0));
        let viewport = Viewport::new(rect, vec2(15.0, -8.0), 1.7);
        let world = vec2(42.0, -17.5);
        let restored = viewport.to_world(viewport.to_screen(world));
        assert!((restored - world).length() < 1e-3);
    }

    #[test]
    fn non_finite_zoom_falls_back_to_clamped_default() {
        assert_eq!(sanitize_zoom(f32::NAN, 0.2, 3.0, 0.85), 0.85);
        assert_eq!(sanitize_zoom(f32::INFINITY, 0.2, 3.0, 0.85), 0.85);
        assert_eq!(sanitize_zoom(9.0, 0.2, 3.0, 0.85), 3.0);
    }

    #[test]
    fn scroll_zoom_is_bounded_and_survives_non_finite_input() {
        assert_eq!(zoom_after_scroll(1.0, f32::NAN, 0.2, 3.0), 1.0);
        assert_eq!(zoom_after_scroll(2.9, 1000.0, 0.2, 3.0), 3.0);
        assert_eq!(zoom_after_scroll(0.21, -1000.0, 0.2, 3.0), 0.2);
        assert!(zoom_after_scroll(f32::NAN, 10.0, 0.2, 3.0) > 1.0);
    }

    #[test]
    fn non_finite_pan_resets_to_zero() {
        assert_eq!(sanitize_pan(vec2(f32::NAN, 1.0)), Vec2::ZERO);
        assert_eq!(sanitize_pan(vec2(3.0, 4.0)), vec2(3.0, 4.0));
    }
}
