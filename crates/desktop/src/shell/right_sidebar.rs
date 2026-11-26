use cottonmd_markdown::ParsedNote;
use eframe::egui::{self, RichText, Ui};

use crate::file_tree::file_display_name;
use crate::shell::file_tree_view::draw_file_row;
use crate::shell::widgets::{fixed_extent_card, muted_text, panel_frame, section_header, PanelSide};
use crate::shell::{SIDEBAR_DEFAULT_WIDTH, SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::state::{AppState, RightTab};
use crate::theme::Palette;
use crate::ui_kit::underline_tabs;

const HEADING_INDENT: f32 = 12.0;

const RIGHT_TABS: [(RightTab, &str, &str); 3] = [
    (RightTab::Backlinks, "", "Links"),
    (RightTab::Outline, "", "Outline"),
    (RightTab::Properties, "", "Props"),
];

#[derive(Default)]
struct InspectorActions {
    open_path: Option<String>,
    open_wikilink: Option<String>,
    toggled_task_line: Option<usize>,
}

pub fn show(ctx: &egui::Context, state: &mut AppState, pal: Palette) {
    if !state.show_right_sidebar {
        return;
    }
    let mut actions = InspectorActions::default();
    egui::SidePanel::right("right_sidebar")
        .resizable(true)
        .show_separator_line(false)
        .default_width(SIDEBAR_DEFAULT_WIDTH)
        .min_width(SIDEBAR_MIN_WIDTH)
        .max_width(SIDEBAR_MAX_WIDTH)
        .frame(panel_frame(pal, PanelSide::Right))
        .show(ctx, |ui| {
            fixed_extent_card(ui, pal, |ui| {
                actions = draw_card_contents(ui, state, pal);
            });
        });
    if let Some(relative_path) = actions.open_path {
        state.open_file(&relative_path);
    }
    if let Some(target) = actions.open_wikilink {
        state.open_wikilink(&target);
    }
    if let Some(line) = actions.toggled_task_line {
        state.toggle_task_line(line);
    }
}

fn draw_card_contents(ui: &mut Ui, state: &mut AppState, pal: Palette) -> InspectorActions {
    underline_tabs(ui, pal, &mut state.right_tab, &RIGHT_TABS);
    ui.add_space(10.0);
    let mut actions = InspectorActions::default();
    let Some(active_path) = state.active_path.as_deref() else {
        muted_text(ui, pal, "Select a note to inspect links, headings, and properties.");
        return actions;
    };
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| match state.right_tab {
            RightTab::Backlinks => draw_links_tab(ui, state, pal, active_path, &mut actions),
            RightTab::Outline => draw_outline_tab(ui, pal, &state.active_parsed().unwrap_or_default()),
            RightTab::Properties => {
                let parsed = state.active_parsed().unwrap_or_default();
                actions.toggled_task_line = draw_properties_tab(ui, pal, &parsed);
            }
        });
    actions
}

fn draw_links_tab(
    ui: &mut Ui,
    state: &AppState,
    pal: Palette,
    active_path: &str,
    actions: &mut InspectorActions,
) {
    let backlinks = state.graph.backlinks(active_path);
    section_header(ui, pal, "Backlinks", Some(backlinks.len()));
    if backlinks.is_empty() {
        muted_text(ui, pal, "No notes link to this note yet.");
    }
    actions.open_path = draw_note_rows(ui, state, pal, &backlinks);

    ui.add_space(14.0);
    let outgoing = state.graph.outgoing_links(active_path);
    section_header(ui, pal, "Outgoing links", Some(outgoing.len()));
    if outgoing.is_empty() {
        muted_text(ui, pal, "No resolved outgoing links.");
    }
    actions.open_path = draw_note_rows(ui, state, pal, &outgoing).or(actions.open_path.take());

    if let Some(missing) = state.graph.unresolved_links().get(active_path) {
        if !missing.is_empty() {
            ui.add_space(14.0);
            section_header(ui, pal, "Unresolved", Some(missing.len()));
            actions.open_wikilink = draw_unresolved_links(ui, pal, missing.iter());
        }
    }
}

fn draw_note_rows(ui: &mut Ui, state: &AppState, pal: Palette, paths: &[&str]) -> Option<String> {
    let mut opened_path = None;
    for path in paths {
        let label = file_display_name(path);
        if draw_file_row(ui, state, pal, path, &label, 0).clicked() {
            opened_path = Some((*path).to_string());
        }
    }
    opened_path
}

fn draw_unresolved_links<'a>(
    ui: &mut Ui,
    pal: Palette,
    targets: impl Iterator<Item = &'a String>,
) -> Option<String> {
    let mut clicked_target = None;
    for target in targets {
        let label = egui::Label::new(
            RichText::new(format!("[[{target}]]"))
                .color(pal.unresolved)
                .underline(),
        )
        .sense(egui::Sense::click());
        if ui.add(label).on_hover_text("Click to create note").clicked() {
            clicked_target = Some(target.clone());
        }
    }
    clicked_target
}

fn draw_outline_tab(ui: &mut Ui, pal: Palette, parsed: &ParsedNote) {
    section_header(ui, pal, "Headings", Some(parsed.headings.len()));
    if parsed.headings.is_empty() {
        muted_text(ui, pal, "No headings in this note.");
        return;
    }
    for heading in &parsed.headings {
        let indent = f32::from(heading.level.saturating_sub(1)) * HEADING_INDENT;
        ui.horizontal(|ui| {
            ui.add_space(indent);
            ui.label(RichText::new(&heading.text).size(12.5).color(pal.text_primary));
        });
    }
}

fn draw_properties_tab(ui: &mut Ui, pal: Palette, parsed: &ParsedNote) -> Option<usize> {
    section_header(ui, pal, "Properties", Some(parsed.properties.len()));
    if parsed.properties.is_empty() {
        muted_text(ui, pal, "No YAML frontmatter properties.");
    }
    for (key, value) in &parsed.properties {
        ui.label(RichText::new(key).size(11.0).color(pal.text_muted));
        ui.label(RichText::new(value).size(12.0).color(pal.text_primary));
        ui.add_space(4.0);
    }

    ui.add_space(14.0);
    section_header(ui, pal, "Tasks", Some(parsed.tasks.len()));
    if parsed.tasks.is_empty() {
        muted_text(ui, pal, "No tasks in this note.");
    }
    draw_tasks(ui, pal, parsed)
}

fn draw_tasks(ui: &mut Ui, pal: Palette, parsed: &ParsedNote) -> Option<usize> {
    let mut toggled_line = None;
    for task in &parsed.tasks {
        let mut is_done = task.status.eq_ignore_ascii_case(&'x');
        ui.horizontal_wrapped(|ui| {
            if ui.checkbox(&mut is_done, "").changed() {
                toggled_line = Some(task.line);
            }
            let color = if is_done { pal.done } else { pal.text_primary };
            ui.label(RichText::new(&task.text).size(12.0).color(color));
        });
    }
    toggled_line
}
