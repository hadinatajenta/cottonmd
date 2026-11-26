use eframe::egui::{self, Align, Layout, Margin, RichText, Ui};
use egui_phosphor::regular as icons;

use crate::file_tree::file_display_name;
use crate::shell::widgets::{accent_button, pick_vault_folder, status_dot, vertical_divider};
use crate::shell::MOD_KEY;
use crate::state::{AppState, CenterMode};
use crate::theme::{semibold, Palette};
use crate::ui_kit::{canvas_frame, icon_button, outlined_icon_button, underline_tabs};

const MODE_TABS: [(CenterMode, &str, &str); 6] = [
    (CenterMode::Edit, icons::NOTE_PENCIL, "Edit"),
    (CenterMode::Split, icons::COLUMNS, "Split"),
    (CenterMode::Preview, icons::EYE, "Preview"),
    (CenterMode::Graph, icons::GRAPH, "Graph"),
    (CenterMode::Bases, icons::TABLE, "Bases"),
    (CenterMode::Canvas, icons::SQUARES_FOUR, "Canvas"),
];

pub fn show(ctx: &egui::Context, state: &mut AppState, pal: Palette) {
    egui::TopBottomPanel::top("top_bar")
        .frame(canvas_frame(pal).inner_margin(Margin { left: 12, right: 12, top: 10, bottom: 0 }))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            draw_header_row(ui, state, pal);
            ui.add_space(8.0);
            underline_tabs(ui, pal, &mut state.center_mode, &MODE_TABS);
            ui.add(egui::Separator::default().spacing(0.0));
        });
}

fn draw_header_row(ui: &mut Ui, state: &mut AppState, pal: Palette) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        let toggle_tooltip = "Toggle sidebar";
        if outlined_icon_button(ui, pal, icons::LIST, toggle_tooltip).clicked() {
            state.show_left_sidebar = !state.show_left_sidebar;
        }
        vertical_divider(ui, pal);
        draw_breadcrumb(ui, state, pal);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            draw_actions(ui, state, pal);
        });
    });
}

fn draw_breadcrumb(ui: &mut Ui, state: &AppState, pal: Palette) {
    ui.spacing_mut().item_spacing.x = 6.0;
    ui.label(RichText::new(state.vault_name()).font(semibold(14.0)).color(pal.text_muted));
    let Some(active_path) = state.active_path.as_deref() else {
        return;
    };
    ui.label(RichText::new("/").size(14.0).color(pal.border_strong));
    ui.label(
        RichText::new(file_display_name(active_path))
            .font(semibold(14.0))
            .color(pal.text_primary),
    );
    if state.is_dirty() {
        status_dot(ui, pal.unsaved);
    }
}

fn draw_actions(ui: &mut Ui, state: &mut AppState, pal: Palette) {
    ui.spacing_mut().item_spacing.x = 4.0;
    if icon_button(ui, pal, icons::SIDEBAR_SIMPLE, "Toggle inspector").clicked() {
        state.show_right_sidebar = !state.show_right_sidebar;
    }
    draw_save_button(ui, state, pal);
    let (theme_icon, theme_tooltip) = if state.dark_mode {
        (icons::SUN, "Switch to light theme")
    } else {
        (icons::MOON, "Switch to dark theme")
    };
    if icon_button(ui, pal, theme_icon, theme_tooltip).clicked() {
        state.dark_mode = !state.dark_mode;
    }
    draw_vault_actions(ui, state, pal);
}

fn draw_vault_actions(ui: &mut Ui, state: &mut AppState, pal: Palette) {
    let has_vault = state.vault.is_some();
    let quick_switch_tooltip = format!("Quick switch ({MOD_KEY}+P)");
    let quick_switch = ui
        .add_enabled_ui(has_vault, |ui| {
            icon_button(ui, pal, icons::MAGNIFYING_GLASS, &quick_switch_tooltip)
        })
        .inner;
    if quick_switch.clicked() {
        state.open_quick_switcher();
    }
    let new_note_tooltip = format!("New note ({MOD_KEY}+N)");
    let new_note = ui
        .add_enabled_ui(has_vault, |ui| {
            icon_button(ui, pal, icons::NOTE_PENCIL, &new_note_tooltip)
        })
        .inner;
    if new_note.clicked() {
        state.open_new_note_dialog();
    }
    let open_vault_tooltip = format!("Open vault ({MOD_KEY}+O)");
    if icon_button(ui, pal, icons::FOLDER_SIMPLE_PLUS, &open_vault_tooltip).clicked() {
        pick_vault_folder(state);
    }
}

fn draw_save_button(ui: &mut Ui, state: &mut AppState, pal: Palette) {
    let tooltip = format!("Save ({MOD_KEY}+S)");
    let is_clicked = if state.is_dirty() {
        ui.add(accent_button(pal, &format!("{}  Save", icons::FLOPPY_DISK)))
            .on_hover_text(tooltip)
            .clicked()
    } else {
        ui.add_enabled_ui(false, |ui| icon_button(ui, pal, icons::FLOPPY_DISK, &tooltip))
            .inner
            .clicked()
    };
    if is_clicked {
        state.save_active_file();
    }
}
