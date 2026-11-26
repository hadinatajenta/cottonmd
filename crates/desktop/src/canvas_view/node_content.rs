use cottonmd_canvas::CanvasNodeKind;

const MAX_BODY_CHARS: usize = 600;

pub struct NodeContent {
    pub icon: &'static str,
    pub kind_label: &'static str,
    pub title: String,
    pub body: String,
    pub linked_file: Option<String>,
}

pub fn describe_node(kind: &CanvasNodeKind) -> NodeContent {
    match kind {
        CanvasNodeKind::Text { text } => NodeContent {
            icon: egui_phosphor::regular::TEXT_T,
            kind_label: "Text",
            title: first_line(text).to_owned(),
            body: truncate_chars(text, MAX_BODY_CHARS),
            linked_file: None,
        },
        CanvasNodeKind::File { file, .. } => NodeContent {
            icon: egui_phosphor::regular::FILE_TEXT,
            kind_label: "File",
            title: file_stem(file).to_owned(),
            body: truncate_chars(file, MAX_BODY_CHARS),
            linked_file: Some(file.clone()),
        },
        CanvasNodeKind::Link { url } => NodeContent {
            icon: egui_phosphor::regular::LINK_SIMPLE,
            kind_label: "Link",
            title: url_host(url).to_owned(),
            body: truncate_chars(url, MAX_BODY_CHARS),
            linked_file: None,
        },
        CanvasNodeKind::Group { label } => NodeContent {
            icon: egui_phosphor::regular::SQUARES_FOUR,
            kind_label: "Group",
            title: label.clone().unwrap_or_else(|| "Group".to_owned()),
            body: String::new(),
            linked_file: None,
        },
    }
}

pub fn is_group(kind: &CanvasNodeKind) -> bool {
    matches!(kind, CanvasNodeKind::Group { .. })
}

fn first_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("Untitled")
}

fn file_stem(path: &str) -> &str {
    let file_name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    if stem.is_empty() {
        file_name
    } else {
        stem
    }
}

fn url_host(url: &str) -> &str {
    let without_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    without_scheme
        .split(['/', '?', '#'])
        .next()
        .filter(|host| !host.is_empty())
        .unwrap_or(url)
}

pub fn truncate_chars(text: &str, max_chars: usize) -> String {
    match text.char_indices().nth(max_chars) {
        Some((byte_index, _)) => format!("{}...", &text[..byte_index]),
        None => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_respects_char_boundaries() {
        assert_eq!(truncate_chars("héllo wörld", 5), "héllo...");
        assert_eq!(truncate_chars("short", 10), "short");
        assert_eq!(truncate_chars("", 3), "");
    }

    #[test]
    fn file_node_title_is_the_file_stem() {
        let content = describe_node(&CanvasNodeKind::File {
            file: "notes/ideas/plan.md".into(),
            subpath: None,
        });
        assert_eq!(content.title, "plan");
        assert_eq!(content.linked_file.as_deref(), Some("notes/ideas/plan.md"));
    }

    #[test]
    fn link_node_title_is_the_host() {
        let content = describe_node(&CanvasNodeKind::Link {
            url: "https://example.com/path?q=1".into(),
        });
        assert_eq!(content.title, "example.com");
    }

    #[test]
    fn empty_text_node_gets_placeholder_title() {
        let content = describe_node(&CanvasNodeKind::Text {
            text: "  \n ".into(),
        });
        assert_eq!(content.title, "Untitled");
    }

    #[test]
    fn dotfile_without_stem_keeps_its_name() {
        assert_eq!(file_stem(".hidden"), ".hidden");
    }
}
