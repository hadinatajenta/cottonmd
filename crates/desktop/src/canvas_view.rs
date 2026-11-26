use std::collections::HashMap;

use cottonmd_canvas::{CanvasDocument, CanvasNode};
use eframe::egui::{
    self, vec2, CornerRadius, Layout, Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2,
};

mod edges;
mod node_card;
mod node_content;
pub mod viewport;

use edges::{draw_edge, EdgeGeometry};
use node_card::{draw_node_card, NodeVisual};
use node_content::{describe_node, is_group};
use viewport::{sanitize_pan, sanitize_zoom, zoom_after_scroll, Viewport};

use crate::theme::{Palette, RADIUS_CARD};
use crate::ui_kit::{empty_state, icon_button, paint_dot_grid, pill_badge, section_label};

const DEFAULT_ZOOM: f32 = 0.85;
const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 3.0;

pub struct CanvasViewState {
    pub pan: Vec2,
    pub zoom: f32,
    selected_node_id: Option<String>,
}

impl Default for CanvasViewState {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: DEFAULT_ZOOM,
            selected_node_id: None,
        }
    }
}

struct SceneFrame<'a> {
    viewport: Viewport,
    visible_rect: Rect,
    node_rects: HashMap<&'a str, Rect>,
}

impl CanvasViewState {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        doc: Option<&CanvasDocument>,
        pal: Palette,
    ) -> Option<String> {
        self.zoom = sanitize_zoom(self.zoom, MIN_ZOOM, MAX_ZOOM, DEFAULT_ZOOM);
        self.pan = sanitize_pan(self.pan);
        self.show_toolbar(ui, pal, doc);
        ui.add_space(8.0);
        let Some(doc) = doc else {
            empty_state(
                ui,
                pal,
                egui_phosphor::regular::SQUARES_FOUR,
                "Select a .canvas file from the Files sidebar to render it here.",
            );
            return None;
        };
        self.show_surface(ui, doc, pal)
    }

    fn show_toolbar(&mut self, ui: &mut Ui, pal: Palette, doc: Option<&CanvasDocument>) {
        ui.horizontal(|ui| {
            section_label(ui, pal, "Canvas");
            if let Some(doc) = doc {
                pill_badge(ui, pal, &format!("{} nodes", doc.nodes.len()));
                pill_badge(ui, pal, &format!("{} edges", doc.edges.len()));
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if icon_button(ui, pal, egui_phosphor::regular::CROSSHAIR, "Reset view").clicked() {
                    self.pan = Vec2::ZERO;
                    self.zoom = DEFAULT_ZOOM;
                }
            });
        });
    }

    fn show_surface(&mut self, ui: &mut Ui, doc: &CanvasDocument, pal: Palette) -> Option<String> {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        self.apply_navigation(ui, &response);
        let viewport = Viewport::new(rect, self.pan, self.zoom);
        ui.painter().rect(
            rect,
            CornerRadius::same(RADIUS_CARD),
            pal.canvas,
            Stroke::new(1.0, pal.border),
            egui::StrokeKind::Inside,
        );
        paint_dot_grid(ui, pal, rect, viewport.grid_offset(rect));
        if doc.nodes.is_empty() {
            self.show_empty_document(ui, pal, rect);
            return None;
        }
        let mut surface = ui.new_child(UiBuilder::new().max_rect(rect));
        surface.set_clip_rect(rect.intersect(ui.clip_rect()));
        let scene = build_scene(doc, viewport, rect);
        self.draw_edges(&surface, pal, doc, &scene);
        let ordered_nodes = nodes_in_draw_order(doc);
        let hovered_id = hovered_node_id(&response, &ordered_nodes, &scene);
        self.draw_nodes(&mut surface, pal, &ordered_nodes, &scene, hovered_id);
        self.apply_click(ui, doc, &response, hovered_id)
    }

    fn show_empty_document(&self, ui: &mut Ui, pal: Palette, rect: Rect) {
        let mut empty_ui = ui.new_child(UiBuilder::new().max_rect(rect));
        empty_state(
            &mut empty_ui,
            pal,
            egui_phosphor::regular::SQUARES_FOUR,
            "This canvas has no nodes yet.",
        );
    }

    fn apply_navigation(&mut self, ui: &Ui, response: &Response) {
        if response.dragged() {
            self.pan = sanitize_pan(self.pan + response.drag_delta());
        }
        if response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > 0.01 {
                self.zoom = zoom_after_scroll(self.zoom, scroll, MIN_ZOOM, MAX_ZOOM);
            }
        }
    }

    fn draw_edges(&self, surface: &Ui, pal: Palette, doc: &CanvasDocument, scene: &SceneFrame) {
        let painter = surface.painter();
        for edge in &doc.edges {
            let (Some(&from_rect), Some(&to_rect)) = (
                scene.node_rects.get(edge.from_node.as_str()),
                scene.node_rects.get(edge.to_node.as_str()),
            ) else {
                continue;
            };
            let geometry = EdgeGeometry::between(from_rect, to_rect, edge);
            if !geometry.is_finite() || !geometry.bounds().intersects(scene.visible_rect) {
                continue;
            }
            let is_highlighted = self
                .selected_node_id
                .as_deref()
                .is_some_and(|selected| selected == edge.from_node || selected == edge.to_node);
            draw_edge(painter, pal, edge, &geometry, is_highlighted);
        }
    }

    fn draw_nodes(
        &self,
        surface: &mut Ui,
        pal: Palette,
        ordered_nodes: &[&CanvasNode],
        scene: &SceneFrame,
        hovered_id: Option<&str>,
    ) {
        for node in ordered_nodes {
            let Some(&node_rect) = scene.node_rects.get(node.id.as_str()) else {
                continue;
            };
            if !node_rect.intersects(scene.visible_rect) {
                continue;
            }
            let visual = NodeVisual {
                is_selected: self.selected_node_id.as_deref() == Some(node.id.as_str()),
                is_hovered: hovered_id == Some(node.id.as_str()),
                zoom: scene.viewport.zoom,
            };
            draw_node_card(surface, pal, node_rect, node, visual);
        }
    }

    fn apply_click(
        &mut self,
        ui: &Ui,
        doc: &CanvasDocument,
        response: &Response,
        hovered_id: Option<&str>,
    ) -> Option<String> {
        let hovered_node = hovered_id.and_then(|id| doc.node(id));
        let linked_file = hovered_node.and_then(|node| describe_node(&node.kind).linked_file);
        if linked_file.is_some() && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if !response.clicked() {
            return None;
        }
        self.selected_node_id = hovered_node.map(|node| node.id.clone());
        linked_file
    }
}

