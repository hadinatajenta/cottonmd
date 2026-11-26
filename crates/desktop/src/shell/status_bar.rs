use eframe::egui::{self, Align, Layout, Margin, RichText, Ui};

use crate::shell::widgets::vertical_divider;
use crate::state::AppState;
use crate::theme::Palette;
use crate::ui_kit::{canvas_frame, card_frame, pill_badge};

pub fn show(ctx: &egui::Context, state: &AppState, pal: Palette) {
    egui::TopBottomPanel::bottom("status_bar")
        .frame(canvas_frame(pal).inner_margin(Margin { left: 8, right: 8, top: 0, bottom: 8 }))
        .show_separator_line(false)
        .show(ctx, |ui| {
            card_frame(pal)
                .inner_margin(Margin::symmetric(12, 5))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| draw_status_row(ui, state, pal));
                });
        });
}

fn draw_status_row(ui: &mut Ui, state: &AppState, pal: Palette) {
    ui.spacing_mut().item_spacing.x = 10.0;
    draw_muted(ui, pal, &state.vault_name());
    vertical_divider(ui, pal);
    draw_muted(ui, pal, &format!("{} notes", state.graph.note_count()));
    if let Some(message) = &state.status_message {
        vertical_divider(ui, pal);
        draw_muted(ui, pal, message);
    }
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if let Some(active_path) = state.active_path.as_deref() {
            draw_note_statistics(ui, state, pal, active_path);
        }
    });
}

fn draw_note_statistics(ui: &mut Ui, state: &AppState, pal: Palette, active_path: &str) {
    let (word_count, char_count) = state.active_note_word_count();
    draw_muted(ui, pal, &format!("{word_count} words  |  {char_count} chars"));
    pill_badge(
        ui,
        pal,
        &format!("{} backlinks", state.graph.backlinks(active_path).len()),
    );
}

fn draw_muted(ui: &mut Ui, pal: Palette, text: &str) {
    ui.label(RichText::new(text).size(11.5).color(pal.text_muted));
}
