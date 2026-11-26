#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bases_view;
mod canvas_view;
mod graph_view;
mod markdown_view;
mod state;
mod theme;

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{self, CornerRadius, Frame, Key, Margin, Modifiers, RichText, Stroke};
use cottonmd_vault::EntryKind;

const MOD_KEY: &str = if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" };
const APP_ICON_RGBA: &[u8] = include_bytes!("../../../assets/AppIcon-64.rgba");

use bases_view::BasesViewState;
use canvas_view::CanvasViewState;
use graph_view::GraphViewState;
use markdown_view::{render_markdown_preview, PreviewAction};
use state::{resolve_link_in_files, AppState, CenterMode, FolderTreeNode, LeftTab, RightTab};
use theme::{apply_theme, Palette};

struct CottonMdApp {
    state: AppState,
    graph_view: GraphViewState,
    bases_view: BasesViewState,
    canvas_view: CanvasViewState,
}

impl CottonMdApp {
    fn new(initial_vault: Option<PathBuf>) -> Self {
        Self {
            state: AppState::new(initial_vault),
            graph_view: GraphViewState::default(),
            bases_view: BasesViewState::default(),
            canvas_view: CanvasViewState::default(),
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input_mut(|i| {
            if i.consume_key(Modifiers::COMMAND, Key::S) {
                self.state.save_active_file();
            }
            if i.consume_key(Modifiers::COMMAND, Key::K) || i.consume_key(Modifiers::COMMAND, Key::O)
            {
                self.state.show_quick_switcher = !self.state.show_quick_switcher;
                self.state.quick_switcher_query.clear();
                self.state.quick_switcher_focus_needed = true;
            }
            if i.consume_key(Modifiers::COMMAND, Key::N) {
                self.state.show_new_note_dialog = true;
                self.state.new_note_name.clear();
            }
            if i.consume_key(Modifiers::COMMAND, Key::E) {
                self.state.center_mode = match self.state.center_mode {
                    CenterMode::Edit => CenterMode::Split,
                    CenterMode::Split => CenterMode::Preview,
                    _ => CenterMode::Edit,
                };
            }
            if i.consume_key(Modifiers::COMMAND, Key::G) {
                self.state.center_mode = CenterMode::Graph;
            }
            if i.consume_key(Modifiers::COMMAND, Key::B) {
                self.state.center_mode = CenterMode::Bases;
            }
            if i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::F) {
                self.state.show_left_sidebar = true;
                self.state.left_tab = LeftTab::Search;
            }
            if i.consume_key(Modifiers::NONE, Key::Escape) {
                self.state.show_quick_switcher = false;
                self.state.show_new_note_dialog = false;
            }
        });
    }

    fn draw_top_bar(&mut self, ctx: &egui::Context, pal: Palette) {
        egui::TopBottomPanel::top("top_bar")
            .frame(
                Frame::new()
                    .fill(pal.bg_sidebar)
                    .stroke(Stroke::new(1.0, pal.border))
                    .inner_margin(Margin::symmetric(10, 6)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.state.show_left_sidebar, "Sidebar")
                        .clicked()
                    {
                        self.state.show_left_sidebar = !self.state.show_left_sidebar;
                    }

                    if ui.button("Open Vault").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.state.open_vault(&folder);
                        }
                    }

                    if ui
                        .add_enabled(self.state.vault.is_some(), egui::Button::new("New Note"))
                        .clicked()
                    {
                        self.state.show_new_note_dialog = true;
                        self.state.new_note_name.clear();
                    }

                    if ui
                        .add_enabled(self.state.vault.is_some(), egui::Button::new("Quick Switch"))
                        .on_hover_text(format!("{MOD_KEY}+K"))
                        .clicked()
                    {
                        self.state.show_quick_switcher = true;
                        self.state.quick_switcher_query.clear();
                        self.state.quick_switcher_focus_needed = true;
                    }

                    ui.separator();

