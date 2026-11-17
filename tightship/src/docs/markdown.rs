//! The parts of a Markdown doc the check reads: its links, code spans, prose, and anchors.

use std::collections::HashSet;

/// A piece of a doc with the line it sits on, 1-based.
pub struct Item {
    pub text: String,
    pub line: usize,
}

/// What the check reads from one doc; fenced code blocks hold none of it.
#[derive(Default)]
pub struct Parsed {
    pub links: Vec<Item>,
    pub spans: Vec<Item>,
    pub prose: Vec<Item>,
    pub anchors: HashSet<String>,
}

/// Parses the doc from line `body_start` on.
pub fn parse(text: &str, body_start: usize) -> Parsed {
    let mut parsed = Parsed::default();
    let mut fence: Option<(char, usize)> = None;
    let mut headings: Vec<String> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if number < body_start {
            continue;
        }
        let trimmed = line.trim_start();
        if let Some((mark, length)) = fence {
            let run = trimmed.chars().take_while(|c| *c == mark).count();
            if run >= length && trimmed[run * mark.len_utf8()..].trim().is_empty() {
                fence = None;
            }
            continue;
        }
        if let Some(mark) = ['`', '~']
            .into_iter()
            .find(|mark| trimmed.starts_with(*mark))
        {
            let run = trimmed.chars().take_while(|c| *c == mark).count();
            if run >= 3 {
                fence = Some((mark, run));
                continue;
            }
        }
        let (prose, spans) = split_spans(line);
        parsed
            .spans
            .extend(spans.into_iter().map(|text| Item { text, line: number }));
        parsed.links.extend(
            links(&prose)
                .into_iter()
                .map(|text| Item { text, line: number }),
        );
        anchors_of_tags(line, &mut parsed.anchors);
        if let Some(title) = heading(line) {
            let base = slug(&title);
            let count = headings.iter().filter(|seen| **seen == base).count();
            parsed.anchors.insert(if count == 0 {
                base.clone()
            } else {
                format!("{base}_{count}")
            });
            headings.push(base);
        }
        parsed.prose.push(Item {
            text: prose,
            line: number,
        });
    }
    parsed
}

/// The line with its code spans cut out, and the spans' contents.
fn split_spans(line: &str) -> (String, Vec<String>) {
    let mut prose = String::new();
    let mut spans = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('`') {
        let run = rest[start..].chars().take_while(|c| *c == '`').count();
        let after = &rest[start + run..];
        let closing = find_run(after, run);
        prose.push_str(&rest[..start]);
        match closing {
            Some(end) => {
                let content = &after[..end];
                let content =
                    if content.len() > 1 && content.starts_with(' ') && content.ends_with(' ') {
                        &content[1..content.len() - 1]
                    } else {
                        content
                    };
                spans.push(content.to_owned());
                rest = &after[end + run..];
            }
            None => {
                prose.push_str(&rest[start..start + run]);
                rest = after;
            }
        }
    }
    prose.push_str(rest);
    (prose, spans)
}

/// Where a run of exactly `length` backticks starts in the text.
fn find_run(text: &str, length: usize) -> Option<usize> {
    let mut offset = 0;
    while let Some(start) = text[offset..].find('`') {
        let at = offset + start;
        let run = text[at..].chars().take_while(|c| *c == '`').count();
        if run == length {
            return Some(at);
        }
        offset = at + run;
    }
    None
}

/// The targets of the inline links and the reference definitions in a line of prose.
fn links(prose: &str) -> Vec<String> {
    let mut found = Vec::new();
    let trimmed = prose.trim_start();
    if trimmed.starts_with('[')
        && let Some((_, target)) = trimmed.split_once("]: ")
    {
        found.push(target.split_whitespace().next().unwrap_or("").to_owned());
    }
    let mut rest = prose;
    while let Some(start) = rest.find("](") {
        let after = &rest[start + 2..];
        let end = after.find(')').unwrap_or(after.len());
        let target = after[..end].split_whitespace().next().unwrap_or("");
        found.push(target.trim_matches(['<', '>']).to_owned());
        rest = &after[end..];
    }
    found.retain(|target| !target.is_empty());
    found
}

/// The ids of the `<a id="…">` tags in a line.
fn anchors_of_tags(line: &str, anchors: &mut HashSet<String>) {
    let mut rest = line;
    while let Some(start) = rest.find("<a ") {
        let tag = &rest[start..];
        let end = tag.find('>').unwrap_or(tag.len());
        if let Some(id) = tag[..end]
            .split_once("id=\"")
            .and_then(|(_, value)| value.split_once('"'))
            .map(|(id, _)| id)
        {
            anchors.insert(id.to_owned());
        }
        rest = &tag[end..];
    }
}

/// The text of an ATX heading, without its marks.
fn heading(line: &str) -> Option<String> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let title = line[hashes..].strip_prefix(' ')?;
    Some(title.trim().trim_end_matches('#').trim().to_owned())
}

/// The anchor Python-Markdown's table of contents gives a heading.
pub fn slug(title: &str) -> String {
    let mut text = String::new();
    let mut rest = title;
    // A link in a heading renders as its text, so its target never reaches the anchor.
    while let Some(start) = rest.find("](") {
        text.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        rest = after.find(')').map_or("", |end| &after[end + 1..]);
    }
    text.push_str(rest);
    let mut kept = String::new();
    for character in text.chars() {
        if character.is_ascii() {
            if character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | ' ' | '\t') {
                kept.push(character);
            }
        } else {
            // NFKD then ASCII keeps an accented letter's base and drops every other symbol.
            let folded = deunicode::deunicode_char(character).unwrap_or("");
            if !folded.is_empty() && folded.chars().all(|c| c.is_ascii_alphanumeric()) {
                kept.push_str(folded);
            }
        }
    }
    let mut slug = String::new();
    let mut gap = false;
    for character in kept.trim().to_lowercase().chars() {
        if character == '-' || character.is_whitespace() {
            gap = true;
        } else {
            if gap && !slug.is_empty() {
                slug.push('-');
            }
            gap = false;
            slug.push(character);
        }
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::{parse, slug};

    #[test]
    fn slugs_like_python_markdown() {
        assert_eq!(slug("FR-SKL-12 — The closeout"), "fr-skl-12-the-closeout");
        assert_eq!(slug("`system/architecture/`"), "systemarchitecture");
        assert_eq!(slug("Instalação rápida"), "instalacao-rapida");
        assert_eq!(slug("M<N> — <name>"), "mn-name");
    }

    #[test]
    fn reads_links_spans_and_anchors_outside_fences() {
        let text = "# Title\n\nSee [a](a.md#x) and `code` [b](<b.md>).\n\n```text\n[c](c.md)\n```\n<a id=\"fr-01\"></a>\n\n# Title\n";
        let parsed = parse(text, 1);
        let links: Vec<&str> = parsed.links.iter().map(|item| item.text.as_str()).collect();
        assert_eq!(links, ["a.md#x", "b.md"]);
        assert_eq!(parsed.spans[0].text, "code");
        for anchor in ["title", "title_1", "fr-01"] {
            assert!(parsed.anchors.contains(anchor), "{anchor}");
        }
    }
}
