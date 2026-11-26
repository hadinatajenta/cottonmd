use eframe::egui::{
    self, Align2, Color32, CornerRadius, Frame, Margin, Rect, Response, RichText, Sense, Shadow,
    Stroke, StrokeKind, Ui, Vec2,
};

use crate::theme::{semibold, Palette, RADIUS_BADGE, RADIUS_CARD, RADIUS_CONTROL};

const DOT_GRID_SPACING: f32 = 22.0;
const DOT_RADIUS: f32 = 1.0;
const TREE_INDENT: f32 = 16.0;

pub fn card_frame(pal: Palette) -> Frame {
    Frame::new()
        .fill(pal.bg_editor)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(RADIUS_CARD))
        .inner_margin(Margin::same(10))
        .shadow(Shadow {
            offset: [0, 2],
            blur: 12,
            spread: 0,
            color: pal.shadow,
        })
}

pub fn canvas_frame(pal: Palette) -> Frame {
    Frame::new().fill(pal.canvas).inner_margin(Margin::same(8))
}

pub fn underline_tabs<T: Copy + PartialEq>(
    ui: &mut Ui,
    pal: Palette,
    current: &mut T,
    tabs: &[(T, &str, &str)],
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 18.0;
        for (value, icon, label) in tabs {
            let is_active = *current == *value;
            let color = if is_active { pal.text_primary } else { pal.text_muted };
            let text = if icon.is_empty() {
                (*label).to_owned()
            } else {
                format!("{icon}  {label}")
            };
            let galley = ui.painter().layout_no_wrap(
                text,
                if is_active { semibold(13.5) } else { egui::FontId::proportional(13.5) },
                color,
            );
            let size = Vec2::new(galley.size().x, galley.size().y + 12.0);
            let (rect, response) = ui.allocate_exact_size(size, Sense::click());
            let text_color = if response.hovered() { pal.text_primary } else { color };
            ui.painter().galley(
                egui::pos2(rect.left(), rect.top() + 4.0),
                galley,
                text_color,
            );
            if is_active {
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        egui::pos2(rect.left(), rect.bottom() - 2.0),
                        Vec2::new(rect.width(), 2.0),
                    ),
                    CornerRadius::same(1),
                    pal.text_primary,
                );
            }
            if response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !is_active {
                *current = *value;
                changed = true;
            }
        }
    });
    changed
}

pub fn pill_badge(ui: &mut Ui, pal: Palette, text: &str) -> Response {
    let galley = ui.painter().layout_no_wrap(
        text.to_uppercase(),
        semibold(10.5),
        pal.text_muted,
    );
    let size = galley.size() + Vec2::new(12.0, 6.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(RADIUS_BADGE), pal.badge_bg);
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        pal.text_muted,
    );
    response
}

pub fn type_chip(ui: &mut Ui, pal: Palette, text: &str) -> Response {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::monospace(12.0),
        pal.text_primary,
    );
    let size = galley.size() + Vec2::new(14.0, 6.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(RADIUS_BADGE), pal.bg_elevated);
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        pal.text_primary,
    );
    response
}

pub fn search_field(
    ui: &mut Ui,
    pal: Palette,
    query: &mut String,
    hint: &str,
    shortcut_hint: &str,
) -> Response {
    let width = ui.available_width();
    let height = 32.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        pal.bg_editor,
        Stroke::new(1.0, pal.border_strong),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_center() + Vec2::new(10.0, 0.0),
        Align2::LEFT_CENTER,
        egui_phosphor::regular::MAGNIFYING_GLASS,
        egui::FontId::proportional(15.0),
        pal.text_muted,
    );
    let hint_width = if shortcut_hint.is_empty() { 0.0 } else { 28.0 };
    let edit_rect = Rect::from_min_max(
        rect.min + Vec2::new(32.0, 2.0),
        rect.max - Vec2::new(hint_width + 6.0, 2.0),
    );
    let response = ui.put(
        edit_rect,
        egui::TextEdit::singleline(query)
            .hint_text(RichText::new(hint).color(pal.text_muted))
            .frame(false)
            .desired_width(edit_rect.width())
            .vertical_align(egui::Align::Center),
    );
    if !shortcut_hint.is_empty() {
        let badge_rect = Rect::from_center_size(
            rect.right_center() - Vec2::new(16.0, 0.0),
            Vec2::new(20.0, 20.0),
        );
        ui.painter()
            .rect_filled(badge_rect, CornerRadius::same(RADIUS_BADGE), pal.bg_elevated);
        ui.painter().text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            shortcut_hint,
            egui::FontId::proportional(12.0),
            pal.text_muted,
        );
    }
    response
}

