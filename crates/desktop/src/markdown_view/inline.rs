use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Sense, Stroke, Ui};

use super::emphasis::{split_emphasis, Emphasis};
use super::{PreviewAction, RenderContext, BODY_LINE_HEIGHT, BODY_SIZE};
use crate::theme::{semibold, Palette, RADIUS_BADGE};

#[derive(Debug, PartialEq)]
pub enum InlineToken<'a> {
    Plain(&'a str),
    Wikilink {
        target_file: &'a str,
        display: &'a str,
        is_embed: bool,
    },
    Code(&'a str),
    Tag(&'a str),
}

pub fn tokenize_inline(text: &str) -> Vec<InlineToken<'_>> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    let mut plain_start = 0;
    while cursor < text.len() {
        let Some((token, end)) = parse_token_at(text, cursor) else {
            cursor += 1;
            while cursor < text.len() && !text.is_char_boundary(cursor) {
                cursor += 1;
            }
            continue;
        };
        let is_embed = matches!(token, InlineToken::Wikilink { is_embed: true, .. });
        let plain_end = if is_embed { cursor - 1 } else { cursor };
        push_plain(&mut tokens, &text[plain_start..plain_end]);
        tokens.push(token);
        cursor = end;
        plain_start = end;
    }
    push_plain(&mut tokens, &text[plain_start..]);
    tokens
}

fn push_plain<'a>(tokens: &mut Vec<InlineToken<'a>>, segment: &'a str) {
    if !segment.is_empty() {
        tokens.push(InlineToken::Plain(segment));
    }
}

fn parse_token_at(text: &str, cursor: usize) -> Option<(InlineToken<'_>, usize)> {
    match text.as_bytes()[cursor] {
        b'[' => parse_wikilink(text, cursor),
        b'`' => parse_code(text, cursor),
        b'#' => parse_tag(text, cursor),
        _ => None,
    }
}

fn parse_wikilink(text: &str, cursor: usize) -> Option<(InlineToken<'_>, usize)> {
    if !text[cursor..].starts_with("[[") {
        return None;
    }
    let relative_end = text[cursor + 2..].find("]]")?;
    let raw = &text[cursor + 2..cursor + 2 + relative_end];
    let (target_part, display) = match raw.split_once('|') {
        Some((target, alias)) => (target.trim(), alias.trim()),
        None => (raw.trim(), raw.trim()),
    };
    let target_file = target_part
        .split_once('#')
        .map_or(target_part, |(file, _)| file.trim());
    let is_embed = cursor > 0 && text.as_bytes()[cursor - 1] == b'!';
    let token = InlineToken::Wikilink {
        target_file,
        display,
        is_embed,
    };
    Some((token, cursor + 2 + relative_end + 2))
}

fn parse_code(text: &str, cursor: usize) -> Option<(InlineToken<'_>, usize)> {
    let relative_end = text[cursor + 1..].find('`')?;
    let code = &text[cursor + 1..cursor + 1 + relative_end];
    Some((InlineToken::Code(code), cursor + 1 + relative_end + 1))
}

fn parse_tag(text: &str, cursor: usize) -> Option<(InlineToken<'_>, usize)> {
    let bytes = text.as_bytes();
    let follows_boundary = cursor == 0 || bytes[cursor - 1].is_ascii_whitespace();
    if !follows_boundary || cursor + 1 >= bytes.len() {
        return None;
    }
    let is_tag_byte = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'/');
    let length = bytes[cursor + 1..]
        .iter()
        .take_while(|&&byte| is_tag_byte(byte))
        .count();
    let end = cursor + 1 + length;
    let has_letter = bytes[cursor + 1..end]
        .iter()
        .any(|byte| !byte.is_ascii_digit());
    (length > 0 && has_letter).then(|| (InlineToken::Tag(&text[cursor + 1..end]), end))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_a_single_token() {
        assert_eq!(
            tokenize_inline("hello world"),
            vec![InlineToken::Plain("hello world")]
        );
        assert!(tokenize_inline("").is_empty());
    }

    #[test]
    fn wikilink_with_alias_and_heading_resolves_file_target() {
        let tokens = tokenize_inline("see [[Notes/Plan#Goals|the plan]] now");
        assert_eq!(
            tokens,
            vec![
                InlineToken::Plain("see "),
                InlineToken::Wikilink {
                    target_file: "Notes/Plan",
                    display: "the plan",
                    is_embed: false
                },
                InlineToken::Plain(" now"),
            ]
        );
    }

    #[test]
    fn embed_consumes_the_leading_bang() {
        let tokens = tokenize_inline("a ![[img]]");
        assert_eq!(tokens[0], InlineToken::Plain("a "));
        assert_eq!(
            tokens[1],
            InlineToken::Wikilink {
                target_file: "img",
                display: "img",
                is_embed: true
            }
        );
    }

    #[test]
    fn unclosed_constructs_stay_plain_without_losing_text() {
        assert_eq!(
            tokenize_inline("a [[open"),
            vec![InlineToken::Plain("a [[open")]
        );
        assert_eq!(
            tokenize_inline("a `open"),
            vec![InlineToken::Plain("a `open")]
        );
        assert_eq!(
            tokenize_inline("x ![[open"),
            vec![InlineToken::Plain("x ![[open")]
        );
    }

    #[test]
    fn tags_need_a_boundary_and_a_letter() {
        assert_eq!(
            tokenize_inline("#todo and #123 and a#b"),
            vec![
                InlineToken::Tag("todo"),
                InlineToken::Plain(" and #123 and a#b"),
            ]
        );
    }

    #[test]
    fn multibyte_text_does_not_panic() {
        let tokens = tokenize_inline("日本語 `コード` #タグ [[ノート]]");
        assert!(tokens.contains(&InlineToken::Code("コード")));
    }
}
