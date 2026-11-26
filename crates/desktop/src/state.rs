use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use cottonmd_canvas::CanvasDocument;
use cottonmd_graph::VaultGraph;
use cottonmd_markdown::{parse_note, ParsedNote};
use cottonmd_search::{IndexedDocument, SearchHit, SearchIndex};
use cottonmd_vault::{EntryKind, Vault, VaultFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftTab {
    Files,
    Search,
    Tags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightTab {
    Backlinks,
    Outline,
    Properties,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterMode {
    Edit,
    Split,
    Preview,
    Graph,
    Bases,
    Canvas,
}

#[derive(Default)]
pub struct FolderTreeNode {
    pub subfolders: BTreeMap<String, FolderTreeNode>,
    pub files: Vec<String>,
}

impl FolderTreeNode {
    fn insert(&mut self, relative_path: &str) {
        let parts: Vec<&str> = relative_path.split('/').collect();
        if parts.len() <= 1 {
            self.files.push(relative_path.to_string());
            return;
        }
        let mut cursor = self;
        for dir in &parts[..parts.len() - 1] {
            cursor = cursor
                .subfolders
                .entry((*dir).to_string())
                .or_default();
        }
        cursor.files.push(relative_path.to_string());
    }
}

pub struct AppState {
    pub dark_mode: bool,
    pub show_left_sidebar: bool,
    pub show_right_sidebar: bool,
    pub left_tab: LeftTab,
    pub right_tab: RightTab,
    pub center_mode: CenterMode,

    pub vault: Option<Vault>,
    pub files: Vec<VaultFile>,
    pub file_map: BTreeMap<String, VaultFile>,
    pub parsed_notes: BTreeMap<String, ParsedNote>,
    pub graph: VaultGraph,
    pub search_index: SearchIndex,
    pub folder_tree: FolderTreeNode,

    pub active_path: Option<String>,
    pub editor_buffer: String,
    pub saved_buffer: String,
    pub active_canvas: Option<CanvasDocument>,
    pub canvas_error: Option<String>,

    pub file_filter: String,
    pub search_query: String,
    pub search_results: Vec<SearchHit>,
    pub selected_tag: Option<String>,

    pub show_quick_switcher: bool,
    pub quick_switcher_query: String,
    pub quick_switcher_focus_needed: bool,

    pub show_new_note_dialog: bool,
    pub new_note_name: String,

    pub status_message: Option<String>,
    pub graph_revision: u64,
}

impl AppState {
    pub fn new(initial_vault: Option<PathBuf>) -> Self {
        let mut state = Self {
            dark_mode: true,
            show_left_sidebar: true,
            show_right_sidebar: true,
            left_tab: LeftTab::Files,
            right_tab: RightTab::Backlinks,
            center_mode: CenterMode::Split,
            vault: None,
            files: Vec::new(),
            file_map: BTreeMap::new(),
            parsed_notes: BTreeMap::new(),
            graph: VaultGraph::default(),
            search_index: SearchIndex::default(),
            folder_tree: FolderTreeNode::default(),
            active_path: None,
            editor_buffer: String::new(),
            saved_buffer: String::new(),
            active_canvas: None,
            canvas_error: None,
            file_filter: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            selected_tag: None,
            show_quick_switcher: false,
            quick_switcher_query: String::new(),
            quick_switcher_focus_needed: false,
            show_new_note_dialog: false,
            new_note_name: String::new(),
            status_message: None,
            graph_revision: 0,
        };

        let candidate = initial_vault.or_else(load_last_vault_path);

        if let Some(path) = candidate {
            if path.is_dir() {
                state.open_vault(&path);
            }
        }

        state
    }

    pub fn is_dirty(&self) -> bool {
        self.active_path.is_some() && self.editor_buffer != self.saved_buffer
    }

    pub fn open_vault(&mut self, dir: &Path) {
        match Vault::open(dir) {
            Ok(v) => {
                save_last_vault_path(v.root());
                self.vault = Some(v);
                self.active_path = None;
                self.editor_buffer.clear();
                self.saved_buffer.clear();
                self.active_canvas = None;
                self.canvas_error = None;
                self.selected_tag = None;
                self.reindex_vault();

                let first_md = self
                    .files
                    .iter()
                    .find(|f| f.relative_path.eq_ignore_ascii_case("Start here.md"))
                    .or_else(|| self.files.iter().find(|f| f.kind == EntryKind::Markdown))
                    .map(|f| f.relative_path.clone());
                if let Some(path) = first_md {
                    self.open_file(&path);
                }
                self.status_message = Some(format!("Opened vault ({} files)", self.files.len()));
            }
            Err(err) => {
                self.status_message = Some(format!("Failed to open vault: {err}"));
            }
        }
    }

    pub fn reindex_vault(&mut self) {
        let Some(ref vault) = self.vault else {
            return;
        };
        let files = match vault.list_files() {
            Ok(f) => f,
            Err(err) => {
                self.status_message = Some(format!("Failed to scan vault: {err}"));
                return;
            }
        };

        let mut file_map = BTreeMap::new();
        let mut parsed_notes = BTreeMap::new();
        let mut search_index = SearchIndex::default();
        let mut tree = FolderTreeNode::default();

        for file in &files {
            file_map.insert(file.relative_path.clone(), file.clone());
            tree.insert(&file.relative_path);

            if file.kind == EntryKind::Markdown {
                let text = if self.active_path.as_deref() == Some(file.relative_path.as_str())
                    && self.is_dirty()
                {
                    self.editor_buffer.clone()
                } else {
                    vault
                        .read_to_string(&file.relative_path)
                        .unwrap_or_default()
                };
                let parsed = parse_note(&text);
                parsed_notes.insert(file.relative_path.clone(), parsed);
                search_index.insert(IndexedDocument {
                    path: file.relative_path.clone(),
                    title: file.stem.clone(),
                    body: text,
                });
            }
        }

        let graph = VaultGraph::build(
            &files,
            parsed_notes.iter().map(|(k, v)| (k.as_str(), v)),
        );

        self.files = files;
        self.file_map = file_map;
        self.parsed_notes = parsed_notes;
        self.graph = graph;
        self.search_index = search_index;
        self.folder_tree = tree;
        self.graph_revision = self.graph_revision.wrapping_add(1);
        self.refresh_search();
    }

    pub fn open_file(&mut self, relative_path: &str) {
        if self.vault.is_none() {
            return;
        }
        if self.is_dirty() {
            self.save_active_file();
        }
        let kind = self
            .file_map
            .get(relative_path)
            .map(|f| f.kind)
            .unwrap_or(EntryKind::Markdown);

        let Some(ref vault) = self.vault else {
            return;
        };
        match vault.read_to_string(relative_path) {
            Ok(text) => {
                self.active_path = Some(relative_path.to_string());
                self.editor_buffer = text.clone();
                self.saved_buffer = text.clone();
                if kind == EntryKind::Canvas {
                    match CanvasDocument::from_json_str(&text) {
                        Ok(doc) => {
                            self.active_canvas = Some(doc);
                            self.canvas_error = None;
                            self.center_mode = CenterMode::Canvas;
                        }
                        Err(e) => {
                            self.active_canvas = None;
                            self.canvas_error = Some(e);
                            self.center_mode = CenterMode::Edit;
                        }
                    }
                } else {
                    self.active_canvas = None;
                    self.canvas_error = None;
                    if matches!(self.center_mode, CenterMode::Graph | CenterMode::Bases | CenterMode::Canvas) {
                        self.center_mode = CenterMode::Split;
                    }
                }
            }
            Err(err) => {
                self.status_message = Some(format!("Cannot read {relative_path}: {err}"));
            }
        }
    }

    pub fn open_wikilink(&mut self, target: &str) {
        if let Some(resolved) = self.resolve_link_target(target) {
            self.open_file(&resolved);
            return;
        }
        let mut candidate = target.trim().to_string();
        if candidate.is_empty() {
            return;
        }
        if !candidate.to_ascii_lowercase().ends_with(".md")
            && !candidate.to_ascii_lowercase().ends_with(".canvas")
        {
            candidate.push_str(".md");
        }
        self.create_and_open_note(&candidate, &format!("# {}\n\n", target.trim()));
    }

    pub fn resolve_link_target(&self, target: &str) -> Option<String> {
        let key = target.trim().to_ascii_lowercase();
        if key.is_empty() {
            return None;
        }
        for f in &self.files {
            let rel = f.relative_path.to_ascii_lowercase();
            if rel == key || rel.strip_suffix(".md") == Some(key.as_str()) {
                return Some(f.relative_path.clone());
            }
        }
        for f in &self.files {
            if f.stem.to_ascii_lowercase() == key {
                return Some(f.relative_path.clone());
            }
        }
        None
    }

    pub fn create_and_open_note(&mut self, raw_name: &str, initial_content: &str) {
        let Some(ref vault) = self.vault else {
            return;
        };
        let normalized = raw_name.trim().replace('\\', "/");
        let trimmed = normalized.trim_start_matches('/');
        if trimmed.is_empty() {
            return;
        }
        let rel = if trimmed.contains('.') {
            trimmed.to_string()
        } else {
            format!("{trimmed}.md")
        };
        if let Err(err) = vault.write_string(&rel, initial_content) {
            self.status_message = Some(format!("Failed to create {rel}: {err}"));
            return;
        }
        self.reindex_vault();
        self.open_file(&rel);
        self.status_message = Some(format!("Created {rel}"));
    }

    pub fn save_active_file(&mut self) {
        let (Some(ref vault), Some(ref path)) = (&self.vault, &self.active_path) else {
            return;
        };
        match vault.write_string(path, &self.editor_buffer) {
            Ok(()) => {
                self.saved_buffer = self.editor_buffer.clone();
                self.status_message = Some(format!("Saved {path}"));
                self.reindex_vault();
            }
            Err(err) => {
                self.status_message = Some(format!("Save failed: {err}"));
            }
        }
    }

    pub fn toggle_task_line(&mut self, line_number_1based: usize) {
        let mut lines: Vec<String> = self.editor_buffer.lines().map(String::from).collect();
        let idx = line_number_1based.saturating_sub(1);
        if let Some(line) = lines.get_mut(idx) {
            if let Some(pos) = line.find("[ ]") {
                line.replace_range(pos..pos + 3, "[x]");
            } else if let Some(pos) = line.find("[x]").or_else(|| line.find("[X]")) {
                line.replace_range(pos..pos + 3, "[ ]");
            }
            let trailing_nl = self.editor_buffer.ends_with('\n');
            self.editor_buffer = lines.join("\n");
            if trailing_nl {
                self.editor_buffer.push('\n');
            }
            self.save_active_file();
        }
    }

    pub fn refresh_search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_results.clear();
        } else {
            self.search_results = self.search_index.search(&self.search_query, 80);
        }
    }

    pub fn active_parsed(&self) -> Option<ParsedNote> {
        let path = self.active_path.as_deref()?;
        if self.is_dirty() {
            Some(parse_note(&self.editor_buffer))
        } else {
            self.parsed_notes.get(path).cloned()
        }
    }

    pub fn quick_switcher_matches(&self) -> Vec<&VaultFile> {
        let q = self.quick_switcher_query.trim().to_ascii_lowercase();
        let mut items: Vec<&VaultFile> = self
            .files
            .iter()
            .filter(|f| {
                q.is_empty()
                    || f.stem.to_ascii_lowercase().contains(&q)
                    || f.relative_path.to_ascii_lowercase().contains(&q)
            })
            .collect();
        items.sort_by(|a, b| {
            let a_exact = a.stem.to_ascii_lowercase().starts_with(&q);
            let b_exact = b.stem.to_ascii_lowercase().starts_with(&q);
            b_exact
                .cmp(&a_exact)
                .then_with(|| a.relative_path.cmp(&b.relative_path))
        });
        items.truncate(30);
        items
    }

    pub fn active_note_word_count(&self) -> (usize, usize) {
        let words = self.editor_buffer.split_whitespace().count();
        let chars = self.editor_buffer.chars().count();
        (words, chars)
    }
}

pub fn resolve_link_in_files(files: &[VaultFile], target: &str) -> Option<String> {
    let key = target.trim().to_ascii_lowercase();
    if key.is_empty() {
        return None;
    }
    for f in files {
        let rel = f.relative_path.to_ascii_lowercase();
        if rel == key || rel.strip_suffix(".md") == Some(key.as_str()) {
            return Some(f.relative_path.clone());
        }
    }
    for f in files {
        if f.stem.to_ascii_lowercase() == key {
            return Some(f.relative_path.clone());
        }
    }
    None
}

fn config_file_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("APPDATA").map(PathBuf::from).or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(|h| PathBuf::from(h).join("AppData").join("Roaming"))
        })?;
        return Some(base.join("CottonMD").join("last_vault.txt"));
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")?;
        return Some(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("CottonMD")
                .join("last_vault.txt"),
        );
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        return Some(base.join("cottonmd").join("last_vault.txt"));
    }
}

fn load_last_vault_path() -> Option<PathBuf> {
    let path = config_file_path()?;
    let raw = fs::read_to_string(path).ok()?;
    let p = PathBuf::from(raw.trim());
    p.is_dir().then_some(p)
}

fn save_last_vault_path(vault_root: &Path) {
    if let Some(cfg) = config_file_path() {
        if let Some(parent) = cfg.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(cfg, vault_root.to_string_lossy().as_bytes());
    }
}
