use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Stroke, TextStyle, Ui};

use crate::bases_view::BasesViewState;
use crate::canvas_view::CanvasViewState;
use crate::graph_view::GraphViewState;
use crate::markdown_view::{render_markdown_preview, PreviewAction};
use crate::shell::widgets::pick_vault_folder;
use crate::shell::MOD_KEY;
use crate::state::{resolve_link_in_files, resolve_relative_in_files, AppState, CenterMode, LeftTab};
use crate::theme::{Palette, RADIUS_CONTROL};
use crate::ui_kit::{canvas_frame, card_frame};

const EDITOR_ROWS: usize = 36;
const CENTER_CARD_MARGIN: i8 = 14;

#[derive(Default)]
pub struct CenterViews {
    graph: GraphViewState,
    bases: BasesViewState,
    canvas: CanvasViewState,
}

#[derive(Default)]
struct CenterOutcome {
    opened_path: Option<String>,
    preview_action: Option<PreviewAction>,
}

pub fn show(ctx: &egui::Context, state: &mut AppState, views: &mut CenterViews, pal: Palette) {
    let mut outcome = CenterOutcome::default();
    egui::CentralPanel::default()
        .frame(canvas_frame(pal))
        .show(ctx, |ui| {
            card_frame(pal)
                .inner_margin(Margin::same(CENTER_CARD_MARGIN))
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    outcome = draw_mode(ui, state, views, pal);
                });
        });
    apply_outcome(state, outcome);
}

fn apply_outcome(state: &mut AppState, outcome: CenterOutcome) {
    match outcome.preview_action {
        Some(PreviewAction::OpenLink(target)) => state.open_wikilink(&target),
        Some(PreviewAction::OpenRelativeLink(href)) => state.open_relative_link(&href),
        Some(PreviewAction::FilterTag(tag)) => {
            state.show_left_sidebar = true;
            state.left_tab = LeftTab::Tags;
            state.selected_tag = Some(tag);
        }
        Some(PreviewAction::ToggleTask(line)) => state.toggle_task_line(line),
        None => {}
    }
    if let Some(relative_path) = outcome.opened_path {
        state.open_file(&relative_path);
    }
}

fn draw_mode(
    ui: &mut Ui,
    state: &mut AppState,
    views: &mut CenterViews,
    pal: Palette,
) -> CenterOutcome {
    let mut outcome = CenterOutcome::default();
    match state.center_mode {
        CenterMode::Graph => outcome.opened_path = views.graph.ui(ui, state, pal),
        CenterMode::Bases => outcome.opened_path = views.bases.ui(ui, state, pal),
        CenterMode::Canvas => {
            outcome.opened_path = views.canvas.ui(ui, state.active_canvas.as_ref(), pal);
        }
        CenterMode::Edit | CenterMode::Split | CenterMode::Preview => {
            outcome.preview_action = draw_note_mode(ui, state, pal);
        }
    }
    outcome
}

fn draw_note_mode(ui: &mut Ui, state: &mut AppState, pal: Palette) -> Option<PreviewAction> {
    if state.active_path.is_none() {
        draw_welcome(ui, state, pal);
        return None;
    }
    if let Some(error) = &state.canvas_error {
        ui.label(RichText::new(format!("Canvas JSON parse error: {error}")).color(pal.unresolved));
    }
    let files = &state.files;
    let active_path = state.active_path.as_deref();
    let link_exists = |target: &str| {
        resolve_link_in_files(files, target).is_some()
            || resolve_relative_in_files(files, active_path, target).is_some()
    };
    match state.center_mode {
        CenterMode::Edit => {
            egui::ScrollArea::vertical().show(ui, |ui| draw_editor(ui, &mut state.editor_buffer));
            None
        }
        CenterMode::Preview => {
            let content = state.editor_buffer.clone();
            egui::ScrollArea::vertical()
                .show(ui, |ui| render_markdown_preview(ui, &content, pal, &link_exists))
                .inner
        }
        _ => draw_split(ui, &mut state.editor_buffer, pal, &link_exists),
    }
}

fn draw_split(
    ui: &mut Ui,
    editor_buffer: &mut String,
    pal: Palette,
    link_exists: &dyn Fn(&str) -> bool,
) -> Option<PreviewAction> {
    let content = editor_buffer.clone();
    let mut preview_action = None;
    ui.columns(2, |columns| {
        egui::ScrollArea::vertical()
            .id_salt("split_editor_scroll")
            .show(&mut columns[0], |ui| draw_editor(ui, editor_buffer));
        preview_action = egui::ScrollArea::vertical()
            .id_salt("split_preview_scroll")
            .show(&mut columns[1], |ui| {
                Frame::new()
                    .fill(pal.bg_editor)
                    .stroke(Stroke::new(1.0, pal.border))
                    .corner_radius(CornerRadius::same(RADIUS_CONTROL))
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        render_markdown_preview(ui, &content, pal, link_exists)
                    })
                    .inner
            })
            .inner;
    });
    preview_action
}

fn draw_editor(ui: &mut Ui, editor_buffer: &mut String) {
    ui.add(
        egui::TextEdit::multiline(editor_buffer)
            .font(TextStyle::Monospace)
            .desired_width(f32::INFINITY)
            .desired_rows(EDITOR_ROWS)
            .frame(false),
    );
}

fn draw_welcome(ui: &mut Ui, state: &mut AppState, pal: Palette) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("CottonMD")
                    .size(22.0)
                    .strong()
                    .color(pal.text_primary),
            );
            ui.add_space(6.0);
            let message = if state.vault.is_some() {
                format!("Select a note from Files or press {MOD_KEY}+N to create one.")
            } else {
                format!("Open a local Markdown vault folder ({MOD_KEY}+O) to start writing.")
            };
            ui.label(RichText::new(message).color(pal.text_muted));
            ui.add_space(12.0);
            if ui.button("Open Vault Folder").clicked() {
                pick_vault_folder(state);
            }
        });
    });
}
