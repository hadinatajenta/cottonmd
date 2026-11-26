#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Emphasis {
    Normal,
    Bold,
    Strike,
}

pub fn split_emphasis(segment: &str) -> Vec<(Emphasis, &str)> {
    let mut spans = Vec::new();
    let mut rest = segment;
    while !rest.is_empty() {
        let Some((open, marker, emphasis)) = find_opening_marker(rest) else {
            spans.push((Emphasis::Normal, rest));
            break;
        };
        let after_open = &rest[open + 2..];
        let Some(close) = after_open.find(marker) else {
            spans.push((Emphasis::Normal, rest));
            break;
        };
        if open > 0 {
            spans.push((Emphasis::Normal, &rest[..open]));
        }
        spans.push((emphasis, &after_open[..close]));
        rest = &after_open[close + 2..];
    }
    spans
}

fn find_opening_marker(rest: &str) -> Option<(usize, &'static str, Emphasis)> {
    match (rest.find("**"), rest.find("~~")) {
        (Some(bold), Some(strike)) if strike < bold => Some((strike, "~~", Emphasis::Strike)),
        (Some(bold), _) => Some((bold, "**", Emphasis::Bold)),
        (None, Some(strike)) => Some((strike, "~~", Emphasis::Strike)),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emphasis_spans_split_bold_and_strike() {
        assert_eq!(
            split_emphasis("a **b** c ~~d~~"),
            vec![
                (Emphasis::Normal, "a "),
                (Emphasis::Bold, "b"),
                (Emphasis::Normal, " c "),
                (Emphasis::Strike, "d"),
            ]
        );
        assert_eq!(split_emphasis("**open"), vec![(Emphasis::Normal, "**open")]);
    }
}
