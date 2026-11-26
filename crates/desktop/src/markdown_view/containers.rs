use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Rect, RichText, Stroke, Ui, Vec2};

use super::inline::render_inline_line;
use super::{BlockOutcome, PreviewAction, RenderContext};
use crate::theme::{semibold, Palette};
use crate::ui_kit::pill_badge;

const ACCENT_BAR_WIDTH: f32 = 3.0;
const CODE_FONT_SIZE: f32 = 12.5;

pub fn fenced_code(
    ui: &mut Ui,
    context: &RenderContext,
    lines: &[&str],
    index: usize,
) -> Option<BlockOutcome> {
    let trimmed = lines[index].trim_start();
    let fence = if trimmed.starts_with("```") {
        "```"
    } else if trimmed.starts_with("~~~") {
        "~~~"
    } else {
        return None;
    };
    let language = trimmed.trim_start_matches(fence).trim();
    let body_start = index + 1;
    let body_length = lines[body_start..]
        .iter()
        .position(|line| line.trim_start().starts_with(fence))
        .unwrap_or(lines.len() - body_start);
    let code = lines[body_start..body_start + body_length].join("\n");
    show_code_block(ui, context.pal, language, &code, index);
    let next_line = (body_start + body_length + 1).min(lines.len());
    Some(BlockOutcome::consumed(next_line, None))
}

fn show_code_block(ui: &mut Ui, pal: Palette, language: &str, code: &str, index: usize) {
    Frame::new()
        .fill(pal.bg_code)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            if !language.is_empty() {
                pill_badge(ui, pal, language);
                ui.add_space(6.0);
            }
            egui::ScrollArea::horizontal()
                .id_salt(("code_block", index))
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(code)
                                .monospace()
                                .size(CODE_FONT_SIZE)
                                .color(pal.text_primary),
                        )
                        .extend(),
                    );
                });
        });
    ui.add_space(10.0);
}

fn is_callout_header(trimmed: &str) -> bool {
    trimmed.starts_with("> [!")
}

fn collect_quote_body<'a>(
    lines: &[&'a str],
    start: usize,
    should_stop_at_callout: bool,
) -> (Vec<&'a str>, usize) {
    let mut body = Vec::new();
    let mut cursor = start;
    while cursor < lines.len() {
        let trimmed = lines[cursor].trim_start();
        if !trimmed.starts_with('>') || (should_stop_at_callout && is_callout_header(trimmed)) {
            break;
        }
        body.push(trimmed.trim_start_matches('>').trim_start());
        cursor += 1;
    }
    (body, cursor)
}

pub fn callout_title(header: &str) -> String {
    header
        .strip_prefix("[!")
        .and_then(|rest| rest.split_once(']'))
        .map(|(kind, rest)| {
            if rest.trim().is_empty() {
                capitalize(kind)
            } else {
                format!("{}: {}", capitalize(kind), rest.trim())
            }
        })
        .unwrap_or_else(|| header.to_string())
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn callout(
    ui: &mut Ui,
    context: &RenderContext,
    lines: &[&str],
    index: usize,
) -> Option<BlockOutcome> {
    let trimmed = lines[index].trim_start();
    if !is_callout_header(trimmed) {
        return None;
    }
    let pal = context.pal;
    let title = callout_title(trimmed.trim_start_matches('>').trim());
    let (body, next_line) = collect_quote_body(lines, index + 1, false);
    let action = bar_container(ui, pal.accent_soft, pal.accent, |ui| {
        ui.label(
            RichText::new(format!("{}  {title}", egui_phosphor::regular::INFO))
                .font(semibold(13.5))
                .color(pal.accent),
        );
        ui.add_space(4.0);
        render_body_lines(ui, context, &body)
    });
    ui.add_space(10.0);
    Some(BlockOutcome::consumed(next_line, action))
}

pub fn quote(
    ui: &mut Ui,
    context: &RenderContext,
    lines: &[&str],
    index: usize,
) -> Option<BlockOutcome> {
    if !lines[index].trim_start().starts_with('>') {
        return None;
    }
    let pal = context.pal;
    let (body, next_line) = collect_quote_body(lines, index, true);
    let action = bar_container(ui, Color32::TRANSPARENT, pal.border_strong, |ui| {
        render_body_lines(ui, context, &body)
    });
    ui.add_space(10.0);
    Some(BlockOutcome::consumed(next_line, action))
}

fn render_body_lines(ui: &mut Ui, context: &RenderContext, body: &[&str]) -> Option<PreviewAction> {
    let mut action = None;
    for body_line in body {
        if let Some(line_action) = render_inline_line(ui, context, body_line) {
            action = Some(line_action);
        }
    }
    action
}

fn bar_container(
    ui: &mut Ui,
    fill: Color32,
    bar_color: Color32,
    add_contents: impl FnOnce(&mut Ui) -> Option<PreviewAction>,
) -> Option<PreviewAction> {
    let frame_response = Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin {
            left: 16,
            right: 12,
            top: 8,
            bottom: 8,
        })
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui)
        });
    let frame_rect = frame_response.response.rect;
    let bar_rect = Rect::from_min_size(
        frame_rect.min + Vec2::new(0.0, 4.0),
        Vec2::new(ACCENT_BAR_WIDTH, (frame_rect.height() - 8.0).max(0.0)),
    );
    ui.painter()
        .rect_filled(bar_rect, CornerRadius::same(2), bar_color);
    frame_response.inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callout_title_combines_kind_and_custom_title() {
        assert_eq!(callout_title("[!note]"), "Note");
        assert_eq!(
            callout_title("[!warning] Careful now"),
            "Warning: Careful now"
        );
        assert_eq!(callout_title("[broken"), "[broken");
    }

    #[test]
    fn quote_body_stops_before_next_callout_when_requested() {
        let lines = ["> one", "> two", "> [!note]", "> three"];
        let (body, next) = collect_quote_body(&lines, 0, true);
        assert_eq!(body, vec!["one", "two"]);
        assert_eq!(next, 2);
        let (all_body, all_next) = collect_quote_body(&lines, 0, false);
        assert_eq!(all_body.len(), 4);
        assert_eq!(all_next, 4);
    }
}
