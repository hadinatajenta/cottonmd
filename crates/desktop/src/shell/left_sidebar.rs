use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Ui};
use egui_phosphor::regular as icons;

use crate::shell::file_tree_view::{draw_file_row, draw_files_tab};
use crate::shell::widgets::{fixed_extent_card, muted_text, panel_frame, PanelSide};
use crate::shell::{SIDEBAR_DEFAULT_WIDTH, SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::state::{AppState, LeftTab};
use crate::theme::{Palette, RADIUS_CONTROL};
use crate::ui_kit::{search_field, tree_row, underline_tabs, TreeRow};

const LEFT_TABS: [(LeftTab, &str, &str); 3] = [
    (LeftTab::Files, "", "Files"),
    (LeftTab::Search, "", "Search"),
    (LeftTab::Tags, "", "Tags"),
];

pub fn show(ctx: &egui::Context, state: &mut AppState, pal: Palette) {
    if !state.show_left_sidebar {
        return;
    }
    let mut opened_path = None;
    egui::SidePanel::left("left_sidebar")
        .resizable(true)
        .show_separator_line(false)
        .default_width(SIDEBAR_DEFAULT_WIDTH)
        .min_width(SIDEBAR_MIN_WIDTH)
        .max_width(SIDEBAR_MAX_WIDTH)
        .frame(panel_frame(pal, PanelSide::Left))
        .show(ctx, |ui| {
            fixed_extent_card(ui, pal, |ui| {
                opened_path = draw_card_contents(ui, state, pal);
            });
        });
    if let Some(relative_path) = opened_path {
        state.open_file(&relative_path);
    }
}

fn draw_card_contents(ui: &mut Ui, state: &mut AppState, pal: Palette) -> Option<String> {
    underline_tabs(ui, pal, &mut state.left_tab, &LEFT_TABS);
    ui.add_space(10.0);
    if state.vault.is_none() {
        muted_text(ui, pal, "No vault open. Click Open Vault to choose a folder.");
        return None;
    }
    match state.left_tab {
        LeftTab::Files => draw_files_tab(ui, state, pal),
        LeftTab::Search => draw_search_tab(ui, state, pal),
        LeftTab::Tags => draw_tags_tab(ui, state, pal),
    }
}

fn draw_search_tab(ui: &mut Ui, state: &mut AppState, pal: Palette) -> Option<String> {
    let field_response = search_field(
        ui,
        pal,
        &mut state.search_query,
        "Search titles and content...",
        "",
    );
    if field_response.changed() {
        state.refresh_search();
    }
    ui.add_space(8.0);
    if state.search_query.trim().is_empty() {
        muted_text(ui, pal, "Type a keyword to search notes in this vault.");
        return None;
    }
    if state.search_results.is_empty() {
        muted_text(ui, pal, "No matching notes found.");
        return None;
    }
    let mut opened_path = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for hit in &state.search_results {
                if draw_search_hit(ui, pal, &hit.path, hit.line, &hit.snippet) {
                    opened_path = Some(hit.path.clone());
                }
            }
        });
    opened_path
}

fn draw_search_hit(ui: &mut Ui, pal: Palette, path: &str, line: usize, snippet: &str) -> bool {
    let header = if line == 0 { path.to_string() } else { format!("{path}:{line}") };
    let mut is_clicked = false;
    Frame::new()
        .fill(pal.bg_elevated)
        .corner_radius(CornerRadius::same(RADIUS_CONTROL))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let header_label = egui::Label::new(RichText::new(header).size(12.0).color(pal.accent))
                .sense(egui::Sense::click());
            is_clicked = ui.add(header_label).clicked();
            ui.label(RichText::new(snippet).size(11.5).color(pal.text_muted));
        });
    ui.add_space(4.0);
    is_clicked
}

fn draw_tags_tab(ui: &mut Ui, state: &mut AppState, pal: Palette) -> Option<String> {
    if state.graph.tags().is_empty() {
        muted_text(ui, pal, "No tags indexed in this vault yet.");
        return None;
    }
    let mut opened_path = None;
    let mut toggled_tag = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (tag, notes) in state.graph.tags() {
                let is_selected = state.selected_tag.as_deref() == Some(tag.as_str());
                let label = format!("#{tag}");
                let meta = notes.len().to_string();
                let row = TreeRow {
                    depth: 0,
                    icon: icons::TAG,
                    label: &label,
                    meta: &meta,
                    is_expanded: Some(is_selected),
                    is_selected,
                    is_accented: false,
                };
                if tree_row(ui, pal, &row).clicked() {
                    toggled_tag = Some(tag.clone());
                }
                if is_selected {
                    let clicked_path = draw_tagged_notes(ui, state, pal, notes);
                    opened_path = clicked_path.or(opened_path.take());
                }
            }
        });
    if let Some(tag) = toggled_tag {
        let is_already_selected = state.selected_tag.as_deref() == Some(tag.as_str());
        state.selected_tag = if is_already_selected { None } else { Some(tag) };
    }
    opened_path
}

fn draw_tagged_notes(
    ui: &mut Ui,
    state: &AppState,
    pal: Palette,
    notes: &std::collections::BTreeSet<String>,
) -> Option<String> {
    let mut opened_path = None;
    for note_path in notes {
        let label = crate::file_tree::file_display_name(note_path);
        if draw_file_row(ui, state, pal, note_path, &label, 1).clicked() {
            opened_path = Some(note_path.clone());
        }
    }
    opened_path
}
