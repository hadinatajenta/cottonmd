use cottonmd_bases::{
    export_csv, query_view, BaseRow, BaseViewKind, BaseViewSpec, FilterPredicate,
};
use cottonmd_vault::EntryKind;
use eframe::egui::{self, RichText, Ui};

mod table;

use table::{show_header, show_row, SortRequest, ROW_HEIGHT};

use crate::state::AppState;
use crate::theme::Palette;
use crate::ui_kit::{card_frame, empty_state, pill_badge, section_label};

const COLUMN_IDS: [&str; 4] = ["file.name", "file.path", "file.tags", "file.size"];

pub struct BasesViewState {
    pub folder_filter: String,
    pub tag_filter: String,
    pub prop_key_filter: String,
    pub sort_col: String,
    pub descending: bool,
    pub export_feedback: Option<String>,
}

impl Default for BasesViewState {
    fn default() -> Self {
        Self {
            folder_filter: String::new(),
            tag_filter: String::new(),
            prop_key_filter: String::new(),
            sort_col: "file.name".to_string(),
            descending: false,
            export_feedback: None,
        }
    }
}

impl BasesViewState {
    pub fn ui(&mut self, ui: &mut Ui, state: &AppState, pal: Palette) -> Option<String> {
        let columns: Vec<String> = COLUMN_IDS.iter().map(|id| (*id).to_owned()).collect();
        let rows = self.query_rows(state, &columns);
        self.show_toolbar(ui, pal, &columns, &rows);
        if let Some(message) = &self.export_feedback {
            ui.label(RichText::new(message).size(11.5).color(pal.accent));
        }
        ui.add_space(8.0);
        if rows.is_empty() {
            empty_state(
                ui,
                pal,
                egui_phosphor::regular::TABLE,
                "No notes match the current Base filters. Clear a filter to show vault notes.",
            );
            return None;
        }
        self.show_table(ui, pal, state, &rows)
    }

    fn collect_filters(&self) -> Vec<FilterPredicate> {
        let mut filters = Vec::new();
        if !self.folder_filter.trim().is_empty() {
            filters.push(FilterPredicate::FolderPrefix(
                self.folder_filter.trim().to_string(),
            ));
        }
        if !self.tag_filter.trim().is_empty() {
            filters.push(FilterPredicate::HasTag(
                self.tag_filter.trim().trim_start_matches('#').to_string(),
            ));
        }
        if !self.prop_key_filter.trim().is_empty() {
            filters.push(FilterPredicate::PropertyExists(
                self.prop_key_filter.trim().to_string(),
            ));
        }
        filters
    }

    fn query_rows(&self, state: &AppState, columns: &[String]) -> Vec<BaseRow> {
        let spec = BaseViewSpec {
            name: "Vault Base".into(),
            kind: BaseViewKind::Table,
            columns: columns.to_vec(),
            filters: self.collect_filters(),
            sort_by: Some(self.sort_col.clone()),
            descending: self.descending,
        };
        let notes: Vec<_> = state
            .files
            .iter()
            .filter(|file| file.kind == EntryKind::Markdown)
            .filter_map(|file| {
                state
                    .parsed_notes
                    .get(&file.relative_path)
                    .map(|parsed| (file, parsed))
            })
            .collect();
        query_view(&spec, &notes)
    }

    fn show_toolbar(&mut self, ui: &mut Ui, pal: Palette, columns: &[String], rows: &[BaseRow]) {
        ui.horizontal_wrapped(|ui| {
            section_label(ui, pal, "Bases");
            pill_badge(ui, pal, &format!("{} rows", rows.len()));
            ui.add_space(8.0);
            filter_field(
                ui,
                pal,
                "Folder",
                "e.g. Guides/",
                &mut self.folder_filter,
                110.0,
            );
            filter_field(ui, pal, "Tag", "e.g. project", &mut self.tag_filter, 100.0);
            filter_field(
                ui,
                pal,
                "Has property",
                "e.g. status",
                &mut self.prop_key_filter,
                90.0,
            );
            let export_label = format!("{} Export CSV", egui_phosphor::regular::DOWNLOAD_SIMPLE);
            if ui.button(export_label).clicked() {
                self.export_rows(columns, rows);
            }
        });
    }

    fn export_rows(&mut self, columns: &[String], rows: &[BaseRow]) {
        let csv = export_csv(columns, rows);
        let Some(save_path) = rfd::FileDialog::new()
            .set_file_name("cottonmd-base.csv")
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            return;
        };
        self.export_feedback = Some(match std::fs::write(&save_path, csv) {
            Ok(()) => format!("Exported {} rows to {}", rows.len(), save_path.display()),
            Err(error) => format!("Export failed: {error}"),
        });
    }

    fn show_table(
        &mut self,
        ui: &mut Ui,
        pal: Palette,
        state: &AppState,
        rows: &[BaseRow],
    ) -> Option<String> {
        let mut open_path = None;
        let mut sort_request = None;
        card_frame(pal).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            sort_request = show_header(ui, pal, &self.sort_col, self.descending);
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, ROW_HEIGHT, rows.len(), |ui, visible_range| {
                    for row in &rows[visible_range] {
                        if show_row(ui, pal, state, row) {
                            open_path = Some(row.path.clone());
                        }
                    }
                });
        });
        if let Some(request) = sort_request {
            self.apply_sort(request);
        }
        open_path
    }

    fn apply_sort(&mut self, request: SortRequest) {
        if self.sort_col == request.column_id {
            self.descending = !self.descending;
        } else {
            self.sort_col = request.column_id.to_string();
            self.descending = false;
        }
    }
}

fn filter_field(
    ui: &mut Ui,
    pal: Palette,
    label: &str,
    hint: &str,
    value: &mut String,
    width: f32,
) {
    ui.label(
        RichText::new(format!("{label}:"))
            .size(12.0)
            .color(pal.text_muted),
    );
    ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(width),
    );
}