pub fn toggle_switch(ui: &mut Ui, pal: Palette, is_on: &mut bool) -> Response {
    let size = Vec2::new(34.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *is_on = !*is_on;
        response.mark_changed();
    }
    let progress = ui.ctx().animate_bool(response.id, *is_on);
    let track_color = if *is_on { pal.accent } else { pal.border_strong };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(10), track_color);
    let knob_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), progress);
    ui.painter()
        .circle_filled(egui::pos2(knob_x, rect.center().y), 7.5, Color32::WHITE);
    response
}

pub fn icon_button(ui: &mut Ui, pal: Palette, icon: &str, tooltip: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(RADIUS_CONTROL), pal.bg_elevated);
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(17.0),
        if response.hovered() { pal.text_primary } else { pal.text_muted },
    );
    response
        .on_hover_text(tooltip)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn outlined_icon_button(ui: &mut Ui, pal: Palette, icon: &str, tooltip: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::click());
    let fill = if response.hovered() { pal.bg_elevated } else { pal.bg_editor };
    ui.painter().rect(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        fill,
        Stroke::new(1.0, pal.border_strong),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(18.0),
        pal.text_primary,
    );
    response
        .on_hover_text(tooltip)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub struct TreeRow<'a> {
    pub depth: usize,
    pub icon: &'a str,
    pub label: &'a str,
    pub meta: &'a str,
    pub is_expanded: Option<bool>,
    pub is_selected: bool,
    pub is_accented: bool,
}

pub fn tree_row(ui: &mut Ui, pal: Palette, row: &TreeRow) -> Response {
    let height = 28.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let painter = ui.painter();
    if row.is_selected {
        painter.rect_filled(rect, CornerRadius::same(RADIUS_CONTROL), pal.bg_elevated);
    } else if response.hovered() {
        painter.rect_filled(
            rect,
            CornerRadius::same(RADIUS_CONTROL),
            pal.bg_elevated.gamma_multiply(0.6),
        );
    }
    for level in 0..row.depth {
        let guide_x = rect.left() + 14.0 + level as f32 * TREE_INDENT;
        painter.line_segment(
            [egui::pos2(guide_x, rect.top()), egui::pos2(guide_x, rect.bottom())],
            Stroke::new(1.0, pal.border),
        );
    }
    let content_left = rect.left() + 6.0 + row.depth as f32 * TREE_INDENT;
    if let Some(is_expanded) = row.is_expanded {
        let chevron = if is_expanded {
            egui_phosphor::regular::CARET_DOWN
        } else {
            egui_phosphor::regular::CARET_RIGHT
        };
        painter.text(
            egui::pos2(content_left + 6.0, rect.center().y),
            Align2::CENTER_CENTER,
            chevron,
            egui::FontId::proportional(12.0),
            pal.text_muted,
        );
    }
    let icon_color = if row.is_accented { pal.accent } else { pal.text_muted };
    painter.text(
        egui::pos2(content_left + 24.0, rect.center().y),
        Align2::CENTER_CENTER,
        row.icon,
        egui::FontId::proportional(15.0),
        icon_color,
    );
    let label_font = if row.is_selected { semibold(13.5) } else { egui::FontId::proportional(13.5) };
    painter.text(
        egui::pos2(content_left + 40.0, rect.center().y),
        Align2::LEFT_CENTER,
        row.label,
        label_font,
        pal.text_primary,
    );
    if !row.meta.is_empty() {
        painter.text(
            egui::pos2(rect.right() - 8.0, rect.center().y),
            Align2::RIGHT_CENTER,
            row.meta,
            egui::FontId::proportional(12.0),
            pal.text_muted,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn paint_dot_grid(ui: &Ui, pal: Palette, rect: Rect, offset: Vec2) {
    let painter = ui.painter_at(rect);
    let start_x = rect.left() + offset.x.rem_euclid(DOT_GRID_SPACING);
    let start_y = rect.top() + offset.y.rem_euclid(DOT_GRID_SPACING);
    let mut y = start_y;
    while y < rect.bottom() {
        let mut x = start_x;
        while x < rect.right() {
            painter.circle_filled(egui::pos2(x, y), DOT_RADIUS, pal.dot_grid);
            x += DOT_GRID_SPACING;
        }
        y += DOT_GRID_SPACING;
    }
}

pub fn node_card_frame(pal: Palette, is_selected: bool) -> Frame {
    let stroke = if is_selected {
        Stroke::new(2.0, pal.accent)
    } else {
        Stroke::new(1.0, pal.border_strong)
    };
    card_frame(pal).stroke(stroke).inner_margin(Margin::same(0))
}

pub fn section_label(ui: &mut Ui, pal: Palette, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .font(semibold(11.0))
            .color(pal.text_muted),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_grid_offset_wraps_into_spacing() {
        assert!((-5.0_f32).rem_euclid(DOT_GRID_SPACING) < DOT_GRID_SPACING);
        assert!((DOT_GRID_SPACING * 3.0 + 4.0).rem_euclid(DOT_GRID_SPACING) - 4.0 < 1e-3);
    }
}
