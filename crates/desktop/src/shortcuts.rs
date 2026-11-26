use eframe::egui::{self, Key, Modifiers};

use crate::state::{AppState, CenterMode, LeftTab};

#[must_use]
pub fn handle_shortcuts(ctx: &egui::Context, state: &mut AppState) -> bool {
    let is_typing = ctx.wants_keyboard_input();
    ctx.input_mut(|input| {
        if input.consume_key(Modifiers::COMMAND, Key::S) {
            state.save_active_file();
        }
        if input.consume_key(Modifiers::COMMAND, Key::P)
            || input.consume_key(Modifiers::COMMAND, Key::K)
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
        input.consume_key(Modifiers::COMMAND, Key::O)
    })
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

    fn press_command_key(state: &mut AppState, key: Key) -> bool {
        let ctx = egui::Context::default();
        let key_event = egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        };
        let raw_input = egui::RawInput {
            modifiers: Modifiers::COMMAND,
            events: vec![key_event],
            ..Default::default()
        };
        let mut should_pick_vault_folder = false;
        let _ = ctx.run(raw_input, |ctx| {
            should_pick_vault_folder = handle_shortcuts(ctx, state);
        });
        should_pick_vault_folder
    }

    #[test]
    fn command_o_requests_vault_picker_without_opening_quick_switcher() {
        let mut state = AppState::empty();
        assert!(press_command_key(&mut state, Key::O));
        assert!(!state.show_quick_switcher);
    }

    #[test]
    fn command_p_and_command_k_toggle_quick_switcher_without_vault_picker() {
        for key in [Key::P, Key::K] {
            let mut state = AppState::empty();
            assert!(!press_command_key(&mut state, key));
            assert!(state.show_quick_switcher);
        }
    }

    #[test]
    fn editor_mode_cycles_through_edit_split_preview() {
        assert_eq!(next_editor_mode(CenterMode::Edit), CenterMode::Split);
        assert_eq!(next_editor_mode(CenterMode::Split), CenterMode::Preview);
        assert_eq!(next_editor_mode(CenterMode::Preview), CenterMode::Edit);
        assert_eq!(next_editor_mode(CenterMode::Graph), CenterMode::Edit);
    }
}
