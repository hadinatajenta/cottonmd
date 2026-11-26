use cottonmd_vault::EntryKind;
use eframe::egui::{self, Ui};
use egui_phosphor::regular as icons;

use crate::file_tree::{
    file_display_name, filter_files, flatten_tree, kind_label, parent_folder, TreeEntryKind,
};
use crate::shell::widgets::muted_text;
use crate::state::AppState;
use crate::theme::Palette;
use crate::ui_kit::{icon_button, search_field, tree_row, TreeRow};

const ICON_BUTTON_WIDTH: f32 = 30.0;
const SEARCH_FIELD_HEIGHT: f32 = 32.0;

pub fn draw_files_tab(ui: &mut Ui, state: &mut AppState, pal: Palette) -> Option<String> {
    draw_filter_row(ui, state, pal);
    ui.add_space(8.0);
    let mut opened_path = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            opened_path = if state.file_filter.trim().is_empty() {
                draw_tree(ui, state, pal)
            } else {
                draw_filtered_files(ui, state, pal)
            };
        });
    opened_path
}

fn draw_filter_row(ui: &mut Ui, state: &mut AppState, pal: Palette) {
    ui.horizontal(|ui| {
        let field_width = ui.available_width() - ICON_BUTTON_WIDTH - ui.spacing().item_spacing.x;
        let field_size = egui::vec2(field_width.max(0.0), SEARCH_FIELD_HEIGHT);
        let field_response = ui
            .allocate_ui(field_size, |ui| {
                search_field(ui, pal, &mut state.file_filter, "Search...", "/")
            })
            .inner;
        if state.should_focus_file_filter {
            field_response.request_focus();
            state.should_focus_file_filter = false;
        }
        if icon_button(ui, pal, icons::ARROWS_CLOCKWISE, "Reload vault").clicked() {
            state.reindex_vault();
        }
    });
}

fn draw_tree(ui: &mut Ui, state: &mut AppState, pal: Palette) -> Option<String> {
    if state.folder_tree.file_count() == 0 {
        muted_text(ui, pal, "This vault has no files yet.");
        return None;
    }
    let mut opened_path = None;
    let mut toggled_folder = None;
    for entry in flatten_tree(&state.folder_tree, &state.collapsed_folders) {
        match &entry.kind {
            TreeEntryKind::Folder { path, is_expanded, file_count } => {
                let file_count_text = file_count.to_string();
                let row = TreeRow {
                    depth: entry.depth,
                    icon: if *is_expanded { icons::FOLDER_OPEN } else { icons::FOLDER },
                    label: &entry.name,
                    meta: &file_count_text,
                    is_expanded: Some(*is_expanded),
                    is_selected: false,
                    is_accented: false,
                };
                if tree_row(ui, pal, &row).clicked() {
                    toggled_folder = Some(path.clone());
                }
            }
            TreeEntryKind::File { relative_path } => {
                if draw_file_row(ui, state, pal, relative_path, &entry.name, entry.depth).clicked()
                {
                    opened_path = Some(relative_path.clone());
                }
            }
        }
    }
    if let Some(folder_path) = toggled_folder {
        state.toggle_folder(&folder_path);
    }
    opened_path
}

fn draw_filtered_files(ui: &mut Ui, state: &AppState, pal: Palette) -> Option<String> {
    let matches = filter_files(&state.files, &state.file_filter);
    if matches.is_empty() {
        muted_text(ui, pal, "No files match this filter.");
        return None;
    }
    let mut opened_path = None;
    for file in matches {
        let parent = parent_folder(&file.relative_path);
        let label = file_display_name(&file.relative_path);
        let row = file_tree_row(state, &file.relative_path, &label, parent, 0);
        if tree_row(ui, pal, &row).clicked() {
            opened_path = Some(file.relative_path.clone());
        }
    }
    opened_path
}

pub fn draw_file_row(
    ui: &mut Ui,
    state: &AppState,
    pal: Palette,
    relative_path: &str,
    label: &str,
    depth: usize,
) -> egui::Response {
    let meta = state
        .file_map
        .get(relative_path)
        .map_or("", |file| kind_label(file.kind));
    let row = file_tree_row(state, relative_path, label, meta, depth);
    tree_row(ui, pal, &row)
}

fn file_tree_row<'a>(
    state: &AppState,
    relative_path: &str,
    label: &'a str,
    meta: &'a str,
    depth: usize,
) -> TreeRow<'a> {
    let kind = state
        .file_map
        .get(relative_path)
        .map_or(EntryKind::Markdown, |file| file.kind);
    TreeRow {
        depth,
        icon: file_icon(kind),
        label,
        meta,
        is_expanded: None,
        is_selected: state.active_path.as_deref() == Some(relative_path),
        is_accented: matches!(kind, EntryKind::Canvas | EntryKind::Base),
    }
}

fn file_icon(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Markdown => icons::FILE_TEXT,
        EntryKind::Canvas => icons::SQUARES_FOUR,
        EntryKind::Base => icons::TABLE,
        EntryKind::Pdf => icons::FILE_PDF,
        EntryKind::Attachment => icons::PAPERCLIP,
    }
}
