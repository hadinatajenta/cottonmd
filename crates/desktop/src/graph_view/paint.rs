use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Rect, RichText, Stroke, Ui, UiBuilder, Vec2,
};

use super::simulation::{draw_radius, matches_filter, EdgeList, SimNode};
use crate::canvas_view::viewport::Viewport;
use crate::theme::{semibold, Palette};
use crate::ui_kit::{card_frame, pill_badge, section_label};

const CULL_MARGIN: f32 = 40.0;
const LABEL_MIN_ZOOM: f32 = 0.7;
const LEGEND_OFFSET: f32 = 12.0;
const LEGEND_WIDTH: f32 = 230.0;

pub struct GraphPaint<'a> {
    pub painter: Painter,
    pub pal: Palette,
    pub viewport: Viewport,
    pub rect: Rect,
    pub active_path: Option<&'a str>,
    pub hovered_index: Option<usize>,
    pub lowercase_filter: &'a str,
}

impl GraphPaint<'_> {
    pub fn draw_edges(&self, nodes: &[SimNode], edges: &EdgeList) {
        let cull_rect = self.rect.expand(CULL_MARGIN);
        for &(from, to) in edges {
            let from_pos = self.viewport.to_screen(nodes[from].pos);
            let to_pos = self.viewport.to_screen(nodes[to].pos);
            if !cull_rect.intersects(Rect::from_two_pos(from_pos, to_pos)) {
                continue;
            }
            let is_highlighted =
                self.is_focused(from, &nodes[from]) || self.is_focused(to, &nodes[to]);
            let stroke = if is_highlighted {
                Stroke::new(1.5, self.pal.accent)
            } else {
                Stroke::new(1.0, self.pal.border_strong)
            };
            self.painter.line_segment([from_pos, to_pos], stroke);
        }
    }

    pub fn draw_nodes(&self, nodes: &[SimNode]) {
        let cull_rect = self.rect.expand(CULL_MARGIN);
        for (index, node) in nodes.iter().enumerate() {
            let center = self.viewport.to_screen(node.pos);
            if cull_rect.contains(center) {
                self.draw_node(index, node, center);
            }
        }
    }

    fn is_focused(&self, index: usize, node: &SimNode) -> bool {
        self.hovered_index == Some(index) || self.active_path == Some(node.path.as_str())
    }

    fn draw_node(&self, index: usize, node: &SimNode, center: egui::Pos2) {
        let radius = draw_radius(node.degree, self.viewport.zoom);
        let is_active = self.active_path == Some(node.path.as_str());
        let is_hovered = self.hovered_index == Some(index);
        let is_match = matches_filter(node, self.lowercase_filter);
        let is_focused = is_active || is_hovered;
        if is_focused {
            self.painter
                .circle_filled(center, radius + 7.0, self.pal.accent_soft);
            self.painter
                .circle_stroke(center, radius + 3.0, Stroke::new(1.5, self.pal.accent));
        }
        self.painter
            .circle_filled(center, radius, self.node_fill(is_focused, is_match));
        let should_label = self.viewport.zoom >= LABEL_MIN_ZOOM
            || is_focused
            || (!self.lowercase_filter.is_empty() && is_match);
        if should_label {
            self.draw_label(node, center, radius, is_focused, is_match);
        }
    }

    fn node_fill(&self, is_focused: bool, is_match: bool) -> Color32 {
        if is_focused {
            self.pal.accent
        } else if is_match {
            self.pal.text_muted.gamma_multiply(0.75)
        } else {
            self.pal.border_strong.gamma_multiply(0.45)
        }
    }

    fn draw_label(
        &self,
        node: &SimNode,
        center: egui::Pos2,
        radius: f32,
        is_focused: bool,
        is_match: bool,
    ) {
        let color = if is_focused {
            self.pal.text_primary
        } else if is_match {
            self.pal.text_muted
        } else {
            self.pal.text_muted.gamma_multiply(0.4)
        };
        let font_size = (11.5 * self.viewport.zoom.clamp(0.8, 1.3)).min(14.0);
        let font = if is_focused {
            semibold(font_size)
        } else {
            FontId::proportional(font_size)
        };
        self.painter.text(
            center + Vec2::new(0.0, radius + 5.0),
            Align2::CENTER_TOP,
            &node.label,
            font,
            color,
        );
    }
}

pub fn show_legend(
    ui: &mut Ui,
    pal: Palette,
    canvas_rect: Rect,
    note_count: usize,
    link_count: usize,
    hovered: Option<&SimNode>,
) {
    let legend_rect = Rect::from_min_size(
        canvas_rect.min + Vec2::splat(LEGEND_OFFSET),
        Vec2::new(LEGEND_WIDTH, 0.0),
    );
    ui.scope_builder(UiBuilder::new().max_rect(legend_rect), |legend_ui| {
        card_frame(pal).show(legend_ui, |card_ui| {
            card_ui.set_width(LEGEND_WIDTH - 20.0);
            card_ui.horizontal(|row| {
                section_label(row, pal, "Graph");
                pill_badge(row, pal, &format!("{note_count} notes"));
                pill_badge(row, pal, &format!("{link_count} links"));
            });
            if let Some(node) = hovered {
                card_ui.add_space(4.0);
                card_ui.add(
                    egui::Label::new(RichText::new(&node.path).size(12.0).color(pal.text_primary))
                        .truncate(),
                );
                card_ui.label(
                    RichText::new(format!("{} connections", node.degree))
                        .size(11.5)
                        .color(pal.text_muted),
                );
            }
        });
    });
}
