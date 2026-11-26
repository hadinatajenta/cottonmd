use std::collections::{BTreeMap, BTreeSet};

use cottonmd_vault::{EntryKind, VaultFile};

#[derive(Default)]
pub struct FolderTreeNode {
    pub subfolders: BTreeMap<String, FolderTreeNode>,
    pub files: Vec<String>,
}

impl FolderTreeNode {
    pub fn insert(&mut self, relative_path: &str) {
        let mut cursor = self;
        if let Some((folder_path, _)) = relative_path.rsplit_once('/') {
            for folder in folder_path.split('/') {
                cursor = cursor.subfolders.entry(folder.to_string()).or_default();
            }
        }
        cursor.files.push(relative_path.to_string());
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
            + self
                .subfolders
                .values()
                .map(FolderTreeNode::file_count)
                .sum::<usize>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeEntryKind {
    Folder { path: String, is_expanded: bool, file_count: usize },
    File { relative_path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    pub depth: usize,
    pub name: String,
    pub kind: TreeEntryKind,
}

pub fn flatten_tree(root: &FolderTreeNode, collapsed_folders: &BTreeSet<String>) -> Vec<TreeEntry> {
    let mut entries = Vec::new();
    collect_entries(root, "", 0, collapsed_folders, &mut entries);
    entries
}

fn collect_entries(
    node: &FolderTreeNode,
    parent_path: &str,
    depth: usize,
    collapsed_folders: &BTreeSet<String>,
    entries: &mut Vec<TreeEntry>,
) {
    for (name, child) in &node.subfolders {
        let path = if parent_path.is_empty() {
            name.clone()
        } else {
            format!("{parent_path}/{name}")
        };
        let is_expanded = !collapsed_folders.contains(&path);
        entries.push(TreeEntry {
            depth,
            name: name.clone(),
            kind: TreeEntryKind::Folder {
                path: path.clone(),
                is_expanded,
                file_count: child.file_count(),
            },
        });
        if is_expanded {
            collect_entries(child, &path, depth + 1, collapsed_folders, entries);
        }
    }
    for relative_path in &node.files {
        entries.push(TreeEntry {
            depth,
            name: file_display_name(relative_path),
            kind: TreeEntryKind::File { relative_path: relative_path.clone() },
        });
    }
}

pub fn toggle_collapsed(collapsed_folders: &mut BTreeSet<String>, folder_path: &str) {
    if !collapsed_folders.remove(folder_path) {
        collapsed_folders.insert(folder_path.to_string());
    }
}

pub fn file_display_name(relative_path: &str) -> String {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    file_name
        .strip_suffix(".md")
        .unwrap_or(file_name)
        .to_string()
}

pub fn parent_folder(relative_path: &str) -> &str {
    relative_path.rsplit_once('/').map_or("", |(parent, _)| parent)
}

pub fn filter_files<'a>(files: &'a [VaultFile], query: &str) -> Vec<&'a VaultFile> {
    let normalized_query = query.trim().to_ascii_lowercase();
    files
        .iter()
        .filter(|file| file.relative_path.to_ascii_lowercase().contains(&normalized_query))
        .collect()
}

pub fn kind_label(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Markdown => "",
        EntryKind::Canvas => "canvas",
        EntryKind::Base => "base",
        EntryKind::Pdf => "pdf",
        EntryKind::Attachment => "file",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_tree(paths: &[&str]) -> FolderTreeNode {
        let mut tree = FolderTreeNode::default();
        for path in paths {
            tree.insert(path);
        }
        tree
    }

    fn vault_file(relative_path: &str) -> VaultFile {
        VaultFile {
            relative_path: relative_path.to_string(),
            stem: file_display_name(relative_path),
            extension: Some("md".to_string()),
            kind: EntryKind::Markdown,
            size_bytes: 0,
            modified: None,
            created: None,
        }
    }

    #[test]
    fn insert_groups_files_under_nested_folders() {
        let tree = build_tree(&["Root.md", "Notes/Ideas.md", "Notes/Deep/Plan.md"]);
        assert_eq!(tree.files, vec!["Root.md"]);
        assert_eq!(tree.file_count(), 3);
        assert_eq!(tree.subfolders["Notes"].files, vec!["Notes/Ideas.md"]);
        assert_eq!(
            tree.subfolders["Notes"].subfolders["Deep"].files,
            vec!["Notes/Deep/Plan.md"]
        );
    }

    #[test]
    fn flatten_lists_folders_before_files_with_depth() {
        let tree = build_tree(&["Root.md", "Notes/Ideas.md"]);
        let entries = flatten_tree(&tree, &BTreeSet::new());
        let summary: Vec<(usize, &str)> = entries
            .iter()
            .map(|entry| (entry.depth, entry.name.as_str()))
            .collect();
        assert_eq!(summary, vec![(0, "Notes"), (1, "Ideas"), (0, "Root")]);
    }

    #[test]
    fn flatten_hides_children_of_collapsed_folders() {
        let tree = build_tree(&["Notes/Deep/Plan.md", "Notes/Ideas.md"]);
        let collapsed: BTreeSet<String> = ["Notes".to_string()].into();
        let entries = flatten_tree(&tree, &collapsed);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].kind,
            TreeEntryKind::Folder {
                path: "Notes".to_string(),
                is_expanded: false,
                file_count: 2
            }
        );
    }

    #[test]
    fn flatten_uses_full_path_for_nested_folder_keys() {
        let tree = build_tree(&["A/B/C.md"]);
        let collapsed: BTreeSet<String> = ["A/B".to_string()].into();
        let entries = flatten_tree(&tree, &collapsed);
        assert_eq!(entries.len(), 2);
        assert!(matches!(
            &entries[1].kind,
            TreeEntryKind::Folder { path, is_expanded: false, .. } if path == "A/B"
        ));
    }

    #[test]
    fn filter_matches_case_insensitively_on_relative_path() {
        let files = vec![vault_file("Notes/Ideas.md"), vault_file("Journal/Day.md")];
        let matches = filter_files(&files, "  ideas ");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].relative_path, "Notes/Ideas.md");
        assert_eq!(filter_files(&files, "notes/").len(), 1);
    }

    #[test]
    fn toggle_collapsed_flips_membership() {
        let mut collapsed = BTreeSet::new();
        toggle_collapsed(&mut collapsed, "Notes");
        assert!(collapsed.contains("Notes"));
        toggle_collapsed(&mut collapsed, "Notes");
        assert!(collapsed.is_empty());
    }

    #[test]
    fn parent_folder_of_root_file_is_empty() {
        assert_eq!(parent_folder("Root.md"), "");
        assert_eq!(parent_folder("A/B/C.md"), "A/B");
    }
}
