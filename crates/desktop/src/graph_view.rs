use std::collections::HashMap;

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use crate::state::AppState;
use crate::theme::Palette;

struct SimNode {
    path: String,
    label: String,
    pos: Vec2,
    vel: Vec2,
    degree: usize,
}

pub struct GraphViewState {
    revision: u64,
    nodes: Vec<SimNode>,
    edges: Vec<(usize, usize)>,
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

        let mut old_pos: HashMap<String, Vec2> = HashMap::new();
        for n in &self.nodes {
            old_pos.insert(n.path.clone(), n.pos);
        }

        let md_files: Vec<_> = state
            .files
            .iter()
            .filter(|f| f.kind == cottonmd_vault::EntryKind::Markdown)
            .collect();

        let count = md_files.len().max(1) as f32;
        let radius = (count.sqrt() * 55.0).clamp(80.0, 380.0);

        let mut idx_by_path = HashMap::new();
        let mut nodes = Vec::with_capacity(md_files.len());

        for (i, f) in md_files.iter().enumerate() {
            let angle = (i as f32 / count) * std::f32::consts::TAU;
            let pos = old_pos.get(&f.relative_path).copied().unwrap_or_else(|| {
                Vec2::new(angle.cos() * radius, angle.sin() * radius)
            });
            idx_by_path.insert(f.relative_path.clone(), i);
            nodes.push(SimNode {
                path: f.relative_path.clone(),
                label: f.stem.clone(),
                pos,
                vel: Vec2::ZERO,
                degree: 0,
            });
        }

