use eframe::egui::{self, RichText};
use cottonmd_bases::{export_csv, query_view, BaseViewKind, BaseViewSpec, FilterPredicate};
use cottonmd_vault::EntryKind;

use crate::state::AppState;
use crate::theme::Palette;

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
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &AppState, pal: Palette) -> Option<String> {
        let mut open_path = None;

        let columns = vec![
            "file.name".to_string(),
            "file.path".to_string(),
            "file.tags".to_string(),
            "file.size".to_string(),
        ];

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

        let spec = BaseViewSpec {
            name: "Vault Base".into(),
            kind: BaseViewKind::Table,
            columns: columns.clone(),
            filters,
            sort_by: Some(self.sort_col.clone()),
            descending: self.descending,
        };

        let items: Vec<_> = state
            .files
            .iter()
            .filter(|f| f.kind == EntryKind::Markdown)
            .filter_map(|f| state.parsed_notes.get(&f.relative_path).map(|p| (f, p)))
            .collect();

        let rows = query_view(&spec, &items);

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Bases Table").strong().color(pal.text_primary));
            ui.separator();
            ui.label(RichText::new("Folder:").size(12.0).color(pal.text_muted));
            ui.add(
                egui::TextEdit::singleline(&mut self.folder_filter)
                    .hint_text("e.g. Guides/")
                    .desired_width(110.0),
            );
            ui.label(RichText::new("Tag:").size(12.0).color(pal.text_muted));
            ui.add(
                egui::TextEdit::singleline(&mut self.tag_filter)
                    .hint_text("e.g. project")
                    .desired_width(100.0),
            );
            ui.label(RichText::new("Has property:").size(12.0).color(pal.text_muted));
            ui.add(
                egui::TextEdit::singleline(&mut self.prop_key_filter)
                    .hint_text("e.g. status")
                    .desired_width(90.0),
            );
            if ui.button("Export CSV").clicked() {
                let csv = export_csv(&columns, &rows);
                if let Some(save_path) = rfd::FileDialog::new()
                    .set_file_name("cottonmd-base.csv")
                    .add_filter("CSV", &["csv"])
                    .save_file()
                {
                    match std::fs::write(&save_path, csv) {
                        Ok(()) => {
                            self.export_feedback =
                                Some(format!("Exported {} rows to {}", rows.len(), save_path.display()));
                        }
                        Err(e) => {
                            self.export_feedback = Some(format!("Export failed: {e}"));
                        }
                    }
                }
            }
            ui.label(
                RichText::new(format!("{} rows", rows.len()))
                    .size(11.5)
                    .color(pal.text_muted),
            );
        });

        if let Some(ref msg) = self.export_feedback {
            ui.label(RichText::new(msg).size(11.5).color(pal.accent));
        }

        ui.separator();

        if rows.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("No notes match the current Base filters. Clear a filter to show vault notes.")
                        .color(pal.text_muted),
                );
            });
            return None;
        }

        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("bases_table_grid")
                .striped(true)
                .spacing([16.0, 6.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    for (col_id, heading) in [
                        ("file.name", "Note"),
                        ("file.path", "Path"),
                        ("file.tags", "Tags"),
                        ("file.size", "Bytes"),
                    ] {
                        let arrow = if self.sort_col == col_id {
                            if self.descending { " v" } else { " ^" }
                        } else {
                            ""
                        };
                        if ui
                            .button(RichText::new(format!("{heading}{arrow}")).strong())
                            .clicked()
                        {
                            if self.sort_col == col_id {
                                self.descending = !self.descending;
                            } else {
                                self.sort_col = col_id.to_string();
                                self.descending = false;
                            }
                        }
                    }
                    ui.label(RichText::new("Links").strong().color(pal.text_muted));
                    ui.label(RichText::new("Backlinks").strong().color(pal.text_muted));
                    ui.end_row();

                    for row in &rows {
                        let resp = ui.add(
                            egui::Label::new(
                                RichText::new(&row.title).color(pal.accent).underline(),
                            )
                            .sense(egui::Sense::click()),
                        );
                        if resp.clicked() {
                            open_path = Some(row.path.clone());
                        }
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        ui.label(RichText::new(&row.path).size(12.0).color(pal.text_muted));
                        let tags = row.values.get("file.tags").map(String::as_str).unwrap_or("");
                        ui.label(RichText::new(tags).size(12.0).color(pal.text_primary));
                        let size = row.values.get("file.size").map(String::as_str).unwrap_or("0");
                        ui.label(RichText::new(size).monospace().size(12.0).color(pal.text_muted));
                        ui.label(
                            RichText::new(state.graph.outgoing_links(&row.path).len().to_string())
                                .monospace()
                                .size(12.0)
                                .color(pal.text_muted),
                        );
                        ui.label(
                            RichText::new(state.graph.backlinks(&row.path).len().to_string())
                                .monospace()
                                .size(12.0)
                                .color(pal.text_muted),
                        );
                        ui.end_row();
                    }
                });
        });

        open_path
    }
}