                    for (mode, label) in [
                        (CenterMode::Edit, "Edit"),
                        (CenterMode::Split, "Split"),
                        (CenterMode::Preview, "Preview"),
                        (CenterMode::Graph, "Graph"),
                        (CenterMode::Bases, "Bases"),
                        (CenterMode::Canvas, "Canvas"),
                    ] {
                        if ui
                            .selectable_label(self.state.center_mode == mode, label)
                            .clicked()
                        {
                            self.state.center_mode = mode;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .selectable_label(self.state.show_right_sidebar, "Inspector")
                            .clicked()
                        {
                            self.state.show_right_sidebar = !self.state.show_right_sidebar;
                        }

                        let theme_label = if self.state.dark_mode { "Light" } else { "Dark" };
                        if ui.button(theme_label).clicked() {
                            self.state.dark_mode = !self.state.dark_mode;
                        }

                        let dirty = self.state.is_dirty();
                        let save_btn = if dirty {
                            egui::Button::new(RichText::new("Save").color(egui::Color32::WHITE))
                                .fill(pal.accent)
                        } else {
                            egui::Button::new("Save")
                        };
                        if ui
                            .add_enabled(dirty, save_btn)
                            .on_hover_text(format!("{MOD_KEY}+S"))
                            .clicked()
                        {
                            self.state.save_active_file();
                        }

                        if let Some(ref path) = self.state.active_path {
                            let label = if dirty {
                                format!("{path} *")
                            } else {
                                path.clone()
                            };
                            ui.label(
                                RichText::new(label)
                                    .size(12.0)
                                    .color(if dirty { pal.unsaved } else { pal.text_muted }),
                            );
                        }
                    });
                });
            });
    }

    fn draw_left_sidebar(&mut self, ctx: &egui::Context, pal: Palette) {
        if !self.state.show_left_sidebar {
            return;
        }
        let mut open_path: Option<String> = None;

        egui::SidePanel::left("left_sidebar")
            .resizable(true)
            .default_width(250.0)
            .min_width(180.0)
            .max_width(420.0)
            .frame(
                Frame::new()
                    .fill(pal.bg_sidebar)
                    .stroke(Stroke::new(1.0, pal.border))
                    .inner_margin(Margin::same(10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.state.left_tab, LeftTab::Files, "Files");
                    ui.selectable_value(&mut self.state.left_tab, LeftTab::Search, "Search");
                    ui.selectable_value(&mut self.state.left_tab, LeftTab::Tags, "Tags");
                });
                ui.separator();

                if self.state.vault.is_none() {
                    ui.label(
                        RichText::new("No vault open. Click Open Vault to choose a folder.")
                            .size(12.0)
                            .color(pal.text_muted),
                    );
                    return;
                }

                match self.state.left_tab {
                    LeftTab::Files => {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.state.file_filter)
                                    .hint_text("Filter files...")
                                    .desired_width(ui.available_width() - 56.0),
                            );
                            if ui.button("Reload").clicked() {
                                self.state.reindex_vault();
                            }
                        });
                        ui.add_space(6.0);

                        let filter = self.state.file_filter.trim().to_ascii_lowercase();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if !filter.is_empty() {
                                for f in &self.state.files {
                                    if f.relative_path.to_ascii_lowercase().contains(&filter) {
                                        let active = self.state.active_path.as_deref()
                                            == Some(f.relative_path.as_str());
                                        if ui.selectable_label(active, &f.relative_path).clicked() {
                                            open_path = Some(f.relative_path.clone());
                                        }
                                    }
                                }
                            } else {
                                draw_folder_tree(
                                    ui,
                                    &self.state.folder_tree,
                                    self.state.active_path.as_deref(),
                                    &self.state.file_map,
                                    pal,
                                    &mut open_path,
                                );
                            }
                        });
                    }
                    LeftTab::Search => {
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.state.search_query)
                                .hint_text("Search titles and content...")
                                .desired_width(ui.available_width()),
                        );
                        if resp.changed() {
                            self.state.refresh_search();
                        }
                        ui.add_space(6.0);
                        if self.state.search_query.trim().is_empty() {
                            ui.label(
                                RichText::new("Type a keyword to search notes in this vault.")
                                    .size(12.0)
                                    .color(pal.text_muted),
                            );
                        } else if self.state.search_results.is_empty() {
                            ui.label(
                                RichText::new("No matching notes found.")
                                    .size(12.0)
                                    .color(pal.text_muted),
                            );
                        } else {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for hit in &self.state.search_results {
                                    let header = if hit.line == 0 {
                                        hit.path.clone()
                                    } else {
                                        format!("{}:{}", hit.path, hit.line)
                                    };
                                    Frame::new()
                                        .fill(pal.bg_elevated)
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::symmetric(8, 6))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            if ui
                                                .add(
                                                    egui::Label::new(
                                                        RichText::new(header)
                                                            .size(12.0)
                                                            .color(pal.accent),
                                                    )
                                                    .sense(egui::Sense::click()),
                                                )
                                                .clicked()
                                            {
                                                open_path = Some(hit.path.clone());
                                            }
                                            ui.label(
                                                RichText::new(&hit.snippet)
                                                    .size(11.5)
                                                    .color(pal.text_muted),
                                            );
                                        });
                                    ui.add_space(4.0);
                                }
                            });
                        }
                    }
                    LeftTab::Tags => {
                        if self.state.graph.tags().is_empty() {
                            ui.label(
                                RichText::new("No tags indexed in this vault yet.")
                                    .size(12.0)
                                    .color(pal.text_muted),
                            );
                        } else {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (tag, notes) in self.state.graph.tags() {
                                    let selected = self.state.selected_tag.as_deref()
                                        == Some(tag.as_str());
                                    if ui
                                        .selectable_label(
                                            selected,
                                            format!("#{tag}  ({})", notes.len()),
                                        )
                                        .clicked()
                                    {
                                        self.state.selected_tag = if selected {
                                            None
                                        } else {
                                            Some(tag.clone())
                                        };
                                    }
                                    if selected {
                                        ui.indent(tag, |ui| {
                                            for note_path in notes {
                                                let active = self.state.active_path.as_deref()
                                                    == Some(note_path.as_str());
                                                if ui
                                                    .selectable_label(active, note_path)
                                                    .clicked()
                                                {
                                                    open_path = Some(note_path.clone());
                                                }
                                            }
                                        });
                                    }
                                }
                            });
                        }
                    }
                }
            });

        if let Some(p) = open_path {
            self.state.open_file(&p);
        }
    }

    fn draw_right_sidebar(&mut self, ctx: &egui::Context, pal: Palette) {
        if !self.state.show_right_sidebar {
            return;
        }
        let mut open_path: Option<String> = None;
        let mut open_wikilink: Option<String> = None;
        let mut toggle_task: Option<usize> = None;

        egui::SidePanel::right("right_sidebar")
            .resizable(true)
            .default_width(250.0)
            .min_width(180.0)
            .max_width(400.0)
            .frame(
                Frame::new()
                    .fill(pal.bg_sidebar)
                    .stroke(Stroke::new(1.0, pal.border))
                    .inner_margin(Margin::same(10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.state.right_tab, RightTab::Backlinks, "Links");
                    ui.selectable_value(&mut self.state.right_tab, RightTab::Outline, "Outline");
                    ui.selectable_value(
                        &mut self.state.right_tab,
                        RightTab::Properties,
                        "Props",
                    );
                });
                ui.separator();

                let Some(ref active) = self.state.active_path else {
                    ui.label(
                        RichText::new("Select a note to inspect links, headings, and properties.")
                            .size(12.0)
                            .color(pal.text_muted),
                    );
                    return;
                };

                egui::ScrollArea::vertical().show(ui, |ui| match self.state.right_tab {
                    RightTab::Backlinks => {
                        let backlinks = self.state.graph.backlinks(active);
                        ui.label(
                            RichText::new(format!("Backlinks ({})", backlinks.len()))
                                .strong()
                                .size(12.0)
                                .color(pal.text_primary),
                        );
                        if backlinks.is_empty() {
                            ui.label(
                                RichText::new("No notes link to this note yet.")
                                    .size(11.5)
                                    .color(pal.text_muted),
                            );
                        } else {
                            for src in backlinks {
                                if ui.link(src).clicked() {
                                    open_path = Some(src.to_string());
                                }
                            }
                        }

                        ui.add_space(12.0);
                        let outgoing = self.state.graph.outgoing_links(active);
                        ui.label(
                            RichText::new(format!("Outgoing links ({})", outgoing.len()))
                                .strong()
                                .size(12.0)
                                .color(pal.text_primary),
                        );
                        if outgoing.is_empty() {
                            ui.label(
                                RichText::new("No resolved outgoing links.")
                                    .size(11.5)
                                    .color(pal.text_muted),
                            );
                        } else {
                            for dst in outgoing {
                                if ui.link(dst).clicked() {
                                    open_path = Some(dst.to_string());
                                }
                            }
                        }

                        if let Some(missing) = self.state.graph.unresolved_links().get(active) {
                            if !missing.is_empty() {
                                ui.add_space(12.0);
                                ui.label(
                                    RichText::new(format!("Unresolved ({})", missing.len()))
                                        .strong()
                                        .size(12.0)
                                        .color(pal.unresolved),
                                );
                                for target in missing {
                                    if ui
                                        .add(
                                            egui::Label::new(
                                                RichText::new(format!("[[{target}]]"))
                                                    .color(pal.unresolved)
                                                    .underline(),
                                            )
                                            .sense(egui::Sense::click()),
                                        )
                                        .on_hover_text("Click to create note")
                                        .clicked()
                                    {
                                        open_wikilink = Some(target.clone());
                                    }
                                }
                            }
                        }
                    }
                    RightTab::Outline => {
                        let parsed = self.state.active_parsed().unwrap_or_default();
                        if parsed.headings.is_empty() {
                            ui.label(
                                RichText::new("No headings in this note.")
                                    .size(12.0)
                                    .color(pal.text_muted),
                            );
                        } else {
                            for h in &parsed.headings {
                                let indent = (h.level.saturating_sub(1) as f32) * 12.0;
                                ui.horizontal(|ui| {
                                    ui.add_space(indent);
                                    ui.label(
                                        RichText::new(&h.text)
                                            .size(12.5)
                                            .color(pal.text_primary),
                                    );
                                });
                            }
                        }
                    }
                    RightTab::Properties => {
                        let parsed = self.state.active_parsed().unwrap_or_default();
                        ui.label(
                            RichText::new(format!("Properties ({})", parsed.properties.len()))
                                .strong()
                                .size(12.0),
                        );
                        if parsed.properties.is_empty() {
                            ui.label(
                                RichText::new("No YAML frontmatter properties.")
                                    .size(11.5)
                                    .color(pal.text_muted),
                            );
                        } else {
                            for (k, v) in &parsed.properties {
                                ui.label(RichText::new(k).size(11.0).color(pal.text_muted));
                                ui.label(RichText::new(v).size(12.0).color(pal.text_primary));
                                ui.add_space(4.0);
                            }
                        }

                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(format!("Tasks ({})", parsed.tasks.len()))
                                .strong()
                                .size(12.0),
                        );
                        if parsed.tasks.is_empty() {
                            ui.label(
                                RichText::new("No tasks in this note.")
                                    .size(11.5)
                                    .color(pal.text_muted),
                            );
                        } else {
                            for task in &parsed.tasks {
                                let mut done = task.status == 'x' || task.status == 'X';
                                ui.horizontal_wrapped(|ui| {
                                    if ui.checkbox(&mut done, "").changed() {
                                        toggle_task = Some(task.line);
                                    }
                                    ui.label(
                                        RichText::new(&task.text)
                                            .size(12.0)
                                            .color(if done { pal.done } else { pal.text_primary }),
                                    );
                                });
                            }
                        }
                    }
                });
            });

        if let Some(p) = open_path {
            self.state.open_file(&p);
        }
        if let Some(link) = open_wikilink {
            self.state.open_wikilink(&link);
        }
        if let Some(line) = toggle_task {
            self.state.toggle_task_line(line);
        }
    }

    fn draw_center(&mut self, ctx: &egui::Context, pal: Palette) {
        let mut preview_action: Option<PreviewAction> = None;
        let mut graph_open: Option<String> = None;
        let mut bases_open: Option<String> = None;
        let mut canvas_open: Option<String> = None;

        egui::CentralPanel::default()
            .frame(Frame::new().fill(pal.bg_editor).inner_margin(Margin::same(14)))
            .show(ctx, |ui| match self.state.center_mode {
                CenterMode::Graph => {
                    graph_open = self.graph_view.ui(ui, &self.state, pal);
                }
                CenterMode::Bases => {
                    bases_open = self.bases_view.ui(ui, &self.state, pal);
                }
                CenterMode::Canvas => {
                    canvas_open =
                        self.canvas_view
                            .ui(ui, self.state.active_canvas.as_ref(), pal);
                }
                CenterMode::Edit | CenterMode::Split | CenterMode::Preview => {
                    if self.state.active_path.is_none() {
                        ui.centered_and_justified(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("CottonMD")
                                        .size(22.0)
                                        .strong()
                                        .color(pal.text_primary),
                                );
                                ui.add_space(6.0);
                                let msg = if self.state.vault.is_some() {
                                    format!("Select a note from Files or press {MOD_KEY}+N to create one.")
                                } else {
                                    "Open a local Markdown vault folder to start writing.".to_string()
                                };
                                ui.label(RichText::new(msg).color(pal.text_muted));
                                ui.add_space(12.0);
                                if ui.button("Open Vault Folder").clicked() {
                                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                        self.state.open_vault(&folder);
                                    }
                                }
                            });
                        });
                        return;
                    }

                    if let Some(ref err) = self.state.canvas_error {
                        ui.label(
                            RichText::new(format!("Canvas JSON parse error: {err}"))
                                .color(pal.unresolved),
                        );
                    }

                    let files_ref = &self.state.files;
                    let exists_fn = |target: &str| resolve_link_in_files(files_ref, target).is_some();

                    match self.state.center_mode {
                        CenterMode::Edit => {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.state.editor_buffer)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(36)
                                        .frame(false),
                                );
                            });
                        }
                        CenterMode::Preview => {
                            let content = self.state.editor_buffer.clone();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                preview_action =
                                    render_markdown_preview(ui, &content, pal, &exists_fn);
                            });
                        }
                        CenterMode::Split => {
                            let content = self.state.editor_buffer.clone();
                            ui.columns(2, |cols| {
                                egui::ScrollArea::vertical()
                                    .id_salt("split_editor_scroll")
                                    .show(&mut cols[0], |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(&mut self.state.editor_buffer)
                                                .font(egui::TextStyle::Monospace)
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(36)
                                                .frame(false),
                                        );
                                    });
                                egui::ScrollArea::vertical()
                                    .id_salt("split_preview_scroll")
                                    .show(&mut cols[1], |ui| {
                                        Frame::new()
                                            .fill(pal.bg_editor)
                                            .stroke(Stroke::new(1.0, pal.border))
                                            .corner_radius(CornerRadius::same(4))
                                            .inner_margin(Margin::symmetric(14, 10))
                                            .show(ui, |ui| {
                                                ui.set_min_width(ui.available_width());
                                                preview_action = render_markdown_preview(
                                                    ui,
                                                    &content,
                                                    pal,
                                                    &exists_fn,
                                                );
                                            });
                                    });
                            });
                        }
                        _ => {}
                    }
                }
            });

        if let Some(action) = preview_action {
            match action {
                PreviewAction::OpenLink(target) => self.state.open_wikilink(&target),
                PreviewAction::FilterTag(tag) => {
                    self.state.show_left_sidebar = true;
                    self.state.left_tab = LeftTab::Tags;
                    self.state.selected_tag = Some(tag);
                }
                PreviewAction::ToggleTask(line) => self.state.toggle_task_line(line),
            }
        }
        if let Some(p) = graph_open.or(bases_open).or(canvas_open) {
            self.state.open_file(&p);
        }
    }

    fn draw_status_bar(&self, ctx: &egui::Context, pal: Palette) {
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                Frame::new()
                    .fill(pal.bg_sidebar)
                    .stroke(Stroke::new(1.0, pal.border))
                    .inner_margin(Margin::symmetric(10, 4)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let vault_label = self
                        .state
                        .vault
                        .as_ref()
                        .and_then(|v| v.root().file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "No vault".into());
                    ui.label(RichText::new(vault_label).size(11.5).color(pal.text_muted));
                    ui.separator();
                    ui.label(
                        RichText::new(format!("{} notes", self.state.graph.note_count()))
                            .size(11.5)
                            .color(pal.text_muted),
                    );
                    if let Some(ref msg) = self.state.status_message {
                        ui.separator();
                        ui.label(RichText::new(msg).size(11.5).color(pal.text_muted));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.state.active_path.is_some() {
                            let (words, chars) = self.state.active_note_word_count();
                            let bl = self
                                .state
                                .active_path
                                .as_deref()
                                .map(|p| self.state.graph.backlinks(p).len())
                                .unwrap_or(0);
                            ui.label(
                                RichText::new(format!(
                                    "{bl} backlinks  |  {words} words  |  {chars} chars"
                                ))
                                .size(11.5)
                                .color(pal.text_muted),
                            );
                        }
                    });
                });
            });
    }

    fn draw_modals(&mut self, ctx: &egui::Context, pal: Palette) {
        if self.state.show_quick_switcher {
            let mut open_path = None;
            let mut close = false;
            egui::Window::new("Quick Switcher")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_TOP, [0.0, 90.0])
                .fixed_size([460.0, 320.0])
                .show(ctx, |ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.state.quick_switcher_query)
                            .hint_text("Type a note name or path...")
                            .desired_width(ui.available_width()),
                    );
                    if self.state.quick_switcher_focus_needed {
                        resp.request_focus();
                        self.state.quick_switcher_focus_needed = false;
                    }
                    let matches: Vec<(String, String)> = self
                        .state
                        .quick_switcher_matches()
                        .into_iter()
                        .map(|f| (f.stem.clone(), f.relative_path.clone()))
                        .collect();

                    if ui.input(|i| i.key_pressed(Key::Enter)) {
                        if let Some((_, first)) = matches.first() {
                            open_path = Some(first.clone());
                            close = true;
                        }
                    }

                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (stem, path) in matches {
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Label::new(
                                            RichText::new(&stem).color(pal.text_primary),
                                        )
                                        .sense(egui::Sense::click()),
                                    )
                                    .clicked()
                                {
                                    open_path = Some(path.clone());
                                    close = true;
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&path)
                                                .size(11.0)
                                                .color(pal.text_muted),
                                        );
                                    },
                                );
                            });
                        }
                    });
                    ui.separator();
                    if ui.button("Close (Esc)").clicked() {
                        close = true;
                    }
                });
            if let Some(p) = open_path {
                self.state.open_file(&p);
            }
            if close {
                self.state.show_quick_switcher = false;
            }
        }

        if self.state.show_new_note_dialog {
            let mut create = false;
            let mut close = false;
            egui::Window::new("Create New Note")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .fixed_size([380.0, 120.0])
                .show(ctx, |ui| {
                    ui.label("Note path relative to vault (e.g. Notes/Ideas.md):");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.state.new_note_name)
                            .hint_text("Untitled.md")
                            .desired_width(ui.available_width()),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        create = true;
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            create = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if create {
                let name = self.state.new_note_name.clone();
                let title = name
                    .rsplit('/')
                    .next()
                    .unwrap_or(&name)
                    .trim_end_matches(".md");
                let initial = format!("# {title}\n\n");
                self.state.create_and_open_note(&name, &initial);
                self.state.show_new_note_dialog = false;
            } else if close {
                self.state.show_new_note_dialog = false;
            }
        }
    }
}

