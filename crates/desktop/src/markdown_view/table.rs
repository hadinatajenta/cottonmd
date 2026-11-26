use eframe::egui::{Align, Layout, Margin, RichText, Ui, Vec2};

use super::inline::render_inline_line;
use super::{BlockOutcome, PreviewAction, RenderContext};
use crate::theme::semibold;
use crate::ui_kit::card_frame;

const CELL_GAP: f32 = 20.0;
const MIN_COLUMN_WIDTH: f32 = 48.0;

#[derive(Debug, PartialEq)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub fn split_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let without_leading = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = without_leading.strip_suffix('|').unwrap_or(without_leading);
    inner
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect()
}

fn is_separator_row(line: &str) -> bool {
    if !line.contains('-') {
        return false;
    }
    let cells = split_cells(line);
    cells.iter().all(|cell| {
        let dashes = cell.trim_matches(':');
        !dashes.is_empty() && dashes.chars().all(|ch| ch == '-')
    })
}

pub fn parse_table(lines: &[&str], index: usize) -> Option<(Table, usize)> {
    let header_line = lines.get(index)?;
    let separator_line = lines.get(index + 1)?;
    if !header_line.contains('|') || !is_separator_row(separator_line) {
        return None;
    }
    let headers = split_cells(header_line);
    let column_count = headers.len();
    let rows: Vec<Vec<String>> = lines[index + 2..]
        .iter()
        .take_while(|line| !line.trim().is_empty() && line.contains('|'))
        .map(|line| fit_to_columns(split_cells(line), column_count))
        .collect();
    let next_line = index + 2 + rows.len();
    Some((Table { headers, rows }, next_line))
}

fn fit_to_columns(mut cells: Vec<String>, column_count: usize) -> Vec<String> {
    cells.resize(column_count, String::new());
    cells
}

pub fn column_width(available_width: f32, column_count: usize) -> f32 {
    if column_count == 0 || !available_width.is_finite() {
        return MIN_COLUMN_WIDTH;
    }
    let gaps = CELL_GAP * (column_count - 1) as f32;
    ((available_width - gaps) / column_count as f32).max(MIN_COLUMN_WIDTH)
}

pub fn table_block(
    ui: &mut Ui,
    context: &RenderContext,
    lines: &[&str],
    index: usize,
) -> Option<BlockOutcome> {
    let (table, next_line) = parse_table(lines, index)?;
    let action = show_table(ui, context, &table);
    ui.add_space(10.0);
    Some(BlockOutcome::consumed(next_line, action))
}

fn show_table(ui: &mut Ui, context: &RenderContext, table: &Table) -> Option<PreviewAction> {
    let pal = context.pal;
    let mut action = None;
    card_frame(pal)
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            let width = column_width(ui.available_width(), table.headers.len());
            show_header_row(ui, context, &table.headers, width);
            for row in &table.rows {
                ui.separator();
                if let Some(row_action) = show_row(ui, context, row, width) {
                    action = Some(row_action);
                }
            }
        });
    action
}

fn show_header_row(ui: &mut Ui, context: &RenderContext, headers: &[String], width: f32) {
    let pal = context.pal;
    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = CELL_GAP;
        for header in headers {
            ui.allocate_ui_with_layout(Vec2::new(width, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(width);
                ui.label(
                    RichText::new(header.to_uppercase())
                        .font(semibold(11.0))
                        .color(pal.text_muted),
                );
            });
        }
    });
}

fn show_row(
    ui: &mut Ui,
    context: &RenderContext,
    cells: &[String],
    width: f32,
) -> Option<PreviewAction> {
    let mut action = None;
    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = CELL_GAP;
        for cell in cells {
            ui.allocate_ui_with_layout(Vec2::new(width, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(width);
                if let Some(cell_action) = render_inline_line(ui, context, cell) {
                    action = Some(cell_action);
                }
            });
        }
    });
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_cells_ignores_outer_pipes() {
        assert_eq!(split_cells("| a | b |"), vec!["a", "b"]);
        assert_eq!(split_cells("a|b"), vec!["a", "b"]);
    }

    #[test]
    fn separator_row_accepts_alignment_markers_only() {
        assert!(is_separator_row("|---|:--:|--:|"));
        assert!(!is_separator_row("| a | b |"));
        assert!(!is_separator_row("|  |  |"));
        assert!(!is_separator_row("plain"));
    }

    #[test]
    fn parse_table_pads_short_rows_and_stops_at_blank_line() {
        let lines = [
            "| a | b |",
            "|---|---|",
            "| 1 |",
            "| 2 | 3 | 4 |",
            "",
            "after",
        ];
        let (table, next_line) = parse_table(&lines, 0).expect("table should parse");
        assert_eq!(table.headers, vec!["a", "b"]);
        assert_eq!(table.rows, vec![vec!["1", ""], vec!["2", "3"]]);
        assert_eq!(next_line, 4);
    }

    #[test]
    fn parse_table_rejects_header_without_separator() {
        assert!(parse_table(&["| a | b |", "| 1 | 2 |"], 0).is_none());
        assert!(parse_table(&["| a | b |"], 0).is_none());
    }

    #[test]
    fn column_width_is_never_below_minimum() {
        assert_eq!(column_width(10.0, 4), MIN_COLUMN_WIDTH);
        assert_eq!(column_width(f32::NAN, 3), MIN_COLUMN_WIDTH);
        assert_eq!(column_width(100.0, 0), MIN_COLUMN_WIDTH);
        assert!((column_width(440.0, 2) - 210.0).abs() < 1e-3);
    }
}
