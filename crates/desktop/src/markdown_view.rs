use cottonmd_markdown::parse_note;
use eframe::egui::{self, Align, Layout, RichText, Ui, Vec2};

mod blocks;
mod containers;
mod emphasis;
mod inline;
mod table;

use crate::theme::Palette;
use crate::ui_kit::{card_frame, empty_state, section_label};

const MAX_CONTENT_WIDTH: f32 = 760.0;
const BODY_SIZE: f32 = 14.5;
const BODY_LINE_HEIGHT: f32 = 23.0;

pub enum PreviewAction {
    OpenLink(String),
    FilterTag(String),
    ToggleTask(usize),
}

struct RenderContext<'a> {
    pal: Palette,
    resolve_exists: &'a dyn Fn(&str) -> bool,
}

struct BlockOutcome {
    next_line: usize,
    action: Option<PreviewAction>,
}

impl BlockOutcome {
    fn consumed(next_line: usize, action: Option<PreviewAction>) -> Self {
        Self { next_line, action }
    }
}

pub fn render_markdown_preview(
    ui: &mut Ui,
    markdown: &str,
    pal: Palette,
    resolve_exists: &dyn Fn(&str) -> bool,
) -> Option<PreviewAction> {
    let context = RenderContext {
        pal,
        resolve_exists,
    };
    let content_width = readable_width(ui.available_width());
    let mut action = None;
    ui.vertical_centered(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(content_width, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(content_width);
                action = render_document(ui, markdown, &context);
            },
        );
    });
    action
}

fn readable_width(available_width: f32) -> f32 {
    if available_width.is_finite() {
        available_width.clamp(0.0, MAX_CONTENT_WIDTH)
    } else {
        MAX_CONTENT_WIDTH
    }
}

fn render_document(ui: &mut Ui, markdown: &str, context: &RenderContext) -> Option<PreviewAction> {
    let pal = context.pal;
    if markdown.trim().is_empty() {
        empty_state(
            ui,
            pal,
            egui_phosphor::regular::FILE_TEXT,
            "This note is empty.",
        );
        return None;
    }
    show_properties(ui, pal, markdown);
    let lines: Vec<&str> = markdown.lines().collect();
    let mut index = frontmatter_end(&lines);
    let mut action = None;
    while index < lines.len() {
        let outcome = render_block(ui, context, &lines, index);
        index = outcome.next_line.max(index + 1);
        if outcome.action.is_some() {
            action = outcome.action;
        }
    }
    action
}

fn show_properties(ui: &mut Ui, pal: Palette, markdown: &str) {
    let parsed = parse_note(markdown);
    if parsed.properties.is_empty() {
        return;
    }
    card_frame(pal).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        section_label(ui, pal, "Properties");
        ui.add_space(6.0);
        egui::Grid::new("preview_props_grid")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .show(ui, |ui| {
                for (key, value) in &parsed.properties {
                    ui.label(
                        RichText::new(key)
                            .monospace()
                            .size(12.0)
                            .color(pal.text_muted),
                    );
                    ui.label(RichText::new(value).size(13.0).color(pal.text_primary));
                    ui.end_row();
                }
            });
    });
    ui.add_space(14.0);
}

fn frontmatter_end(lines: &[&str]) -> usize {
    if lines.first().map(|line| line.trim()) != Some("---") {
        return 0;
    }
    lines
        .iter()
        .skip(1)
        .position(|line| matches!(line.trim(), "---" | "..."))
        .map_or(lines.len(), |closing| closing + 2)
}

fn render_block(
    ui: &mut Ui,
    context: &RenderContext,
    lines: &[&str],
    index: usize,
) -> BlockOutcome {
    if let Some(outcome) = containers::fenced_code(ui, context, lines, index) {
        return outcome;
    }
    if let Some(outcome) = blocks::spacing_or_rule(ui, lines[index], index) {
        return outcome;
    }
    if let Some(outcome) = blocks::heading(ui, context, lines[index], index) {
        return outcome;
    }
    if let Some(outcome) = containers::callout(ui, context, lines, index) {
        return outcome;
    }
    if let Some(outcome) = containers::quote(ui, context, lines, index) {
        return outcome;
    }
    if let Some(outcome) = table::table_block(ui, context, lines, index) {
        return outcome;
    }
    if let Some(outcome) = blocks::task_item(ui, context, lines[index], index) {
        return outcome;
    }
    if let Some(outcome) = blocks::bullet_item(ui, context, lines[index], index) {
        return outcome;
    }
    let action = inline::render_inline_line(ui, context, lines[index]);
    BlockOutcome::consumed(index + 1, action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readable_width_is_capped_and_safe() {
        assert_eq!(readable_width(2000.0), MAX_CONTENT_WIDTH);
        assert_eq!(readable_width(300.0), 300.0);
        assert_eq!(readable_width(-5.0), 0.0);
        assert_eq!(readable_width(f32::INFINITY), MAX_CONTENT_WIDTH);
    }

    #[test]
    fn frontmatter_end_skips_closed_block_only() {
        assert_eq!(frontmatter_end(&["---", "a: 1", "---", "body"]), 3);
        assert_eq!(frontmatter_end(&["body", "---"]), 0);
        assert_eq!(frontmatter_end(&["---", "a: 1"]), 2);
        assert_eq!(frontmatter_end(&[]), 0);
    }
}
