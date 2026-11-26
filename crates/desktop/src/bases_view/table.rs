use cottonmd_bases::BaseRow;
use eframe::egui::{
    self, Align, CornerRadius, Layout, Rect, RichText, Sense, Stroke, Ui, UiBuilder, Vec2,
};

use crate::state::AppState;
use crate::theme::{semibold, Palette, RADIUS_CONTROL};
use crate::ui_kit::{pill_badge, type_chip};

pub const ROW_HEIGHT: f32 = 40.0;
const HEADER_HEIGHT: f32 = 30.0;
const CELL_PADDING: f32 = 10.0;
const MAX_VISIBLE_TAGS: usize = 3;
const COLUMN_WEIGHTS: [f32; 6] = [0.24, 0.27, 0.21, 0.08, 0.10, 0.10];

pub struct ColumnSpec {
    pub sort_id: Option<&'static str>,
    pub heading: &'static str,
}

pub const COLUMNS: [ColumnSpec; 6] = [
    ColumnSpec {
        sort_id: Some("file.name"),
        heading: "Note",
    },
    ColumnSpec {
        sort_id: Some("file.path"),
        heading: "Path",
    },
    ColumnSpec {
        sort_id: Some("file.tags"),
        heading: "Tags",
    },
    ColumnSpec {
        sort_id: Some("file.size"),
        heading: "Bytes",
    },
    ColumnSpec {
        sort_id: None,
        heading: "Links",
    },
    ColumnSpec {
        sort_id: None,
        heading: "Backlinks",
    },
];

pub struct SortRequest {
    pub column_id: &'static str,
}

pub fn column_widths(total_width: f32) -> [f32; 6] {
    if !total_width.is_finite() || total_width <= 0.0 {
        return [0.0; 6];
    }
    let weight_sum: f32 = COLUMN_WEIGHTS.iter().sum();
    COLUMN_WEIGHTS.map(|weight| total_width * weight / weight_sum)
}

pub fn split_tags(raw_tags: &str) -> Vec<&str> {
    raw_tags
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .collect()
}

pub fn show_header(
    ui: &mut Ui,
    pal: Palette,
    sort_column: &str,
    is_descending: bool,
) -> Option<SortRequest> {
    let widths = column_widths(ui.available_width());
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), HEADER_HEIGHT),
        Sense::hover(),
    );
    let mut request = None;
    let mut left = rect.left();
    for (column, width) in COLUMNS.iter().zip(widths) {
        let cell_rect = Rect::from_min_size(
            egui::pos2(left, rect.top()),
            Vec2::new(width, HEADER_HEIGHT),
        );
        left += width;
        let is_sorted = column.sort_id == Some(sort_column);
        let clicked = ui
            .scope_builder(cell_builder(cell_rect), |cell_ui| {
                header_cell(cell_ui, pal, column, is_sorted, is_descending)
            })
            .inner;
        if let (true, Some(column_id)) = (clicked, column.sort_id) {
            request = Some(SortRequest { column_id });
        }
    }
    request
}

fn header_cell(
    ui: &mut Ui,
    pal: Palette,
    column: &ColumnSpec,
    is_sorted: bool,
    is_descending: bool,
) -> bool {
    let arrow = match (is_sorted, is_descending) {
        (false, _) => "",
        (true, true) => egui_phosphor::regular::CARET_DOWN,
        (true, false) => egui_phosphor::regular::CARET_UP,
    };
    let color = if is_sorted {
        pal.text_primary
    } else {
        pal.text_muted
    };
    let text = RichText::new(format!("{} {arrow}", column.heading.to_uppercase()))
        .font(semibold(11.0))
        .color(color);
    let sense = if column.sort_id.is_some() {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = ui.add(egui::Label::new(text).sense(sense).truncate());
    response.clicked()
}

pub fn show_row(ui: &mut Ui, pal: Palette, state: &AppState, row: &BaseRow) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(RADIUS_CONTROL), pal.bg_elevated);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    ui.painter()
        .hline(rect.x_range(), rect.bottom(), Stroke::new(1.0, pal.border));
    let widths = column_widths(rect.width());
    let mut left = rect.left();
    for (column_index, width) in widths.into_iter().enumerate() {
        let cell_rect =
            Rect::from_min_size(egui::pos2(left, rect.top()), Vec2::new(width, ROW_HEIGHT));
        left += width;
        ui.scope_builder(cell_builder(cell_rect), |cell_ui| {
            row_cell(cell_ui, pal, state, row, column_index, response.hovered());
        });
    }
    response.clicked()
}

fn cell_builder(cell_rect: Rect) -> UiBuilder {
    UiBuilder::new()
        .max_rect(cell_rect.shrink2(Vec2::new(CELL_PADDING, 0.0)))
        .layout(Layout::left_to_right(Align::Center))
}

fn row_cell(
    ui: &mut Ui,
    pal: Palette,
    state: &AppState,
    row: &BaseRow,
    column_index: usize,
    is_row_hovered: bool,
) {
    match column_index {
        0 => title_cell(ui, pal, &row.title, is_row_hovered),
        1 => truncated_mono(ui, &row.path, pal.text_muted),
        2 => tag_cell(
            ui,
            pal,
            row.values.get("file.tags").map_or("", String::as_str),
        ),
        3 => truncated_mono(
            ui,
            row.values.get("file.size").map_or("0", String::as_str),
            pal.text_muted,
        ),
        4 => {
            pill_badge(
                ui,
                pal,
                &state.graph.outgoing_links(&row.path).len().to_string(),
            );
        }
        _ => {
            pill_badge(ui, pal, &state.graph.backlinks(&row.path).len().to_string());
        }
    }
}

fn title_cell(ui: &mut Ui, pal: Palette, title: &str, is_row_hovered: bool) {
    ui.label(
        RichText::new(egui_phosphor::regular::FILE_TEXT)
            .size(15.0)
            .color(pal.text_muted),
    );
    let mut text = RichText::new(title)
        .font(semibold(13.5))
        .color(pal.text_primary);
    if is_row_hovered {
        text = text.color(pal.accent).underline();
    }
    ui.add(egui::Label::new(text).truncate());
}

fn truncated_mono(ui: &mut Ui, text: &str, color: egui::Color32) {
    ui.add(egui::Label::new(RichText::new(text).monospace().size(12.0).color(color)).truncate());
}

fn tag_cell(ui: &mut Ui, pal: Palette, raw_tags: &str) {
    let tags = split_tags(raw_tags);
    for tag in tags.iter().take(MAX_VISIBLE_TAGS) {
        type_chip(ui, pal, tag);
    }
    if tags.len() > MAX_VISIBLE_TAGS {
        pill_badge(ui, pal, &format!("+{}", tags.len() - MAX_VISIBLE_TAGS));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_widths_fill_the_total() {
        let total: f32 = column_widths(960.0).iter().sum();
        assert!((total - 960.0).abs() < 0.01);
    }

    #[test]
    fn column_widths_are_zero_for_unusable_totals() {
        assert_eq!(column_widths(f32::NAN), [0.0; 6]);
        assert_eq!(column_widths(-10.0), [0.0; 6]);
        assert_eq!(column_widths(f32::INFINITY), [0.0; 6]);
    }

    #[test]
    fn split_tags_trims_and_drops_blanks() {
        assert_eq!(split_tags("a, b ,, c"), vec!["a", "b", "c"]);
        assert!(split_tags("").is_empty());
    }
}
