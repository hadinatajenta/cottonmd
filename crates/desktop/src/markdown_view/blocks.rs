use eframe::egui::{self, Align2, CornerRadius, RichText, Sense, Stroke, Ui, Vec2};

use super::inline::render_inline_line;
use super::{BlockOutcome, PreviewAction, RenderContext, BODY_LINE_HEIGHT, BODY_SIZE};
use crate::theme::{semibold, Palette};

const CHECKBOX_SIZE: f32 = 16.0;
const INDENT_PER_SPACE: f32 = 4.0;

pub fn heading_level(trimmed: &str) -> Option<(usize, &str)> {
    let hashes = trimmed.chars().take_while(|&ch| ch == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let text = trimmed[hashes..].strip_prefix(' ')?;
    Some((hashes, text.trim()))
}

pub fn heading_size(level: usize) -> f32 {
    match level {
        0 | 1 => 28.0,
        2 => 22.0,
        3 => 18.0,
        4 => 16.0,
        _ => 14.5,
    }
}

pub fn parse_task_line(trimmed: &str) -> Option<(bool, &str)> {
    let rest = trimmed
        .strip_prefix("- [")
        .or_else(|| trimmed.strip_prefix("* ["))
        .or_else(|| trimmed.strip_prefix("+ ["))?;
    let mut chars = rest.chars();
    let status = chars.next()?;
    if chars.next()? != ']' {
        return None;
    }
    Some((status == 'x' || status == 'X', chars.as_str().trim()))
}

fn strip_bullet_marker(trimmed: &str) -> Option<&str> {
    trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))
}

pub fn spacing_or_rule(ui: &mut Ui, line: &str, index: usize) -> Option<BlockOutcome> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        ui.add_space(8.0);
    } else if matches!(trimmed, "---" | "***" | "___") {
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);
    } else {
        return None;
    }
    Some(BlockOutcome::consumed(index + 1, None))
}

pub fn heading(
    ui: &mut Ui,
    context: &RenderContext,
    line: &str,
    index: usize,
) -> Option<BlockOutcome> {
    let (level, text) = heading_level(line.trim_start())?;
    let pal = context.pal;
    ui.add_space(if level <= 2 { 16.0 } else { 10.0 });
    ui.label(
        RichText::new(text)
            .font(semibold(heading_size(level)))
            .color(pal.text_primary),
    );
    if level == 1 {
        ui.add_space(4.0);
        ui.separator();
    }
    ui.add_space(4.0);
    Some(BlockOutcome::consumed(index + 1, None))
}

pub fn task_item(
    ui: &mut Ui,
    context: &RenderContext,
    line: &str,
    index: usize,
) -> Option<BlockOutcome> {
    let (is_checked, task_text) = parse_task_line(line.trim_start())?;
    let pal = context.pal;
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        if task_checkbox(ui, pal, is_checked) {
            action = Some(PreviewAction::ToggleTask(index + 1));
        }
        if is_checked {
            ui.label(
                RichText::new(task_text)
                    .size(BODY_SIZE)
                    .line_height(Some(BODY_LINE_HEIGHT))
                    .strikethrough()
                    .color(pal.text_muted),
            );
        } else if let Some(link_action) = render_inline_line(ui, context, task_text) {
            action = Some(link_action);
        }
    });
    Some(BlockOutcome::consumed(index + 1, action))
}

fn task_checkbox(ui: &mut Ui, pal: Palette, is_checked: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(CHECKBOX_SIZE), Sense::click());
    let corner = CornerRadius::same(5);
    if is_checked {
        ui.painter().rect_filled(rect, corner, pal.accent);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            egui_phosphor::regular::CHECK,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
    } else {
        let stroke_color = if response.hovered() {
            pal.accent
        } else {
            pal.border_strong
        };
        ui.painter().rect(
            rect.shrink(0.5),
            corner,
            pal.bg_editor,
            Stroke::new(1.5, stroke_color),
            egui::StrokeKind::Inside,
        );
    }
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

pub fn bullet_item(
    ui: &mut Ui,
    context: &RenderContext,
    line: &str,
    index: usize,
) -> Option<BlockOutcome> {
    let trimmed = line.trim_start();
    let item_text = strip_bullet_marker(trimmed)?;
    let indent = line.len().saturating_sub(trimmed.len()) as f32 * INDENT_PER_SPACE;
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        ui.add_space(indent + 4.0);
        ui.label(
            RichText::new("•")
                .size(BODY_SIZE)
                .line_height(Some(BODY_LINE_HEIGHT))
                .color(context.pal.text_muted),
        );
        action = render_inline_line(ui, context, item_text);
    });
    Some(BlockOutcome::consumed(index + 1, action))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_sizes_decrease_with_level() {
        let sizes: Vec<f32> = (1..=6).map(heading_size).collect();
        assert!(sizes.windows(2).all(|pair| pair[0] >= pair[1]));
        assert_eq!(heading_size(0), heading_size(1));
        assert_eq!(heading_size(99), heading_size(6));
    }

    #[test]
    fn heading_requires_space_after_hashes() {
        assert_eq!(heading_level("## Title "), Some((2, "Title")));
        assert_eq!(heading_level("#tag"), None);
        assert_eq!(heading_level("####### seven"), None);
        assert_eq!(heading_level("plain"), None);
    }

    #[test]
    fn task_line_parses_state_and_text() {
        assert_eq!(parse_task_line("- [x] done"), Some((true, "done")));
        assert_eq!(parse_task_line("* [ ] todo"), Some((false, "todo")));
        assert_eq!(parse_task_line("- [xx] nope"), None);
        assert_eq!(parse_task_line("- plain"), None);
        assert_eq!(parse_task_line("- ["), None);
    }

    #[test]
    fn bullet_marker_is_stripped() {
        assert_eq!(strip_bullet_marker("- item"), Some("item"));
        assert_eq!(strip_bullet_marker("+ item"), Some("item"));
        assert_eq!(strip_bullet_marker("-item"), None);
    }
}