fn build_scene(doc: &CanvasDocument, viewport: Viewport, visible_rect: Rect) -> SceneFrame<'_> {
    let node_rects = doc
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node_screen_rect(&viewport, node)))
        .filter(|(_, rect)| rect.is_finite() && rect.is_positive())
        .collect();
    SceneFrame {
        viewport,
        visible_rect,
        node_rects,
    }
}

fn node_screen_rect(viewport: &Viewport, node: &CanvasNode) -> Rect {
    let min: Pos2 = viewport.to_screen(vec2(node.bounds.x as f32, node.bounds.y as f32));
    let size = vec2(node.bounds.width as f32, node.bounds.height as f32) * viewport.zoom;
    Rect::from_min_size(min, size)
}

fn nodes_in_draw_order(doc: &CanvasDocument) -> Vec<&CanvasNode> {
    let (groups, others): (Vec<_>, Vec<_>) =
        doc.nodes.iter().partition(|node| is_group(&node.kind));
    groups.into_iter().chain(others).collect()
}

fn hovered_node_id<'a>(
    response: &Response,
    ordered_nodes: &[&'a CanvasNode],
    scene: &SceneFrame,
) -> Option<&'a str> {
    let pointer = response.hover_pos()?;
    ordered_nodes
        .iter()
        .rev()
        .find(|node| {
            scene
                .node_rects
                .get(node.id.as_str())
                .is_some_and(|rect| rect.contains(pointer))
        })
        .map(|node| node.id.as_str())
}
