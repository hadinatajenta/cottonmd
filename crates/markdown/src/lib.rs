use std::collections::BTreeMap;

mod links;

pub use links::{
    is_external_link, parse_markdown_link_span, resolve_relative_link, LinkStyle, MarkdownLinkSpan,
    WikiLink,
};
use links::{extract_markdown_links, extract_wikilinks};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub status: char,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedNote {
    pub properties: BTreeMap<String, String>,
    pub aliases: Vec<String>,
    pub headings: Vec<Heading>,
    pub links: Vec<WikiLink>,
    pub tags: Vec<String>,
    pub tasks: Vec<TaskItem>,
}

pub fn parse_note(content: &str) -> ParsedNote {
    let mut parsed = ParsedNote::default();
    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;

    if lines.first().map(|l| l.trim()) == Some("---") {
        idx = 1;
        let mut current_list_key: Option<String> = None;
        while idx < lines.len() {
            let line = lines[idx];
            if line.trim() == "---" || line.trim() == "..." {
                idx += 1;
                break;
            }
            let trimmed = line.trim();
            if let Some(item) = trimmed.strip_prefix("- ") {
                if let Some(ref key) = current_list_key {
                    let clean = strip_quotes(item.trim());
                    if key == "tags" || key == "tag" {
                        push_unique(&mut parsed.tags, normalize_tag(clean));
                    } else if key == "aliases" || key == "alias" {
                        push_unique(&mut parsed.aliases, clean.to_string());
                    } else {
                        let entry = parsed.properties.entry(key.clone()).or_default();
                        if !entry.is_empty() {
                            entry.push_str(", ");
                        }
                        entry.push_str(clean);
                    }
                }
            } else if let Some((k, v)) = trimmed.split_once(':') {
                let key = k.trim().to_string();
                let val = v.trim();
                if val.is_empty() {
                    current_list_key = Some(key);
                } else {
                    current_list_key = None;
                    let clean_val = strip_quotes(val).to_string();
                    if key == "tags" || key == "tag" {
                        for part in parse_inline_yaml_list(&clean_val) {
                            push_unique(&mut parsed.tags, normalize_tag(&part));
                        }
                    } else if key == "aliases" || key == "alias" {
                        for part in parse_inline_yaml_list(&clean_val) {
                            push_unique(&mut parsed.aliases, part);
                        }
                    }
                    parsed.properties.insert(key, clean_val);
                }
            }
            idx += 1;
        }
    }

    let mut in_fence = false;
    let mut fence_marker: Option<char> = None;

    for (line_idx, line) in lines.iter().enumerate().skip(idx) {
        let line_no = line_idx + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let marker = trimmed.chars().next().unwrap_or('`');
            if !in_fence {
                in_fence = true;
                fence_marker = Some(marker);
            } else if fence_marker == Some(marker) {
                in_fence = false;
                fence_marker = None;
            }
            continue;
        }
        if in_fence {
            continue;
        }

        if let Some(heading) = parse_heading(trimmed, line_no) {
            parsed.headings.push(heading);
        }
        if let Some(task) = parse_task(trimmed, line_no) {
            parsed.tasks.push(task);
        }

        extract_wikilinks(line, line_no, &mut parsed.links);
        extract_markdown_links(line, line_no, &mut parsed.links);
        extract_inline_tags(line, &mut parsed.tags);
    }

    parsed
}

fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn parse_inline_yaml_list(raw: &str) -> Vec<String> {
    let inner = raw
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(raw);
    inner
        .split(',')
        .map(strip_quotes)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn normalize_tag(raw: &str) -> String {
    raw.trim().trim_start_matches('#').to_string()
}

fn push_unique(vec: &mut Vec<String>, item: String) {
    if !item.is_empty() && !vec.contains(&item) {
        vec.push(item);
    }
}

fn parse_heading(trimmed: &str, line: usize) -> Option<Heading> {
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&hashes) {
        let rest = &trimmed[hashes..];
        if let Some(title) = rest.strip_prefix(' ') {
            let text = title.trim().trim_end_matches('#').trim().to_string();
            if !text.is_empty() {
                return Some(Heading {
                    level: hashes as u8,
                    text,
                    line,
                });
            }
        }
    }
    None
}

fn parse_task(trimmed: &str, line: usize) -> Option<TaskItem> {
    let after_bullet = trimmed
        .strip_prefix("- [")
        .or_else(|| trimmed.strip_prefix("* ["))
        .or_else(|| trimmed.strip_prefix("+ ["))?;
    let mut chars = after_bullet.chars();
    let status = chars.next()?;
    if chars.next()? != ']' {
        return None;
    }
    let text = chars.as_str().trim().to_string();
    Some(TaskItem { status, text, line })
}

fn extract_inline_tags(line: &str, out: &mut Vec<String>) {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_code = false;
    let mut in_link = false;

    while i < chars.len() {
        let c = chars[i];
        if c == '`' {
            in_code = !in_code;
            i += 1;
            continue;
        }
        if !in_code && c == '[' && i + 1 < chars.len() && chars[i + 1] == '[' {
            in_link = true;
            i += 2;
            continue;
        }
        if in_link && c == ']' && i + 1 < chars.len() && chars[i + 1] == ']' {
            in_link = false;
            i += 2;
            continue;
        }
        if !in_code && !in_link && c == '#' {
            let prev_ok = i == 0 || chars[i - 1].is_whitespace() || chars[i - 1] == '(';
            if prev_ok && i + 1 < chars.len() {
                let mut j = i + 1;
                let mut has_non_digit = false;
                while j < chars.len() {
                    let tc = chars[j];
                    if tc.is_alphanumeric() || tc == '_' || tc == '-' || tc == '/' {
                        if !tc.is_ascii_digit() {
                            has_non_digit = true;
                        }
                        j += 1;
                    } else {
                        break;
                    }
                }
                if j > i + 1 && has_non_digit {
                    let tag: String = chars[i + 1..j].iter().collect();
                    push_unique(out, tag);
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_links_tags_headings_and_tasks() {
        let md = "\
---
title: \"Project Alpha\"
status: active
tags:
  - rust/rewrite
  - pkm
aliases: [Alpha, ProjA]
---

# Overview

See [[Architecture#Core Crates|Core]] and ![[diagram.canvas]] #notes/design.

- [ ] Build vault scanner
- [x] Verify parser

```md
[[Ignored Link]] #ignored
```
";
        let note = parse_note(md);
        assert_eq!(note.properties.get("title").map(String::as_str), Some("Project Alpha"));
        assert_eq!(note.aliases, vec!["Alpha", "ProjA"]);
        assert_eq!(note.headings.len(), 1);
        assert_eq!(note.headings[0].text, "Overview");
        assert_eq!(note.links.len(), 2);
        assert_eq!(note.links[0].target, "Architecture");
        assert_eq!(note.links[0].subpath.as_deref(), Some("Core Crates"));
        assert_eq!(note.links[0].alias.as_deref(), Some("Core"));
        assert!(!note.links[0].is_embed);
        assert!(note.links[1].is_embed);
        assert_eq!(note.tags, vec!["rust/rewrite", "pkm", "notes/design"]);
        assert_eq!(note.tasks.len(), 2);
        assert_eq!(note.tasks[0].status, ' ');
        assert_eq!(note.tasks[1].status, 'x');
    }
}
