use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Sense, Ui, UiBuilder, Vec2,
};

use crate::state::AppState;
use crate::theme::{Palette, RADIUS_CONTROL};
use crate::ui_kit::{canvas_frame, card_frame, pill_badge, section_label};

#[derive(Clone, Copy)]
pub enum PanelSide {
    Left,
    Right,
}

pub fn panel_frame(pal: Palette, side: PanelSide) -> Frame {
    let margin = match side {
        PanelSide::Left => Margin { left: 8, right: 0, top: 4, bottom: 8 },
        PanelSide::Right => Margin { left: 0, right: 8, top: 4, bottom: 8 },
    };
    canvas_frame(pal).inner_margin(margin)
}

pub fn fixed_extent_card(ui: &mut Ui, pal: Palette, add_contents: impl FnOnce(&mut Ui)) {
    let outer_rect = ui.available_rect_before_wrap();
    let mut card_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(outer_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    card_frame(pal).show(&mut card_ui, |ui| {
        ui.set_min_size(ui.available_size());
        ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
        add_contents(ui);
    });
    ui.allocate_rect(outer_rect, Sense::hover());
}

pub fn muted_text(ui: &mut Ui, pal: Palette, text: &str) {
    ui.label(RichText::new(text).size(12.0).color(pal.text_muted));
}

pub fn section_header(ui: &mut Ui, pal: Palette, title: &str, count: Option<usize>) {
    ui.horizontal(|ui| {
        section_label(ui, pal, title);
        if let Some(count) = count {
            pill_badge(ui, pal, &count.to_string());
        }
    });
}

pub fn vertical_divider(ui: &mut Ui, pal: Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 22.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, pal.border_strong);
}

pub fn status_dot(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.5, color);
}

pub fn accent_button(pal: Palette, label: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(label.to_owned()).color(Color32::WHITE))
        .fill(pal.accent)
        .corner_radius(CornerRadius::same(RADIUS_CONTROL))
}

pub fn pick_vault_folder(state: &mut AppState) {
    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
        state.open_vault(&folder);
    }
}
