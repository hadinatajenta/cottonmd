#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedDocument {
    pub path: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub path: String,
    pub line: usize,
    pub snippet: String,
    pub score: u32,
}

#[derive(Debug, Default)]
pub struct SearchIndex {
    docs: Vec<IndexedDocument>,
}

impl SearchIndex {
    pub fn insert(&mut self, doc: IndexedDocument) {
        self.docs.push(doc);
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let needle = query.trim().to_ascii_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }

        let mut hits = Vec::new();
        for doc in &self.docs {
            if doc.title.to_ascii_lowercase().contains(&needle) {
                hits.push(SearchHit {
                    path: doc.path.clone(),
                    line: 0,
                    snippet: doc.title.clone(),
                    score: 100,
                });
            }
            for (idx, line) in doc.body.lines().enumerate() {
                if line.to_ascii_lowercase().contains(&needle) {
                    hits.push(SearchHit {
                        path: doc.path.clone(),
                        line: idx + 1,
                        snippet: line.trim().to_string(),
                        score: 50,
                    });
                    break;
                }
            }
        }

        hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
        hits.truncate(limit);
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_title_matches_above_body_matches() {
        let mut idx = SearchIndex::default();
        idx.insert(IndexedDocument {
            path: "Notes/Body.md".into(),
            title: "Body".into(),
            body: "Mentions rust graph engine".into(),
        });
        idx.insert(IndexedDocument {
            path: "Notes/Rust.md".into(),
            title: "Rust".into(),
            body: "Overview".into(),
        });

        let results = idx.search("rust", 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].path, "Notes/Rust.md");
        assert_eq!(results[1].path, "Notes/Body.md");
    }
}