        let mut edges = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            for target in state.graph.outgoing_links(&node.path) {
                if let Some(&j) = idx_by_path.get(target) {
                    if i != j {
                        edges.push((i, j));
                    }
                }
            }
        }

        for &(a, b) in &edges {
            nodes[a].degree += 1;
            nodes[b].degree += 1;
        }

        self.nodes = nodes;
        self.edges = edges;
        self.dragging_node = None;
        self.energy = 1.0;
    }

    fn step_physics(&mut self) {
        let n = self.nodes.len();
        if n == 0 || self.energy < 0.01 {
            return;
        }

        let mut forces = vec![Vec2::ZERO; n];
        let repulsion = 2800.0_f32;
        let spring_k = 0.018_f32;
        let ideal_len = 95.0_f32;
        let gravity = 0.012_f32;

        for i in 0..n {
            for j in (i + 1)..n {
                let delta = self.nodes[i].pos - self.nodes[j].pos;
                let dist_sq = delta.length_sq().max(25.0);
                let dist = dist_sq.sqrt();
                let dir = delta / dist;
                let f = dir * (repulsion / dist_sq);
                forces[i] += f;
                forces[j] -= f;
            }
        }

        for &(a, b) in &self.edges {
            let delta = self.nodes[b].pos - self.nodes[a].pos;
            let dist = delta.length().max(1.0);
            let displacement = dist - ideal_len;
            let f = (delta / dist) * (displacement * spring_k);
            forces[a] += f;
            forces[b] -= f;
        }

        let mut total_speed = 0.0_f32;
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if self.dragging_node == Some(i) {
                node.vel = Vec2::ZERO;
                continue;
            }
            forces[i] -= node.pos * gravity;
            node.vel = (node.vel + forces[i] * 0.45) * 0.82;
            let speed = node.vel.length();
            if speed > 16.0 {
                node.vel = node.vel / speed * 16.0;
            }
            node.pos += node.vel;
            total_speed += speed;
        }

        self.energy = total_speed / (n as f32);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, state: &AppState, pal: Palette) -> Option<String> {
        self.sync_from_state(state);

        let mut clicked_path = None;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Graph").strong().color(pal.text_primary));
            ui.separator();
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("Filter nodes...")
                    .desired_width(180.0),
            );
            if ui.button("Reset view").clicked() {
                self.pan = Vec2::ZERO;
                self.zoom = 1.0;
            }
            if ui.button("Re-layout").clicked() {
                self.energy = 1.0;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} nodes  |  {} links",
                        self.nodes.len(),
                        self.edges.len()
                    ))
                    .size(11.5)
                    .color(pal.text_muted),
                );
            });
        });
        ui.separator();

        if self.nodes.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No markdown notes to graph. Open a vault or create a note.")
                        .color(pal.text_muted),
                );
            });
            return None;
        }

        if self.energy >= 0.04 || self.dragging_node.is_some() {
            self.step_physics();
            ui.ctx().request_repaint();
        }

        let avail = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(avail, Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, pal.bg_editor);

        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.01 {
                let factor = (1.0 + scroll * 0.002).clamp(0.8, 1.25);
                self.zoom = (self.zoom * factor).clamp(0.25, 3.5);
            }
        }

        let center = rect.center() + self.pan;
        let to_screen = |world: Vec2| -> Pos2 { center + world * self.zoom };
        let to_world = |screen: Pos2| -> Vec2 { (screen - center) / self.zoom };

        let pointer_pos = response.hover_pos();
        let hovered_node = pointer_pos.and_then(|p| {
            self.nodes.iter().enumerate().find_map(|(i, n)| {
                let r = (5.0 + (n.degree as f32).sqrt() * 2.5) * self.zoom;
                let sp = to_screen(n.pos);
                (sp.distance(p) <= r.max(8.0)).then_some(i)
            })
        });

        if response.drag_started() {
            if let Some(idx) = hovered_node {
                self.dragging_node = Some(idx);
            }
        }
        if response.dragged() {
            if let Some(idx) = self.dragging_node {
                if let Some(p) = response.interact_pointer_pos() {
                    self.nodes[idx].pos = to_world(p);
                    self.energy = 0.5;
                }
            } else {
                self.pan += response.drag_delta();
            }
        }
        if response.drag_stopped() {
            self.dragging_node = None;
        }
        if response.clicked() {
            if let Some(idx) = hovered_node {
                clicked_path = Some(self.nodes[idx].path.clone());
            }
        }

        let filter_q = self.filter.trim().to_ascii_lowercase();
        let active_path = state.active_path.as_deref();

        for &(a, b) in &self.edges {
            let pa = to_screen(self.nodes[a].pos);
            let pb = to_screen(self.nodes[b].pos);
            let highlighted = hovered_node == Some(a)
                || hovered_node == Some(b)
                || active_path == Some(self.nodes[a].path.as_str())
                || active_path == Some(self.nodes[b].path.as_str());
            let stroke = if highlighted {
                Stroke::new(1.5, pal.accent)
            } else {
                Stroke::new(1.0, pal.border)
            };
            painter.line_segment([pa, pb], stroke);
        }

        for (i, node) in self.nodes.iter().enumerate() {
            let matches_filter = filter_q.is_empty()
                || node.label.to_ascii_lowercase().contains(&filter_q)
                || node.path.to_ascii_lowercase().contains(&filter_q);

            let sp = to_screen(node.pos);
            if !rect.expand(40.0).contains(sp) {
                continue;
            }
            let r = (4.5 + (node.degree as f32).sqrt() * 2.2) * self.zoom;
            let is_active = active_path == Some(node.path.as_str());
            let is_hovered = hovered_node == Some(i);

            let fill = if is_active || is_hovered {
                pal.accent
            } else if matches_filter {
                pal.text_muted
            } else {
                Color32::from_rgba_unmultiplied(
                    pal.border.r(),
                    pal.border.g(),
                    pal.border.b(),
                    110,
                )
            };

            painter.circle_filled(sp, r, fill);
            if is_active {
                painter.circle_stroke(sp, r + 2.5, Stroke::new(1.5, pal.accent));
            }

            if self.zoom >= 0.7 || is_active || is_hovered || !filter_q.is_empty() && matches_filter {
                let text_color = if is_active || is_hovered {
                    pal.text_primary
                } else if matches_filter {
                    pal.text_muted
                } else {
                    Color32::from_rgba_unmultiplied(
                        pal.text_muted.r(),
                        pal.text_muted.g(),
                        pal.text_muted.b(),
                        70,
                    )
                };
                painter.text(
                    sp + Vec2::new(0.0, r + 4.0),
                    Align2::CENTER_TOP,
                    &node.label,
                    FontId::proportional((11.5 * self.zoom.clamp(0.8, 1.3)).min(14.0)),
                    text_color,
                );
            }
        }

        if let Some(idx) = hovered_node {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            let n = &self.nodes[idx];
            let tip_rect = Rect::from_min_size(rect.min + Vec2::new(12.0, 12.0), Vec2::new(260.0, 24.0));
            painter.rect_filled(tip_rect, 4.0, pal.bg_elevated);
            painter.rect_stroke(tip_rect, 4.0, Stroke::new(1.0, pal.border), egui::StrokeKind::Inside);
            painter.text(
                tip_rect.left_center() + Vec2::new(8.0, 0.0),
                Align2::LEFT_CENTER,
                format!("{} ({} links)", n.path, n.degree),
                FontId::proportional(11.5),
                pal.text_primary,
            );
        }

        clicked_path
    }
}
