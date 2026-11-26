use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Sense, Stroke, Ui};

use super::emphasis::{split_emphasis, Emphasis};
use super::tokens::{tokenize_inline, InlineToken};
use super::{PreviewAction, RenderContext, BODY_LINE_HEIGHT, BODY_SIZE};
use crate::theme::{semibold, Palette, RADIUS_BADGE};

fn body_text(text: &str, color: egui::Color32) -> RichText {
    RichText::new(text)
        .size(BODY_SIZE)
        .line_height(Some(BODY_LINE_HEIGHT))
        .color(color)
}

pub fn render_inline_line(
    ui: &mut Ui,
    context: &RenderContext,
    text: &str,
) -> Option<PreviewAction> {
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        for token in tokenize_inline(text) {
            if let Some(token_action) = render_token(ui, context, &token) {
                action = Some(token_action);
            }
        }
    });
    action
}

fn render_token(
    ui: &mut Ui,
    context: &RenderContext,
    token: &InlineToken,
) -> Option<PreviewAction> {
    let pal = context.pal;
    match *token {
        InlineToken::Plain(segment) => {
            render_emphasized(ui, pal, segment);
            None
        }
        InlineToken::Wikilink {
            target_file,
            display,
            is_embed,
        } => render_wikilink(ui, context, target_file, display, is_embed),
        InlineToken::Code(code) => {
            render_code_chip(ui, pal, code);
            None
        }
        InlineToken::Tag(tag) => render_tag_pill(ui, pal, tag),
    }
}

fn render_emphasized(ui: &mut Ui, pal: Palette, segment: &str) {
    for (emphasis, span) in split_emphasis(segment) {
        if span.is_empty() {
            continue;
        }
        let text = match emphasis {
            Emphasis::Normal => body_text(span, pal.text_primary),
            Emphasis::Bold => body_text(span, pal.text_primary).font(semibold(BODY_SIZE)),
            Emphasis::Strike => body_text(span, pal.text_muted).strikethrough(),
        };
        ui.label(text);
    }
}

fn render_wikilink(
    ui: &mut Ui,
    context: &RenderContext,
    target_file: &str,
    display: &str,
    is_embed: bool,
) -> Option<PreviewAction> {
    let pal = context.pal;
    let color = if (context.resolve_exists)(target_file) {
        pal.accent
    } else {
        pal.unresolved
    };
    let label = if is_embed {
        format!("[Embed: {display}]")
    } else {
        display.to_string()
    };
    let response =
        ui.add(egui::Label::new(body_text(&label, color).underline()).sense(Sense::click()));
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
        .clicked()
        .then(|| PreviewAction::OpenLink(target_file.to_string()))
}

fn render_code_chip(ui: &mut Ui, pal: Palette, code: &str) {
    Frame::new()
        .fill(pal.bg_elevated)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(RADIUS_BADGE))
        .inner_margin(Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.label(
                RichText::new(code)
                    .monospace()
                    .size(12.5)
                    .color(pal.text_primary),
            );
        });
}

fn render_tag_pill(ui: &mut Ui, pal: Palette, tag: &str) -> Option<PreviewAction> {
    let response = Frame::new()
        .fill(pal.accent_soft)
        .corner_radius(CornerRadius::same(RADIUS_BADGE))
        .inner_margin(Margin::symmetric(7, 1))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(format!("#{tag}"))
                        .font(semibold(12.0))
                        .color(pal.accent),
                )
                .sense(Sense::click()),
            )
        })
        .inner;
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
        .clicked()
        .then(|| PreviewAction::FilterTag(tag.to_string()))
}
