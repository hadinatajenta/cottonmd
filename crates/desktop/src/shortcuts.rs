use eframe::egui::{self, Key, Modifiers};

use crate::state::{AppState, CenterMode, LeftTab};

pub fn handle_shortcuts(ctx: &egui::Context, state: &mut AppState) {
    let is_typing = ctx.wants_keyboard_input();
    ctx.input_mut(|input| {
        if input.consume_key(Modifiers::COMMAND, Key::S) {
            state.save_active_file();
        }
        if input.consume_key(Modifiers::COMMAND, Key::K)
            || input.consume_key(Modifiers::COMMAND, Key::O)
        {
            toggle_quick_switcher(state);
        }
        if input.consume_key(Modifiers::COMMAND, Key::N) {
            state.open_new_note_dialog();
        }
        if input.consume_key(Modifiers::COMMAND, Key::E) {
            state.center_mode = next_editor_mode(state.center_mode);
        }
        if input.consume_key(Modifiers::COMMAND, Key::G) {
            state.center_mode = CenterMode::Graph;
        }
        if input.consume_key(Modifiers::COMMAND, Key::B) {
            state.center_mode = CenterMode::Bases;
        }
        if input.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::F) {
            state.show_left_sidebar = true;
            state.left_tab = LeftTab::Search;
        }
        if !is_typing && input.consume_key(Modifiers::NONE, Key::Slash) {
            state.focus_file_filter();
        }
        if input.consume_key(Modifiers::NONE, Key::Escape) {
            state.show_quick_switcher = false;
            state.show_new_note_dialog = false;
        }
    });
}

fn toggle_quick_switcher(state: &mut AppState) {
    if state.show_quick_switcher {
        state.show_quick_switcher = false;
    } else {
        state.open_quick_switcher();
    }
}

fn next_editor_mode(current: CenterMode) -> CenterMode {
    match current {
        CenterMode::Edit => CenterMode::Split,
        CenterMode::Split => CenterMode::Preview,
        _ => CenterMode::Edit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_mode_cycles_through_edit_split_preview() {
        assert_eq!(next_editor_mode(CenterMode::Edit), CenterMode::Split);
        assert_eq!(next_editor_mode(CenterMode::Split), CenterMode::Preview);
        assert_eq!(next_editor_mode(CenterMode::Preview), CenterMode::Edit);
        assert_eq!(next_editor_mode(CenterMode::Graph), CenterMode::Edit);
    }
}
