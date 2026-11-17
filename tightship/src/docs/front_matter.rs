//! The `covers` key of a doc's YAML front matter, read without a YAML parser.

/// One pattern of `covers`, with the line it sits on.
pub struct Cover {
    pub pattern: String,
    pub line: usize,
}

/// The `covers` patterns of a doc, and the first line of its body, 1-based.
///
/// A doc with no front matter, or one that never closes, covers nothing and its body starts at line 1.
pub fn covers(text: &str) -> (Vec<Cover>, usize) {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|line| line.trim_end()) != Some("---") {
        return (Vec::new(), 1);
    }
    let Some(close) = lines
        .iter()
        .skip(1)
        .position(|line| line.trim_end() == "---")
        .map(|index| index + 1)
    else {
        return (Vec::new(), 1);
    };
    let mut found = Vec::new();
    let mut in_list = false;
    for (index, line) in lines.iter().enumerate().take(close).skip(1) {
        let number = index + 1;
        if let Some(value) = line.strip_prefix("covers:") {
            let value = value.trim();
            in_list = value.is_empty();
            if let Some(inline) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
                found.extend(
                    inline
                        .split(',')
                        .map(unquote)
                        .filter(|pattern| !pattern.is_empty())
                        .map(|pattern| Cover {
                            pattern,
                            line: number,
                        }),
                );
            } else if !value.is_empty() {
                found.push(Cover {
                    pattern: unquote(value),
                    line: number,
                });
            }
        } else if in_list && let Some(item) = line.trim_start().strip_prefix("- ") {
            found.push(Cover {
                pattern: unquote(item),
                line: number,
            });
        } else if !line.starts_with(' ') {
            in_list = false;
        }
    }
    (found, close + 2)
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches(['"', '\'']).to_owned()
}

#[cfg(test)]
mod tests {
    use super::covers;

    fn patterns(text: &str) -> Vec<String> {
        covers(text)
            .0
            .into_iter()
            .map(|cover| cover.pattern)
            .collect()
    }

    #[test]
    fn reads_an_inline_and_a_block_list() {
        assert_eq!(
            patterns("---\ncovers: [src/a/**, \"src/b.rs\"]\n---\n# A\n"),
            ["src/a/**", "src/b.rs"]
        );
        assert_eq!(
            patterns("---\ntitle: x\ncovers:\n  - src/a/**\n  - src/b.rs\nother: y\n---\n"),
            ["src/a/**", "src/b.rs"]
        );
    }

    #[test]
    fn covers_nothing_without_a_closed_front_matter() {
        assert!(patterns("# A\ncovers: [src/**]\n").is_empty());
        assert!(patterns("---\ncovers: [src/**]\n").is_empty());
        assert_eq!(covers("---\ncovers: [a]\n---\n# A\n").1, 4);
    }
}
