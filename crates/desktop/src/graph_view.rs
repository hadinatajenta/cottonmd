use eframe::egui::{self, vec2, CornerRadius, Layout, Response, Sense, Stroke, Ui, Vec2};

mod paint;
mod simulation;

use paint::{show_legend, GraphPaint};
use simulation::{build_graph, hit_radius, step_physics, EdgeList, SimNode};

use crate::canvas_view::viewport::{sanitize_pan, sanitize_zoom, zoom_after_scroll, Viewport};
use crate::state::AppState;
use crate::theme::{Palette, RADIUS_CARD};
use crate::ui_kit::{empty_state, icon_button, paint_dot_grid, search_field};

const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 3.5;
const RESTING_ENERGY: f32 = 0.04;
const SEARCH_WIDTH: f32 = 220.0;

pub struct GraphViewState {
    revision: u64,
    nodes: Vec<SimNode>,
    edges: EdgeList,
    pan: Vec2,
    zoom: f32,
    dragging_node: Option<usize>,
    energy: f32,
    filter: String,
}

impl Default for GraphViewState {
    fn default() -> Self {
        Self {
            revision: u64::MAX,
            nodes: Vec::new(),
            edges: Vec::new(),
            pan: Vec2::ZERO,
            zoom: 1.0,
            dragging_node: None,
            energy: 0.0,
            filter: String::new(),
        }
    }
}

impl GraphViewState {
    fn sync_from_state(&mut self, state: &AppState) {
        if self.revision == state.graph_revision {
            return;
        }
        self.revision = state.graph_revision;
        let (nodes, edges) = build_graph(state, &self.nodes);
        self.nodes = nodes;
        self.edges = edges;
        self.dragging_node = None;
        self.energy = 1.0;
    }

    pub fn ui(&mut self, ui: &mut Ui, state: &AppState, pal: Palette) -> Option<String> {
        self.sync_from_state(state);
        self.zoom = sanitize_zoom(self.zoom, MIN_ZOOM, MAX_ZOOM, 1.0);
        self.pan = sanitize_pan(self.pan);
        self.show_toolbar(ui, pal);
        ui.add_space(8.0);
        if self.nodes.is_empty() {
            empty_state(
                ui,
                pal,
                egui_phosphor::regular::GRAPH,
                "No markdown notes to graph. Open a vault or create a note.",
            );
            return None;
        }
        if self.energy >= RESTING_ENERGY || self.dragging_node.is_some() {
            self.energy = step_physics(&mut self.nodes, &self.edges, self.dragging_node);
            ui.ctx().request_repaint();
        }
        self.show_canvas(ui, state, pal)
    }

    fn show_toolbar(&mut self, ui: &mut Ui, pal: Palette) {
        ui.horizontal(|ui| {
            ui.allocate_ui(vec2(SEARCH_WIDTH, 32.0), |ui| {
                search_field(ui, pal, &mut self.filter, "Filter nodes...", "");
            });
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if icon_button(
                    ui,
                    pal,
                    egui_phosphor::regular::ARROWS_CLOCKWISE,
                    "Re-layout",
                )
                .clicked()
                {
                    self.energy = 1.0;
                }
                if icon_button(ui, pal, egui_phosphor::regular::CROSSHAIR, "Reset view").clicked() {
                    self.pan = Vec2::ZERO;
                    self.zoom = 1.0;
                }
            });
        });
    }

    fn show_canvas(&mut self, ui: &mut Ui, state: &AppState, pal: Palette) -> Option<String> {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        self.apply_scroll_zoom(ui, &response);
        let viewport = Viewport::new(rect, self.pan, self.zoom);
        ui.painter().rect(
            rect,
            CornerRadius::same(RADIUS_CARD),
            pal.canvas,
            Stroke::new(1.0, pal.border),
            egui::StrokeKind::Inside,
        );
        paint_dot_grid(ui, pal, rect, viewport.grid_offset(rect));
        let hovered_index = self.hovered_node_index(&response, &viewport);
        let clicked_path = self.apply_pointer(&response, &viewport, hovered_index);
        let lowercase_filter = self.filter.trim().to_ascii_lowercase();
        let graph_paint = GraphPaint {
            painter: ui.painter_at(rect),
            pal,
            viewport,
            rect,
            active_path: state.active_path.as_deref(),
            hovered_index,
            lowercase_filter: &lowercase_filter,
        };
        graph_paint.draw_edges(&self.nodes, &self.edges);
        graph_paint.draw_nodes(&self.nodes);
        if hovered_index.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        show_legend(
            ui,
            pal,
            rect,
            self.nodes.len(),
            self.edges.len(),
            hovered_index.map(|index| &self.nodes[index]),
        );
        clicked_path
    }

    fn apply_scroll_zoom(&mut self, ui: &Ui, response: &Response) {
        if !response.hovered() {
            return;
        }
        let scroll = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll.abs() > 0.01 {
            self.zoom = zoom_after_scroll(self.zoom, scroll, MIN_ZOOM, MAX_ZOOM);
        }
    }

    fn hovered_node_index(&self, response: &Response, viewport: &Viewport) -> Option<usize> {
        let pointer = response.hover_pos()?;
        self.nodes.iter().position(|node| {
            viewport.to_screen(node.pos).distance(pointer) <= hit_radius(node.degree, viewport.zoom)
        })
    }

    fn apply_pointer(
        &mut self,
        response: &Response,
        viewport: &Viewport,
        hovered_index: Option<usize>,
    ) -> Option<String> {
        if response.drag_started() {
            self.dragging_node = hovered_index;
        }
        if response.dragged() {
            self.drag_selection(response, viewport);
        }
        if response.drag_stopped() {
            self.dragging_node = None;
        }
        if response.clicked() {
            return hovered_index.map(|index| self.nodes[index].path.clone());
        }
        None
    }

    fn drag_selection(&mut self, response: &Response, viewport: &Viewport) {
        let Some(index) = self.dragging_node else {
            self.pan = sanitize_pan(self.pan + response.drag_delta());
            return;
        };
        if let Some(pointer) = response.interact_pointer_pos() {
            self.nodes[index].pos = viewport.to_world(pointer);
            self.energy = 0.5;
        }
    }
}
