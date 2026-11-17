//! Matching a `covers` pattern against a path: `*` within one segment, `**` across any number of them.

/// Whether the pattern matches the path; a pattern with no wildcard also matches everything under it.
pub fn matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim_start_matches("./");
    if !pattern.contains('*') {
        let folder = pattern.trim_end_matches('/');
        return path == folder || path.starts_with(&format!("{folder}/"));
    }
    let wanted: Vec<&str> = pattern.split('/').collect();
    let parts: Vec<&str> = path.split('/').collect();
    segments(&wanted, &parts)
}

fn segments(wanted: &[&str], parts: &[&str]) -> bool {
    match wanted.split_first() {
        None => parts.is_empty(),
        Some((&"**", rest)) => (0..=parts.len()).any(|skip| segments(rest, &parts[skip..])),
        Some((first, rest)) => parts
            .split_first()
            .is_some_and(|(part, others)| segment(first, part) && segments(rest, others)),
    }
}

fn segment(wanted: &str, part: &str) -> bool {
    let Some((head, tail)) = wanted.split_once('*') else {
        return wanted == part;
    };
    let Some(after) = part.strip_prefix(head) else {
        return false;
    };
    (0..=after.len())
        .filter(|cut| after.is_char_boundary(*cut))
        .any(|cut| segment(tail, &after[cut..]))
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn matches_wildcards_and_folders() {
        assert!(matches("src/**", "src/a/b.rs"));
        assert!(matches("src/**/*.rs", "src/b.rs"));
        assert!(matches("src/*.rs", "src/b.rs"));
        assert!(!matches("src/*.rs", "src/a/b.rs"));
        assert!(matches("launcher-env.*", "launcher-env.sh"));
        assert!(matches("src/", "src/a/b.rs"));
        assert!(matches("src/b.rs", "src/b.rs"));
        assert!(!matches("src", "srcs/b.rs"));
        assert!(!matches("src/**", "other/b.rs"));
    }
}
