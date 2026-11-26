use cottonmd_vault::VaultFile;
use eframe::egui::{self, Align2, Key, Margin, Modifiers, RichText, Ui};
use egui_phosphor::regular as icons;

use crate::file_tree::file_display_name;
use crate::shell::widgets::{accent_button, muted_text};
use crate::state::{move_selection, AppState};
use crate::theme::Palette;
use crate::ui_kit::{card_frame, search_field, section_label, tree_row, TreeRow};

const QUICK_SWITCHER_WIDTH: f32 = 520.0;
const QUICK_SWITCHER_MAX_LIST_HEIGHT: f32 = 320.0;
const NEW_NOTE_WIDTH: f32 = 400.0;

pub fn show(ctx: &egui::Context, state: &mut AppState, pal: Palette) {
    draw_quick_switcher(ctx, state, pal);
    draw_new_note_dialog(ctx, state, pal);
}

fn modal_frame(pal: Palette) -> egui::Frame {
    card_frame(pal).inner_margin(Margin::same(14))
}

fn consume_navigation_step(ctx: &egui::Context) -> isize {
    ctx.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
            1
        } else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
            -1
        } else {
            0
        }
    })
}

fn draw_quick_switcher(ctx: &egui::Context, state: &mut AppState, pal: Palette) {
    if !state.show_quick_switcher {
        return;
    }
    let matches: Vec<VaultFile> = state.quick_switcher_matches().into_iter().cloned().collect();
    let navigation_step = consume_navigation_step(ctx);
    state.quick_switcher_selected =
        move_selection(state.quick_switcher_selected, navigation_step, matches.len());
    let mut chosen_path = None;
    egui::Window::new("Quick Switcher")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_TOP, [0.0, 90.0])
        .frame(modal_frame(pal))
        .show(ctx, |ui| {
            ui.set_width(QUICK_SWITCHER_WIDTH);
            chosen_path = draw_quick_switcher_body(ui, state, pal, &matches, navigation_step != 0);
        });
    if let Some(relative_path) = chosen_path {
        state.open_file(&relative_path);
        state.show_quick_switcher = false;
    }
}

fn draw_quick_switcher_body(
    ui: &mut Ui,
    state: &mut AppState,
    pal: Palette,
    matches: &[VaultFile],
    should_scroll_to_selection: bool,
) -> Option<String> {
    let field_response = search_field(
        ui,
        pal,
        &mut state.quick_switcher_query,
        "Type a note name or path...",
        "",
    );
    if state.quick_switcher_focus_needed {
        field_response.request_focus();
        state.quick_switcher_focus_needed = false;
    }
    if field_response.changed() {
        state.quick_switcher_selected = 0;
    }
    ui.add_space(10.0);
    section_label(ui, pal, "Notes");
    ui.add_space(4.0);
    let mut chosen_path = draw_match_list(ui, state, pal, matches, should_scroll_to_selection);
    if ui.input(|input| input.key_pressed(Key::Enter)) {
        chosen_path = matches
            .get(state.quick_switcher_selected)
            .map(|file| file.relative_path.clone());
    }
    ui.add_space(8.0);
    muted_text(ui, pal, "Up/Down to navigate    Enter to open    Esc to close");
    chosen_path
}

fn draw_match_list(
    ui: &mut Ui,
    state: &AppState,
    pal: Palette,
    matches: &[VaultFile],
    should_scroll_to_selection: bool,
) -> Option<String> {
    if matches.is_empty() {
        muted_text(ui, pal, "No matching notes.");
        return None;
    }
    let mut chosen_path = None;
    egui::ScrollArea::vertical()
        .max_height(QUICK_SWITCHER_MAX_LIST_HEIGHT)
        .show(ui, |ui| {
            for (index, file) in matches.iter().enumerate() {
                let is_selected = index == state.quick_switcher_selected;
                let row = TreeRow {
                    depth: 0,
                    icon: icons::FILE_TEXT,
                    label: &file.stem,
                    meta: &file.relative_path,
                    is_expanded: None,
                    is_selected,
                    is_accented: false,
                };
                let response = tree_row(ui, pal, &row);
                if is_selected && should_scroll_to_selection {
                    response.scroll_to_me(None);
                }
                if response.clicked() {
                    chosen_path = Some(file.relative_path.clone());
                }
            }
        });
    chosen_path
}

pub fn initial_note_content(note_name: &str) -> String {
    format!("# {}\n\n", file_display_name(note_name))
}

fn draw_new_note_dialog(ctx: &egui::Context, state: &mut AppState, pal: Palette) {
    if !state.show_new_note_dialog {
        return;
    }
    let mut should_create = false;
    let mut should_close = false;
    egui::Window::new("Create New Note")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(modal_frame(pal))
        .show(ctx, |ui| {
            ui.set_width(NEW_NOTE_WIDTH);
            section_label(ui, pal, "New note");
            ui.add_space(6.0);
            muted_text(ui, pal, "Note path relative to vault (e.g. Notes/Ideas.md)");
            ui.add_space(6.0);
            let field_response = ui.add(
                egui::TextEdit::singleline(&mut state.new_note_name)
                    .hint_text(RichText::new("Untitled.md").color(pal.text_muted))
                    .desired_width(ui.available_width()),
            );
            should_create = field_response.lost_focus()
                && ui.input(|input| input.key_pressed(Key::Enter));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                should_create |= ui.add(accent_button(pal, "Create")).clicked();
                should_close = ui.button("Cancel").clicked();
            });
        });
    if should_create {
        let note_name = state.new_note_name.clone();
        state.create_and_open_note(&note_name, &initial_note_content(&note_name));
        state.show_new_note_dialog = false;
    } else if should_close {
        state.show_new_note_dialog = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_content_uses_file_name_without_extension_as_title() {
        assert_eq!(initial_note_content("Notes/Ideas.md"), "# Ideas\n\n");
        assert_eq!(initial_note_content("Plain"), "# Plain\n\n");
    }
}
