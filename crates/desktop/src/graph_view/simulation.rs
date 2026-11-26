use std::collections::HashMap;

use eframe::egui::Vec2;

use crate::state::AppState;

const REPULSION: f32 = 2800.0;
const SPRING_STIFFNESS: f32 = 0.018;
const IDEAL_EDGE_LENGTH: f32 = 95.0;
const GRAVITY: f32 = 0.012;
const MAX_SPEED: f32 = 16.0;

pub struct SimNode {
    pub path: String,
    pub label: String,
    pub pos: Vec2,
    pub vel: Vec2,
    pub degree: usize,
}

pub type EdgeList = Vec<(usize, usize)>;

pub fn ring_position(index: usize, count: usize) -> Vec2 {
    let safe_count = count.max(1) as f32;
    let radius = (safe_count.sqrt() * 55.0).clamp(80.0, 380.0);
    let angle = (index as f32 / safe_count) * std::f32::consts::TAU;
    Vec2::new(angle.cos() * radius, angle.sin() * radius)
}

pub fn build_graph(state: &AppState, previous: &[SimNode]) -> (Vec<SimNode>, EdgeList) {
    let previous_positions: HashMap<&str, Vec2> = previous
        .iter()
        .map(|node| (node.path.as_str(), node.pos))
        .collect();
    let markdown_files: Vec<_> = state
        .files
        .iter()
        .filter(|file| file.kind == cottonmd_vault::EntryKind::Markdown)
        .collect();
    let mut nodes: Vec<SimNode> = markdown_files
        .iter()
        .enumerate()
        .map(|(index, file)| SimNode {
            path: file.relative_path.clone(),
            label: file.stem.clone(),
            pos: previous_positions
                .get(file.relative_path.as_str())
                .copied()
                .unwrap_or_else(|| ring_position(index, markdown_files.len())),
            vel: Vec2::ZERO,
            degree: 0,
        })
        .collect();
    let edges = collect_edges(state, &nodes);
    for &(from, to) in &edges {
        nodes[from].degree += 1;
        nodes[to].degree += 1;
    }
    (nodes, edges)
}

fn collect_edges(state: &AppState, nodes: &[SimNode]) -> EdgeList {
    let index_by_path: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.path.as_str(), index))
        .collect();
    let mut edges = Vec::new();
    for (from, node) in nodes.iter().enumerate() {
        for target in state.graph.outgoing_links(&node.path) {
            match index_by_path.get(target) {
                Some(&to) if to != from => edges.push((from, to)),
                _ => {}
            }
        }
    }
    edges
}

pub fn step_physics(
    nodes: &mut [SimNode],
    edges: &[(usize, usize)],
    dragging: Option<usize>,
) -> f32 {
    if nodes.is_empty() {
        return 0.0;
    }
    let mut forces = repulsion_forces(nodes);
    for &(from, to) in edges {
        let delta = nodes[to].pos - nodes[from].pos;
        let distance = delta.length().max(1.0);
        let pull = (delta / distance) * ((distance - IDEAL_EDGE_LENGTH) * SPRING_STIFFNESS);
        forces[from] += pull;
        forces[to] -= pull;
    }
    let mut total_speed = 0.0_f32;
    for (index, node) in nodes.iter_mut().enumerate() {
        if dragging == Some(index) {
            node.vel = Vec2::ZERO;
            continue;
        }
        forces[index] -= node.pos * GRAVITY;
        total_speed += integrate(node, forces[index]);
    }
    total_speed / nodes.len() as f32
}

fn repulsion_forces(nodes: &[SimNode]) -> Vec<Vec2> {
    let mut forces = vec![Vec2::ZERO; nodes.len()];
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let delta = nodes[i].pos - nodes[j].pos;
            let distance_sq = delta.length_sq().max(25.0);
            let push = (delta / distance_sq.sqrt()) * (REPULSION / distance_sq);
            forces[i] += push;
            forces[j] -= push;
        }
    }
    forces
}

fn integrate(node: &mut SimNode, force: Vec2) -> f32 {
    node.vel = (node.vel + force * 0.45) * 0.82;
    let speed = node.vel.length();
    if speed > MAX_SPEED {
        node.vel = node.vel / speed * MAX_SPEED;
    }
    node.pos += node.vel;
    if !node.pos.is_finite() || !node.vel.is_finite() {
        node.pos = Vec2::ZERO;
        node.vel = Vec2::ZERO;
        return 0.0;
    }
    speed
}

pub fn draw_radius(degree: usize, zoom: f32) -> f32 {
    (4.5 + (degree as f32).sqrt() * 2.2) * zoom
}

pub fn hit_radius(degree: usize, zoom: f32) -> f32 {
    ((5.0 + (degree as f32).sqrt() * 2.5) * zoom).max(8.0)
}

pub fn matches_filter(node: &SimNode, lowercase_query: &str) -> bool {
    lowercase_query.is_empty()
        || node.label.to_ascii_lowercase().contains(lowercase_query)
        || node.path.to_ascii_lowercase().contains(lowercase_query)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_at(x: f32, y: f32) -> SimNode {
        SimNode {
            path: "note.md".into(),
            label: "Note".into(),
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            degree: 0,
        }
    }

    #[test]
    fn empty_graph_has_zero_energy() {
        assert_eq!(step_physics(&mut [], &[], None), 0.0);
    }

    #[test]
    fn overlapping_nodes_are_pushed_apart_without_nan() {
        let mut nodes = vec![node_at(0.0, 0.0), node_at(0.0, 0.0)];
        let energy = step_physics(&mut nodes, &[], None);
        assert!(energy.is_finite());
        assert!(nodes.iter().all(|node| node.pos.is_finite()));
    }

    #[test]
    fn dragged_node_stays_in_place() {
        let mut nodes = vec![node_at(10.0, 0.0), node_at(11.0, 0.0)];
        step_physics(&mut nodes, &[(0, 1)], Some(0));
        assert_eq!(nodes[0].pos, Vec2::new(10.0, 0.0));
    }

    #[test]
    fn non_finite_position_is_reset() {
        let mut nodes = vec![node_at(f32::NAN, 0.0)];
        step_physics(&mut nodes, &[], None);
        assert_eq!(nodes[0].pos, Vec2::ZERO);
    }

    #[test]
    fn ring_position_handles_zero_count() {
        assert!(ring_position(0, 0).is_finite());
    }

    #[test]
    fn filter_matches_label_or_path_case_insensitively() {
        let node = node_at(0.0, 0.0);
        assert!(matches_filter(&node, ""));
        assert!(matches_filter(&node, "note"));
        assert!(!matches_filter(&node, "missing"));
    }

    #[test]
    fn hit_radius_never_drops_below_tap_target() {
        assert_eq!(hit_radius(0, 0.25), 8.0);
    }
}