impl eframe::App for CottonMdApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let pal = apply_theme(ctx, self.state.dark_mode);
        self.handle_shortcuts(ctx);
        self.draw_top_bar(ctx, pal);
        self.draw_status_bar(ctx, pal);
        self.draw_left_sidebar(ctx, pal);
        self.draw_right_sidebar(ctx, pal);
        self.draw_center(ctx, pal);
        self.draw_modals(ctx, pal);
    }
}

fn draw_folder_tree(
    ui: &mut egui::Ui,
    node: &FolderTreeNode,
    active_path: Option<&str>,
    file_map: &std::collections::BTreeMap<String, cottonmd_vault::VaultFile>,
    pal: Palette,
    open_path: &mut Option<String>,
) {
    for (dir_name, child) in &node.subfolders {
        egui::CollapsingHeader::new(RichText::new(dir_name).color(pal.text_primary))
            .default_open(true)
            .show(ui, |ui| {
                draw_folder_tree(ui, child, active_path, file_map, pal, open_path);
            });
    }

    for rel in &node.files {
        let active = active_path == Some(rel.as_str());
        let (display, badge) = match file_map.get(rel) {
            Some(f) => {
                let tag = match f.kind {
                    EntryKind::Markdown => "",
                    EntryKind::Canvas => " [canvas]",
                    EntryKind::Base => " [base]",
                    EntryKind::Pdf => " [pdf]",
                    EntryKind::Attachment => " [file]",
                };
                (format!("{}{tag}", f.stem), f.kind)
            }
            None => (rel.clone(), EntryKind::Markdown),
        };
        let text = if active {
            RichText::new(display).color(pal.accent).strong()
        } else if badge == EntryKind::Markdown || badge == EntryKind::Canvas {
            RichText::new(display).color(pal.text_primary)
        } else {
            RichText::new(display).color(pal.text_muted)
        };
        if ui.selectable_label(active, text).clicked() {
            *open_path = Some(rel.clone());
        }
    }
}

fn main() -> eframe::Result<()> {
    let initial_vault = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with('-'))
        .map(PathBuf::from);

    let icon = egui::IconData {
        rgba: APP_ICON_RGBA.to_vec(),
        width: 64,
        height: 64,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CottonMD")
            .with_icon(Arc::new(icon))
            .with_inner_size([1320.0, 840.0])
            .with_min_inner_size([760.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CottonMD",
        options,
        Box::new(move |_cc| Ok(Box::new(CottonMdApp::new(initial_vault)))),
    )
}
