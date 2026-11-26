use std::collections::{BTreeMap, BTreeSet};

use cottonmd_markdown::ParsedNote;
use cottonmd_vault::{EntryKind, VaultFile};

#[derive(Debug, Clone, Default)]
pub struct VaultGraph {
    notes: BTreeSet<String>,
    outgoing: BTreeMap<String, BTreeSet<String>>,
    incoming: BTreeMap<String, BTreeSet<String>>,
    unresolved: BTreeMap<String, BTreeSet<String>>,
    tags: BTreeMap<String, BTreeSet<String>>,
}

impl VaultGraph {
    pub fn build<'a>(
        files: &[VaultFile],
        parsed_notes: impl IntoIterator<Item = (&'a str, &'a ParsedNote)>,
    ) -> Self {
        let mut graph = Self::default();
        let mut by_stem: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut by_rel_lower: BTreeMap<String, String> = BTreeMap::new();
        let mut by_no_ext_lower: BTreeMap<String, String> = BTreeMap::new();

        for file in files {
            let rel_lower = file.relative_path.to_ascii_lowercase();
            by_rel_lower.insert(rel_lower.clone(), file.relative_path.clone());
            if let Some(stripped) = rel_lower.strip_suffix(".md") {
                by_no_ext_lower.insert(stripped.to_string(), file.relative_path.clone());
            }
            by_stem
                .entry(file.stem.to_ascii_lowercase())
                .or_default()
                .push(file.relative_path.clone());

            if file.kind == EntryKind::Markdown {
                graph.notes.insert(file.relative_path.clone());
            }
        }

        for (source_path, parsed) in parsed_notes {
            graph.notes.insert(source_path.to_string());
            for tag in &parsed.tags {
                graph
                    .tags
                    .entry(tag.clone())
                    .or_default()
                    .insert(source_path.to_string());
            }

            for link in &parsed.links {
                if link.target.is_empty() {
                    continue;
                }
                let key = link.target.to_ascii_lowercase();
                let resolved = by_rel_lower
                    .get(&key)
                    .or_else(|| by_no_ext_lower.get(&key))
                    .or_else(|| by_stem.get(&key).and_then(|v| v.first()))
                    .cloned();

                match resolved {
                    Some(target_path) => {
                        graph
                            .outgoing
                            .entry(source_path.to_string())
                            .or_default()
                            .insert(target_path.clone());
                        graph
                            .incoming
                            .entry(target_path)
                            .or_default()
                            .insert(source_path.to_string());
                    }
                    None => {
                        graph
                            .unresolved
                            .entry(source_path.to_string())
                            .or_default()
                            .insert(link.target.clone());
                    }
                }
            }
        }

        graph
    }

    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    pub fn outgoing_links(&self, note_path: &str) -> Vec<&str> {
        self.outgoing
            .get(note_path)
            .map(|set| set.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    pub fn backlinks(&self, note_path: &str) -> Vec<&str> {
        self.incoming
            .get(note_path)
            .map(|set| set.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    pub fn orphans(&self) -> Vec<&str> {
        self.notes
            .iter()
            .filter(|n| self.incoming.get(*n).is_none_or(|s| s.is_empty()))
            .map(String::as_str)
            .collect()
    }

    pub fn deadends(&self) -> Vec<&str> {
        self.notes
            .iter()
            .filter(|n| self.outgoing.get(*n).is_none_or(|s| s.is_empty()))
            .map(String::as_str)
            .collect()
    }

    pub fn unresolved_links(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.unresolved
    }

    pub fn tags(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cottonmd_markdown::parse_note;

    fn md_file(path: &str, stem: &str) -> VaultFile {
        VaultFile {
            relative_path: path.to_string(),
            stem: stem.to_string(),
            extension: Some("md".to_string()),
            kind: EntryKind::Markdown,
            size_bytes: 10,
            modified: None,
            created: None,
        }
    }

    #[test]
    fn resolves_backlinks_orphans_deadends_and_unresolved() {
        let files = vec![
            md_file("Start.md", "Start"),
            md_file("Guide.md", "Guide"),
            md_file("Lonely.md", "Lonely"),
        ];
        let p_start = parse_note("Link to [[Guide]] and [[Missing Note]] #index");
        let p_guide = parse_note("No outgoing links here.");
        let p_lonely = parse_note("Isolated.");

        let g = VaultGraph::build(
            &files,
            [
                ("Start.md", &p_start),
                ("Guide.md", &p_guide),
                ("Lonely.md", &p_lonely),
            ],
        );

        assert_eq!(g.outgoing_links("Start.md"), vec!["Guide.md"]);
        assert_eq!(g.backlinks("Guide.md"), vec!["Start.md"]);
        assert_eq!(g.orphans(), vec!["Lonely.md", "Start.md"]);
        assert_eq!(g.deadends(), vec!["Guide.md", "Lonely.md"]);
        assert!(g.unresolved_links()["Start.md"].contains("Missing Note"));
        assert!(g.tags()["index"].contains("Start.md"));
    }
}
