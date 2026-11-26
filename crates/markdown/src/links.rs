const NOTE_EXTENSIONS: [&str; 4] = [".md", ".markdown", ".canvas", ".base"];
const EXTERNAL_SCHEMES: [&str; 3] = ["mailto:", "tel:", "data:"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStyle {
    Wiki,
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLink {
    pub raw: String,
    pub target: String,
    pub subpath: Option<String>,
    pub alias: Option<String>,
    pub is_embed: bool,
    pub line: usize,
    pub style: LinkStyle,
}

pub fn is_external_link(href: &str) -> bool {
    let lowered = href.trim().to_ascii_lowercase();
    lowered.contains("://") || EXTERNAL_SCHEMES.iter().any(|scheme| lowered.starts_with(scheme))
}

pub fn resolve_relative_link(source_path: &str, href: &str) -> Option<String> {
    let link_path = strip_link_decorations(href);
    if link_path.is_empty() || is_external_link(link_path) {
        return None;
    }
    let decoded = percent_decode(link_path);
    let (base_segments, relative_part) = match decoded.strip_prefix('/') {
        Some(rooted) => (Vec::new(), rooted.to_string()),
        None => (parent_segments(source_path), decoded),
    };
    normalize_segments(base_segments, &relative_part)
}

pub(crate) fn extract_wikilinks(line: &str, line_no: usize, out: &mut Vec<WikiLink>) {
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_code = false;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            in_code = !in_code;
            i += 1;
            continue;
        }
        if !in_code && i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let start = i + 2;
            if let Some(rel_end) = line[start..].find("]]") {
                let raw = &line[start..start + rel_end];
                let is_embed = i > 0 && bytes[i - 1] == b'!';
                if let Some(link) = build_wikilink(raw, is_embed, line_no) {
                    out.push(link);
                }
                i = start + rel_end + 2;
                continue;
            }
        }
        i += 1;
    }
}

fn build_wikilink(raw: &str, is_embed: bool, line_no: usize) -> Option<WikiLink> {
    if raw.is_empty() || raw.contains('[') {
        return None;
    }
    let (link_part, alias) = match raw.split_once('|') {
        Some((link_text, alias_text)) => (link_text.trim(), Some(alias_text.trim().to_string())),
        None => (raw.trim(), None),
    };
    let (target, subpath) = match link_part.split_once('#') {
        Some((target_text, section)) => {
            (target_text.trim().to_string(), Some(section.trim().to_string()))
        }
        None => (link_part.to_string(), None),
    };
    Some(WikiLink {
        raw: raw.to_string(),
        target,
        subpath,
        alias,
        is_embed,
        line: line_no,
        style: LinkStyle::Wiki,
    })
}

