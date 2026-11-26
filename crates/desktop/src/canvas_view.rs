use eframe::egui::{self, Align2, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use cottonmd_canvas::{CanvasDocument, CanvasNode, CanvasNodeKind, EdgeSide};

use crate::theme::Palette;

pub struct CanvasViewState {
    pub pan: Vec2,
    pub zoom: f32,
}

impl Default for CanvasViewState {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 0.85,
        }
    }
}

impl CanvasViewState {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        doc: Option<&CanvasDocument>,
        pal: Palette,
    ) -> Option<String> {
        let mut open_file = None;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Canvas").strong().color(pal.text_primary));
            ui.separator();
            if ui.button("Reset view").clicked() {
                self.pan = Vec2::ZERO;
                self.zoom = 0.85;
            }
            if let Some(d) = doc {
                ui.label(
                    egui::RichText::new(format!(
                        "{} nodes  |  {} edges",
                        d.nodes.len(),
                        d.edges.len()
                    ))
                    .size(11.5)
                    .color(pal.text_muted),
                );
            }
        });
        ui.separator();

        let Some(doc) = doc else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Select a .canvas file from the Files sidebar to render it here.")
                        .color(pal.text_muted),
                );
            });
            return None;
        };

        let avail = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(avail, Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, pal.bg_editor);

        if response.dragged() {
            self.pan += response.drag_delta();
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.01 {
                let factor = (1.0 + scroll * 0.002).clamp(0.8, 1.25);
                self.zoom = (self.zoom * factor).clamp(0.2, 3.0);
            }
        }

        let origin = rect.center() + self.pan;
        let to_screen_rect = |n: &CanvasNode| -> Rect {
            let min = origin + Vec2::new(n.bounds.x as f32, n.bounds.y as f32) * self.zoom;
            let size = Vec2::new(n.bounds.width as f32, n.bounds.height as f32) * self.zoom;
            Rect::from_min_size(min, size)
        };

        for edge in &doc.edges {
            if let (Some(from), Some(to)) = (doc.node(&edge.from_node), doc.node(&edge.to_node)) {
                let p1 = side_anchor(to_screen_rect(from), edge.from_side);
                let p2 = side_anchor(to_screen_rect(to), edge.to_side);
                painter.line_segment([p1, p2], Stroke::new(1.5, pal.accent));
                if let Some(ref lbl) = edge.label {
                    let mid = Pos2::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
                    painter.text(
                        mid,
                        Align2::CENTER_CENTER,
                        lbl,
                        FontId::proportional(11.0),
                        pal.text_muted,
                    );
                }
            }
        }

        let hover_pos = response.hover_pos();
        for node in &doc.nodes {
            let nrect = to_screen_rect(node);
            if !rect.intersects(nrect) {
                continue;
            }
            let is_group = matches!(node.kind, CanvasNodeKind::Group { .. });
            let fill = if is_group { pal.bg_sidebar } else { pal.bg_elevated };
            let hovered = hover_pos.is_some_and(|p| nrect.contains(p));
            let border = if hovered { pal.accent } else { pal.border };

            painter.rect_filled(nrect, CornerRadius::same(6), fill);
            painter.rect_stroke(
                nrect,
                CornerRadius::same(6),
                Stroke::new(1.0, border),
                egui::StrokeKind::Inside,
            );

            let (title, linked_file) = match &node.kind {
                CanvasNodeKind::Text { text } => (text.clone(), None),
                CanvasNodeKind::File { file, .. } => (format!("File: {file}"), Some(file.clone())),
                CanvasNodeKind::Link { url } => (url.clone(), None),
                CanvasNodeKind::Group { label } => {
                    (label.clone().unwrap_or_else(|| "Group".into()), None)
                }
            };

            painter.text(
                nrect.min + Vec2::new(10.0, 10.0),
                Align2::LEFT_TOP,
                title,
                FontId::proportional((13.0 * self.zoom).clamp(9.0, 16.0)),
                if linked_file.is_some() {
                    pal.accent
                } else {
                    pal.text_primary
                },
            );

            if hovered && linked_file.is_some() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                if response.clicked() {
                    open_file = linked_file;
                }
            }
        }

        open_file
    }
}

fn side_anchor(r: Rect, side: Option<EdgeSide>) -> Pos2 {
    match side {
        Some(EdgeSide::Top) => r.center_top(),
        Some(EdgeSide::Right) => r.right_center(),
        Some(EdgeSide::Bottom) => r.center_bottom(),
        Some(EdgeSide::Left) => r.left_center(),
        None => r.center(),
    }
}
