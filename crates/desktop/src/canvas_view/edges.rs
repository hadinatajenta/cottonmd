use cottonmd_canvas::{CanvasEdge, EdgeSide};
use eframe::egui::{
    self, epaint::CubicBezierShape, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke,
    Vec2,
};

use crate::theme::Palette;

const CONTROL_REACH_RATIO: f32 = 0.4;
const END_DOT_RADIUS: f32 = 3.5;
const LABEL_PADDING: Vec2 = Vec2::new(10.0, 4.0);

pub struct EdgeGeometry {
    pub start: Pos2,
    pub start_control: Pos2,
    pub end_control: Pos2,
    pub end: Pos2,
}

impl EdgeGeometry {
    pub fn between(from_rect: Rect, to_rect: Rect, edge: &CanvasEdge) -> Self {
        let start = side_anchor(from_rect, edge.from_side);
        let end = side_anchor(to_rect, edge.to_side);
        let reach = start.distance(end) * CONTROL_REACH_RATIO;
        Self {
            start,
            start_control: start + side_normal(edge.from_side) * reach,
            end_control: end + side_normal(edge.to_side) * reach,
            end,
        }
    }

    pub fn midpoint(&self) -> Pos2 {
        let sum = self.start.to_vec2()
            + self.start_control.to_vec2() * 3.0
            + self.end_control.to_vec2() * 3.0
            + self.end.to_vec2();
        (sum / 8.0).to_pos2()
    }

    pub fn bounds(&self) -> Rect {
        Rect::from_points(&[self.start, self.start_control, self.end_control, self.end])
    }

    pub fn is_finite(&self) -> bool {
        [self.start, self.start_control, self.end_control, self.end]
            .iter()
            .all(|point| point.x.is_finite() && point.y.is_finite())
    }
}

pub fn draw_edge(
    painter: &Painter,
    pal: Palette,
    edge: &CanvasEdge,
    geometry: &EdgeGeometry,
    is_highlighted: bool,
) {
    let color = if is_highlighted {
        pal.accent
    } else {
        pal.border_strong
    };
    let stroke = Stroke::new(if is_highlighted { 2.0 } else { 1.5 }, color);
    painter.add(CubicBezierShape::from_points_stroke(
        [
            geometry.start,
            geometry.start_control,
            geometry.end_control,
            geometry.end,
        ],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));
    painter.circle_filled(geometry.end, END_DOT_RADIUS, color);
    if let Some(label) = edge.label.as_deref().filter(|text| !text.is_empty()) {
        draw_edge_label(painter, pal, geometry.midpoint(), label);
    }
}

fn draw_edge_label(painter: &Painter, pal: Palette, center: Pos2, label: &str) {
    let galley =
        painter.layout_no_wrap(label.to_owned(), FontId::proportional(11.0), pal.text_muted);
    let chip_rect = Rect::from_center_size(center, galley.size() + LABEL_PADDING);
    painter.rect(
        chip_rect,
        CornerRadius::same(crate::theme::RADIUS_BADGE),
        pal.bg_editor,
        Stroke::new(1.0, pal.border),
        egui::StrokeKind::Inside,
    );
    painter.galley(
        chip_rect.center() - galley.size() / 2.0,
        galley,
        pal.text_muted,
    );
}

fn side_anchor(rect: Rect, side: Option<EdgeSide>) -> Pos2 {
    match side {
        Some(EdgeSide::Top) => rect.center_top(),
        Some(EdgeSide::Right) => rect.right_center(),
        Some(EdgeSide::Bottom) => rect.center_bottom(),
        Some(EdgeSide::Left) => rect.left_center(),
        None => rect.center(),
    }
}

fn side_normal(side: Option<EdgeSide>) -> Vec2 {
    match side {
        Some(EdgeSide::Top) => Vec2::new(0.0, -1.0),
        Some(EdgeSide::Right) => Vec2::new(1.0, 0.0),
        Some(EdgeSide::Bottom) => Vec2::new(0.0, 1.0),
        Some(EdgeSide::Left) => Vec2::new(-1.0, 0.0),
        None => Vec2::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn edge_between(from_side: Option<EdgeSide>, to_side: Option<EdgeSide>) -> CanvasEdge {
        CanvasEdge {
            id: "edge".into(),
            from_node: "a".into(),
            from_side,
            to_node: "b".into(),
            to_side,
            label: None,
        }
    }

    #[test]
    fn sideless_edge_is_a_straight_line_with_midpoint_between_centers() {
        let from_rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        let to_rect = Rect::from_min_size(pos2(300.0, 0.0), vec2(100.0, 100.0));
        let geometry = EdgeGeometry::between(from_rect, to_rect, &edge_between(None, None));
        assert_eq!(geometry.start_control, geometry.start);
        assert!((geometry.midpoint().x - 200.0).abs() < 1e-3);
        assert!((geometry.midpoint().y - 50.0).abs() < 1e-3);
    }

    #[test]
    fn side_anchors_bulge_outward_from_the_node() {
        let from_rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
        let to_rect = Rect::from_min_size(pos2(300.0, 0.0), vec2(100.0, 100.0));
        let edge = edge_between(Some(EdgeSide::Right), Some(EdgeSide::Left));
        let geometry = EdgeGeometry::between(from_rect, to_rect, &edge);
        assert!(geometry.start_control.x > geometry.start.x);
        assert!(geometry.end_control.x < geometry.end.x);
    }

    #[test]
    fn non_finite_rect_produces_non_finite_geometry() {
        let broken = Rect::from_min_size(pos2(f32::NAN, 0.0), vec2(10.0, 10.0));
        let fine = Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0));
        let geometry = EdgeGeometry::between(broken, fine, &edge_between(None, None));
        assert!(!geometry.is_finite());
    }
}
