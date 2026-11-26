use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use cottonmd_markdown::parse_note;

use crate::theme::Palette;

pub enum PreviewAction {
    OpenLink(String),
    FilterTag(String),
    ToggleTask(usize),
}

pub fn render_markdown_preview(
    ui: &mut egui::Ui,
    markdown: &str,
    pal: Palette,
    resolve_exists: &dyn Fn(&str) -> bool,
) -> Option<PreviewAction> {
    let mut action: Option<PreviewAction> = None;
    let parsed = parse_note(markdown);

    if !parsed.properties.is_empty() {
        Frame::new()
            .fill(pal.bg_elevated)
            .stroke(Stroke::new(1.0, pal.border))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Properties")
                        .size(11.0)
                        .color(pal.text_muted),
                );
                ui.add_space(4.0);
                egui::Grid::new("preview_props_grid")
                    .num_columns(2)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        for (k, v) in &parsed.properties {
                            ui.label(RichText::new(k).size(12.0).color(pal.text_muted));
                            ui.label(RichText::new(v).size(12.0).color(pal.text_primary));
                            ui.end_row();
                        }
                    });
            });
        ui.add_space(10.0);
    }

    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;

    if lines.first().map(|l| l.trim()) == Some("---") {
        i = 1;
        while i < lines.len() {
            if matches!(lines[i].trim(), "---" | "...") {
                i += 1;
                break;
            }
            i += 1;
        }
    }

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fence = if trimmed.starts_with("```") { "```" } else { "~~~" };
            let lang = trimmed.trim_start_matches(fence).trim();
            i += 1;
            let mut code_lines = Vec::new();
            while i < lines.len() && !lines[i].trim_start().starts_with(fence) {
                code_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            Frame::new()
                .fill(pal.bg_code)
                .stroke(Stroke::new(1.0, pal.border))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    if !lang.is_empty() {
                        ui.label(RichText::new(lang).size(10.5).color(pal.text_muted));
                        ui.add_space(2.0);
                    }
                    ui.label(
                        RichText::new(code_lines.join("\n"))
                            .monospace()
                            .size(12.5)
                            .color(pal.text_primary),
                    );
                });
            ui.add_space(6.0);
            continue;
        }

        if trimmed.is_empty() {
            ui.add_space(6.0);
            i += 1;
            continue;
        }

        if matches!(trimmed, "---" | "***" | "___") {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            i += 1;
            continue;
        }

        let hashes = trimmed.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&hashes) {
            if let Some(rest) = trimmed[hashes..].strip_prefix(' ') {
                let size = match hashes {
                    1 => 24.0,
                    2 => 20.0,
                    3 => 17.0,
                    4 => 15.0,
                    _ => 13.5,
                };
                ui.add_space(if hashes <= 2 { 10.0 } else { 6.0 });
                ui.label(
                    RichText::new(rest.trim())
                        .size(size)
                        .strong()
                        .color(pal.text_primary),
                );
                if hashes == 1 {
                    ui.separator();
                }
                ui.add_space(2.0);
                i += 1;
                continue;
            }
        }

        if trimmed.starts_with("> [!") {
            let header = trimmed.trim_start_matches('>').trim();
            let callout_title = header
                .strip_prefix("[!")
                .and_then(|s| s.split_once(']'))
                .map(|(kind, rest)| {
                    if rest.trim().is_empty() {
                        capitalize(kind)
                    } else {
                        format!("{}: {}", capitalize(kind), rest.trim())
                    }
                })
                .unwrap_or_else(|| header.to_string());
            i += 1;
            let mut body = Vec::new();
            while i < lines.len() && lines[i].trim_start().starts_with('>') {
                body.push(lines[i].trim_start().trim_start_matches('>').trim_start());
                i += 1;
            }
            Frame::new()
                .fill(pal.accent_soft)
                .stroke(Stroke::new(1.0, pal.accent))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(RichText::new(callout_title).strong().color(pal.accent));
                    if !body.is_empty() {
                        ui.add_space(4.0);
                        for bline in body {
                            if let Some(a) =
                                render_inline_line(ui, bline, pal, resolve_exists)
                            {
                                action = Some(a);
                            }
                        }
                    }
                });
            ui.add_space(6.0);
            continue;
        }

        if let Some(quote) = trimmed.strip_prefix('>') {
            Frame::new()
                .fill(pal.bg_elevated)
                .stroke(Stroke::new(1.0, pal.border))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    if let Some(a) =
                        render_inline_line(ui, quote.trim_start(), pal, resolve_exists)
                    {
                        action = Some(a);
                    }
                });
            ui.add_space(4.0);
            i += 1;
            continue;
        }

        if let Some((checked, task_text)) = parse_task_line(trimmed) {
            let line_no = i + 1;
            let mut is_checked = checked;
            ui.horizontal_wrapped(|ui| {
                if ui.checkbox(&mut is_checked, "").changed() {
                    action = Some(PreviewAction::ToggleTask(line_no));
                }
                if checked {
                    ui.label(
                        RichText::new(task_text)
                            .strikethrough()
                            .color(pal.text_muted),
                    );
                } else if let Some(a) =
                    render_inline_line(ui, task_text, pal, resolve_exists)
                {
                    action = Some(a);
                }
            });
            i += 1;
            continue;
        }

        if let Some(item) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
        {
            let indent = line.len().saturating_sub(trimmed.len()) as f32 * 4.0;
            ui.horizontal_wrapped(|ui| {
                ui.add_space(indent);
                ui.label(RichText::new("-").color(pal.text_muted));
                if let Some(a) = render_inline_line(ui, item, pal, resolve_exists) {
                    action = Some(a);
                }
            });
            i += 1;
            continue;
        }

        if let Some(a) = render_inline_line(ui, line, pal, resolve_exists) {
            action = Some(a);
        }
        i += 1;
    }

    action
}

