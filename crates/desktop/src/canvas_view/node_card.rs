use cottonmd_canvas::CanvasNode;
use eframe::egui::{
    self,
    text::{LayoutJob, TextFormat, TextWrapping},
    Align, Align2, Color32, FontId, Galley, Layout, Painter, Rect, Shadow, Stroke, Ui, UiBuilder,
    Vec2,
};
use std::sync::Arc;

use super::node_content::{describe_node, is_group, NodeContent};
use crate::theme::{semibold, Palette};
use crate::ui_kit::{node_card_frame, pill_badge};

const DETAIL_MIN_WIDTH: f32 = 90.0;
const BADGE_MIN_WIDTH: f32 = 170.0;
const HEADER_PADDING: f32 = 10.0;
const BODY_PADDING: f32 = 12.0;

#[derive(Clone, Copy)]
pub struct NodeVisual {
    pub is_selected: bool,
    pub is_hovered: bool,
    pub zoom: f32,
}

pub fn draw_node_card(
    ui: &mut Ui,
    pal: Palette,
    rect: Rect,
    node: &CanvasNode,
    visual: NodeVisual,
) {
    let is_detailed = rect.width() >= DETAIL_MIN_WIDTH && rect.height() >= DETAIL_MIN_WIDTH / 2.0;
    paint_card_background(ui, pal, rect, is_group(&node.kind), is_detailed, visual);
    if !is_detailed {
        return;
    }
    let content = describe_node(&node.kind);
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let header_height = (32.0 * visual.zoom).clamp(22.0, 44.0);
    let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));
    paint_header(ui, &painter, pal, header_rect, &content, visual.zoom);
    if !content.body.is_empty() {
        paint_body(&painter, pal, rect, header_height, &content, visual.zoom);
    }
}

fn paint_card_background(
    ui: &Ui,
    pal: Palette,
    rect: Rect,
    is_group_node: bool,
    is_detailed: bool,
    visual: NodeVisual,
) {
    let mut frame = node_card_frame(pal, visual.is_selected);
    if visual.is_hovered && !visual.is_selected {
        frame = frame.stroke(Stroke::new(1.0, pal.accent));
    }
    if is_group_node {
        frame = frame.fill(pal.bg_elevated.gamma_multiply(0.55));
    }
    if is_group_node || !is_detailed {
        frame = frame.shadow(Shadow::NONE);
    }
    let content_rect = rect.shrink(frame.stroke.width);
    ui.painter().add(frame.paint(content_rect));
}

fn paint_header(
    ui: &mut Ui,
    painter: &Painter,
    pal: Palette,
    header_rect: Rect,
    content: &NodeContent,
    zoom: f32,
) {
    let font_size = (13.0 * zoom).clamp(10.0, 18.0);
    let has_badge = header_rect.width() >= BADGE_MIN_WIDTH;
    let badge_reserve = if has_badge { 70.0 } else { 0.0 };
    let icon_center = header_rect.left_center() + Vec2::new(HEADER_PADDING + font_size * 0.5, 0.0);
    let icon_color = if content.linked_file.is_some() {
        pal.accent
    } else {
        pal.text_muted
    };
    painter.text(
        icon_center,
        Align2::CENTER_CENTER,
        content.icon,
        FontId::proportional(font_size + 2.0),
        icon_color,
    );
    let title_left = icon_center.x + font_size + 6.0;
    let title_width = (header_rect.right() - HEADER_PADDING - badge_reserve - title_left).max(0.0);
    let title_color = if content.linked_file.is_some() {
        pal.accent
    } else {
        pal.text_primary
    };
    let galley = single_line_galley(
        painter,
        &content.title,
        semibold(font_size),
        title_color,
        title_width,
    );
    let title_pos = egui::pos2(title_left, header_rect.center().y - galley.size().y / 2.0);
    painter.galley(title_pos, galley, title_color);
    if has_badge {
        paint_kind_badge(ui, pal, header_rect, content.kind_label);
    }
    painter.hline(
        header_rect.x_range(),
        header_rect.bottom(),
        Stroke::new(1.0, pal.border),
    );
}

fn paint_kind_badge(ui: &mut Ui, pal: Palette, header_rect: Rect, kind_label: &str) {
    let badge_area = header_rect.shrink2(Vec2::new(HEADER_PADDING, 0.0));
    let clip = header_rect.intersect(ui.clip_rect());
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(badge_area)
            .layout(Layout::right_to_left(Align::Center)),
        |badge_ui| {
            badge_ui.set_clip_rect(clip);
            pill_badge(badge_ui, pal, kind_label);
        },
    );
}

fn paint_body(
    painter: &Painter,
    pal: Palette,
    rect: Rect,
    header_height: f32,
    content: &NodeContent,
    zoom: f32,
) {
    let font_size = (12.5 * zoom).clamp(8.0, 18.0);
    let wrap_width = (rect.width() - BODY_PADDING * 2.0).max(0.0);
    let (font, color) = if content.linked_file.is_some() {
        (FontId::monospace(font_size), pal.text_muted)
    } else {
        (FontId::proportional(font_size), pal.text_primary)
    };
    let galley = painter.layout(content.body.clone(), font, color, wrap_width);
    let body_pos = rect.min + Vec2::new(BODY_PADDING, header_height + BODY_PADDING * 0.8);
    painter.galley(body_pos, galley, color);
}

fn single_line_galley(
    painter: &Painter,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(text.to_owned(), TextFormat::simple(font, color));
    job.wrap = TextWrapping {
        max_width,
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    painter.layout_job(job)
}