pub(crate) fn extract_markdown_links(line: &str, line_no: usize, out: &mut Vec<WikiLink>) {
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_code = false;
    while i < bytes.len() {
        match bytes[i] {
            b'`' => in_code = !in_code,
            b'[' if !in_code && !is_wikilink_bracket(bytes, i) => {
                if let Some((link, consumed_until)) = parse_markdown_link(line, i, line_no) {
                    if let Some(link) = link {
                        out.push(link);
                    }
                    i = consumed_until;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
}

fn is_wikilink_bracket(bytes: &[u8], index: usize) -> bool {
    bytes.get(index + 1) == Some(&b'[') || (index > 0 && bytes[index - 1] == b'[')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkdownLinkSpan<'a> {
    pub display: &'a str,
    pub href: &'a str,
    pub is_image: bool,
    pub end: usize,
}

pub fn parse_markdown_link_span(line: &str, open_bracket: usize) -> Option<MarkdownLinkSpan<'_>> {
    if line.as_bytes().get(open_bracket) != Some(&b'[') {
        return None;
    }
    let close_bracket = find_matching(line, open_bracket, b'[', b']')?;
    if line.as_bytes().get(close_bracket + 1) != Some(&b'(') {
        return None;
    }
    let open_paren = close_bracket + 1;
    let close_paren = find_matching(line, open_paren, b'(', b')')?;
    Some(MarkdownLinkSpan {
        display: &line[open_bracket + 1..close_bracket],
        href: &line[open_paren + 1..close_paren],
        is_image: open_bracket > 0 && line.as_bytes()[open_bracket - 1] == b'!',
        end: close_paren + 1,
    })
}

fn parse_markdown_link(
    line: &str,
    open_bracket: usize,
    line_no: usize,
) -> Option<(Option<WikiLink>, usize)> {
    let span = parse_markdown_link_span(line, open_bracket)?;
    let link = build_markdown_link(span.display, span.href, span.is_image, line_no);
    Some((link, span.end))
}

fn build_markdown_link(display: &str, href: &str, is_embed: bool, line_no: usize) -> Option<WikiLink> {
    let link_path = strip_link_decorations(href);
    if link_path.is_empty() || is_external_link(link_path) || !is_note_path(link_path) {
        return None;
    }
    let subpath = href
        .split_once('#')
        .map(|(_, section)| section.trim_end_matches('>').trim().to_string())
        .filter(|section| !section.is_empty());
    Some(WikiLink {
        raw: format!("[{display}]({href})"),
        target: link_path.to_string(),
        subpath,
        alias: Some(display.trim().to_string()).filter(|text| !text.is_empty()),
        is_embed,
        line: line_no,
        style: LinkStyle::Markdown,
    })
}

fn find_matching(line: &str, open_index: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut depth = 0usize;
    for (index, byte) in bytes.iter().enumerate().skip(open_index) {
        if *byte == open {
            depth += 1;
        } else if *byte == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn strip_link_decorations(href: &str) -> &str {
    let trimmed = href.trim();
    let without_title = match trimmed.strip_prefix('<') {
        Some(angled) => angled.split('>').next().unwrap_or_default(),
        None => trimmed.split_whitespace().next().unwrap_or_default(),
    };
    without_title
        .split(['#', '?'])
        .next()
        .unwrap_or_default()
        .trim()
}

fn is_note_path(link_path: &str) -> bool {
    let lowered = percent_decode(link_path).to_ascii_lowercase();
    NOTE_EXTENSIONS.iter().any(|extension| lowered.ends_with(extension))
}

fn parent_segments(source_path: &str) -> Vec<String> {
    let mut segments: Vec<String> = source_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_owned)
        .collect();
    segments.pop();
    segments
}

fn normalize_segments(mut segments: Vec<String>, relative_part: &str) -> Option<String> {
    for segment in relative_part.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            name => segments.push(name.to_string()),
        }
    }
    (!segments.is_empty()).then(|| segments.join("/"))
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex_pair = bytes.get(i + 1..i + 3).and_then(|pair| std::str::from_utf8(pair).ok());
        match (bytes[i], hex_pair.and_then(|pair| u8::from_str_radix(pair, 16).ok())) {
            (b'%', Some(decoded_byte)) => {
                decoded.push(decoded_byte);
                i += 3;
            }
            (byte, _) => {
                decoded.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn markdown_links(line: &str) -> Vec<WikiLink> {
        let mut links = Vec::new();
        extract_markdown_links(line, 1, &mut links);
        links
    }

    #[test]
    fn extracts_relative_note_links_with_display_and_section() {
        let links = markdown_links("| [sprint_32](../sprint_32/backend-work.md#tickets) |");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "../sprint_32/backend-work.md");
        assert_eq!(links[0].subpath.as_deref(), Some("tickets"));
        assert_eq!(links[0].alias.as_deref(), Some("sprint_32"));
        assert_eq!(links[0].style, LinkStyle::Markdown);
    }

    #[test]
    fn ignores_external_urls_images_code_and_wikilinks() {
        let line = "[Jira](https://jira.example.com/browse/X-1) ![chart](img.png) `[a](b.md)` [[Wiki]] [mail](mailto:a@b.c)";
        assert!(markdown_links(line).is_empty());
    }

    #[test]
    fn handles_brackets_in_display_angle_paths_and_titles() {
        let links = markdown_links("See [the `[x]` doc](<My Notes/a b.md> \"Title\") and [c](c.md 'title').");
        let targets: Vec<&str> = links.iter().map(|link| link.target.as_str()).collect();
        assert_eq!(targets, vec!["My Notes/a b.md", "c.md"]);
    }

    #[test]
    fn resolves_relative_paths_against_the_source_note() {
        let source = "docs/sprint_36/docs/CRMMS-66241/README.md";
        assert_eq!(
            resolve_relative_link(source, "../../../../_library/repos/zenith.md").as_deref(),
            Some("_library/repos/zenith.md")
        );
        assert_eq!(
            resolve_relative_link("docs/review/README.md", "./a%20b.md#part").as_deref(),
            Some("docs/review/a b.md")
        );
        assert_eq!(
            resolve_relative_link("docs/README.md", "/_library/AGENTS.md").as_deref(),
            Some("_library/AGENTS.md")
        );
    }

    #[test]
    fn rejects_paths_escaping_the_vault_and_external_links() {
        assert_eq!(resolve_relative_link("README.md", "../outside.md"), None);
        assert_eq!(resolve_relative_link("a/README.md", "https://x.dev/a.md"), None);
        assert_eq!(resolve_relative_link("a/README.md", "#section"), None);
    }

    #[test]
    fn percent_decode_keeps_invalid_sequences() {
        assert_eq!(percent_decode("a%20b%zz%"), "a b%zz%");
    }
}