fn parse_task_line(trimmed: &str) -> Option<(bool, &str)> {
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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn render_inline_line(
    ui: &mut egui::Ui,
    text: &str,
    pal: Palette,
    resolve_exists: &dyn Fn(&str) -> bool,
) -> Option<PreviewAction> {
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        let bytes = text.as_bytes();
        let mut cursor = 0;
        let mut plain_start = 0;

        let flush_plain = |ui: &mut egui::Ui, s: &str| {
            if !s.is_empty() {
                render_styled_plain(ui, s, pal);
            }
        };

        while cursor < bytes.len() {
            if cursor + 1 < bytes.len() && bytes[cursor] == b'[' && bytes[cursor + 1] == b'[' {
                let is_embed = cursor > 0 && bytes[cursor - 1] == b'!';
                let flush_end = if is_embed { cursor - 1 } else { cursor };
                flush_plain(ui, &text[plain_start..flush_end]);

                if let Some(rel_end) = text[cursor + 2..].find("]]") {
                    let raw = &text[cursor + 2..cursor + 2 + rel_end];
                    let (target_part, display) = match raw.split_once('|') {
                        Some((t, alias)) => (t.trim(), alias.trim()),
                        None => (raw.trim(), raw.trim()),
                    };
                    let target_file = target_part
                        .split_once('#')
                        .map(|(t, _)| t.trim())
                        .unwrap_or(target_part);
                    let exists = resolve_exists(target_file);
                    let color = if exists { pal.accent } else { pal.unresolved };
                    let label = if is_embed {
                        format!("[Embed: {display}]")
                    } else {
                        display.to_string()
                    };
                    let resp = ui.add(
                        egui::Label::new(RichText::new(label).color(color).underline())
                            .sense(egui::Sense::click()),
                    );
                    if resp.clicked() {
                        action = Some(PreviewAction::OpenLink(target_file.to_string()));
                    }
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    cursor = cursor + 2 + rel_end + 2;
                    plain_start = cursor;
                    continue;
                }
            }

            if bytes[cursor] == b'`' {
                if let Some(rel_end) = text[cursor + 1..].find('`') {
                    flush_plain(ui, &text[plain_start..cursor]);
                    let code = &text[cursor + 1..cursor + 1 + rel_end];
                    ui.label(
                        RichText::new(code)
                            .monospace()
                            .size(12.5)
                            .background_color(pal.bg_code)
                            .color(pal.text_primary),
                    );
                    cursor = cursor + 1 + rel_end + 1;
                    plain_start = cursor;
                    continue;
                }
            }

            if bytes[cursor] == b'#'
                && (cursor == 0 || bytes[cursor - 1].is_ascii_whitespace())
                && cursor + 1 < bytes.len()
            {
                let mut end = cursor + 1;
                let mut has_alpha = false;
                while end < bytes.len() {
                    let b = bytes[end];
                    if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'/' {
                        if !b.is_ascii_digit() {
                            has_alpha = true;
                        }
                        end += 1;
                    } else {
                        break;
                    }
                }
                if end > cursor + 1 && has_alpha {
                    flush_plain(ui, &text[plain_start..cursor]);
                    let tag = &text[cursor + 1..end];
                    let resp = ui.add(
                        egui::Label::new(
                            RichText::new(format!("#{tag}"))
                                .size(12.0)
                                .color(pal.accent)
                                .background_color(pal.accent_soft),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if resp.clicked() {
                        action = Some(PreviewAction::FilterTag(tag.to_string()));
                    }
                    cursor = end;
                    plain_start = cursor;
                    continue;
                }
            }

            cursor += 1;
            while cursor < bytes.len() && !text.is_char_boundary(cursor) {
                cursor += 1;
            }
        }

        flush_plain(ui, &text[plain_start..]);
    });
    action
}

fn render_styled_plain(ui: &mut egui::Ui, segment: &str, pal: Palette) {
    let mut rest = segment;
    while !rest.is_empty() {
        if let Some(pos) = rest.find("**") {
            let before = &rest[..pos];
            if !before.is_empty() {
                ui.label(RichText::new(before).color(pal.text_primary));
            }
            let after_open = &rest[pos + 2..];
            if let Some(close) = after_open.find("**") {
                let bold_text = &after_open[..close];
                ui.label(RichText::new(bold_text).strong().color(pal.text_primary));
                rest = &after_open[close + 2..];
                continue;
            }
            ui.label(RichText::new(&rest[pos..]).color(pal.text_primary));
            break;
        }
        if let Some(pos) = rest.find("~~") {
            let before = &rest[..pos];
            if !before.is_empty() {
                ui.label(RichText::new(before).color(pal.text_primary));
            }
            let after_open = &rest[pos + 2..];
            if let Some(close) = after_open.find("~~") {
                let struck = &after_open[..close];
                ui.label(RichText::new(struck).strikethrough().color(pal.text_muted));
                rest = &after_open[close + 2..];
                continue;
            }
            ui.label(RichText::new(&rest[pos..]).color(pal.text_primary));
            break;
        }
        ui.label(RichText::new(rest).color(pal.text_primary));
        break;
    }
    let _ = Color32::TRANSPARENT;
}
