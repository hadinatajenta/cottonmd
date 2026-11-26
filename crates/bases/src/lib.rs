use std::collections::BTreeMap;

use cottonmd_markdown::ParsedNote;
use cottonmd_vault::VaultFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseViewKind {
    Table,
    Cards,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterPredicate {
    FolderPrefix(String),
    HasTag(String),
    PropertyEquals { key: String, value: String },
    PropertyExists(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseViewSpec {
    pub name: String,
    pub kind: BaseViewKind,
    pub columns: Vec<String>,
    pub filters: Vec<FilterPredicate>,
    pub sort_by: Option<String>,
    pub descending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseRow {
    pub path: String,
    pub title: String,
    pub values: BTreeMap<String, String>,
}

pub fn query_view(
    spec: &BaseViewSpec,
    items: &[(&VaultFile, &ParsedNote)],
) -> Vec<BaseRow> {
    let mut rows: Vec<BaseRow> = items
        .iter()
        .filter(|(file, note)| matches_all(&spec.filters, file, note))
        .map(|(file, note)| {
            let mut values = BTreeMap::new();
            for col in &spec.columns {
                let val = resolve_column(col, file, note);
                values.insert(col.clone(), val);
            }
            BaseRow {
                path: file.relative_path.clone(),
                title: file.stem.clone(),
                values,
            }
        })
        .collect();

    if let Some(ref sort_col) = spec.sort_by {
        rows.sort_by(|a, b| {
            let va = a.values.get(sort_col).map(String::as_str).unwrap_or("");
            let vb = b.values.get(sort_col).map(String::as_str).unwrap_or("");
            if spec.descending {
                vb.cmp(va)
            } else {
                va.cmp(vb)
            }
        });
    }

    rows
}

pub fn export_csv(columns: &[String], rows: &[BaseRow]) -> String {
    let mut out = String::new();
    let header: Vec<String> = columns.iter().map(|c| csv_escape(c)).collect();
    out.push_str(&header.join(","));
    out.push('\n');

    for row in rows {
        let cells: Vec<String> = columns
            .iter()
            .map(|c| {
                let val = row.values.get(c).map(String::as_str).unwrap_or("");
                csv_escape(val)
            })
            .collect();
        out.push_str(&cells.join(","));
        out.push('\n');
    }

    out
}

fn matches_all(filters: &[FilterPredicate], file: &VaultFile, note: &ParsedNote) -> bool {
    filters.iter().all(|pred| match pred {
        FilterPredicate::FolderPrefix(prefix) => file.relative_path.starts_with(prefix),
        FilterPredicate::HasTag(tag) => note.tags.iter().any(|t| t == tag),
        FilterPredicate::PropertyEquals { key, value } => {
            note.properties.get(key).map(String::as_str) == Some(value.as_str())
        }
        FilterPredicate::PropertyExists(key) => note.properties.contains_key(key),
    })
}

fn resolve_column(col: &str, file: &VaultFile, note: &ParsedNote) -> String {
    match col {
        "file.name" => file.stem.clone(),
        "file.path" => file.relative_path.clone(),
        "file.size" => file.size_bytes.to_string(),
        "file.tags" => note.tags.join(", "),
        other => note
            .properties
            .get(other)
            .cloned()
            .unwrap_or_default(),
    }
}

fn csv_escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cottonmd_markdown::parse_note;
    use cottonmd_vault::EntryKind;

    #[test]
    fn filters_sorts_and_exports_csv() {
        let f1 = VaultFile {
            relative_path: "Projects/Beta.md".into(),
            stem: "Beta".into(),
            extension: Some("md".into()),
            kind: EntryKind::Markdown,
            size_bytes: 120,
            modified: None,
            created: None,
        };
        let f2 = VaultFile {
            relative_path: "Projects/Alpha.md".into(),
            stem: "Alpha".into(),
            extension: Some("md".into()),
            kind: EntryKind::Markdown,
            size_bytes: 90,
            modified: None,
            created: None,
        };
        let n1 = parse_note("---\nstatus: active\n---\n#project");
        let n2 = parse_note("---\nstatus: active\n---\n#project");

        let spec = BaseViewSpec {
            name: "Active Projects".into(),
            kind: BaseViewKind::Table,
            columns: vec!["file.name".into(), "status".into()],
            filters: vec![
                FilterPredicate::FolderPrefix("Projects/".into()),
                FilterPredicate::PropertyEquals {
                    key: "status".into(),
                    value: "active".into(),
                },
            ],
            sort_by: Some("file.name".into()),
            descending: false,
        };

        let rows = query_view(&spec, &[(&f1, &n1), (&f2, &n2)]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "Alpha");
        let csv = export_csv(&spec.columns, &rows);
        assert_eq!(csv, "file.name,status\nAlpha,active\nBeta,active\n");
    }
}
