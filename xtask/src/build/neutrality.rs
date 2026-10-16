use std::fs;
use std::io;
use std::path::Path;

use super::content::{self, Content};
use super::harness::Harness;
use super::words::is_part_of_a_word;
use crate::diagnostics::{Diagnostic, naming};

pub fn check(content: &Content, harnesses: &[Harness], diagnostics: &mut Vec<Diagnostic>) {
    for file in &content.files {
        let text = String::from_utf8_lossy(&file.bytes);
        scan(&file.path, &text, harnesses, diagnostics, |harness| {
            format!("use a {{{{capability:…}}}} token defined in harnesses/{harness}/harness.toml")
        });
    }
}

pub fn check_binary(
    root: &Path,
    harnesses: &[Harness],
    diagnostics: &mut Vec<Diagnostic>,
) -> io::Result<()> {
    let sources = root.join("tightship").join("src");
    if !sources.is_dir() {
        return Ok(());
    }
    for file in content::files_under(&sources)? {
        let text = fs::read_to_string(&file).map_err(naming(&file))?;
        let relative = file.strip_prefix(root).map_err(io::Error::other)?;
        let path = relative.to_string_lossy().replace('\\', "/");
        scan(&path, &text, harnesses, diagnostics, |harness| {
            format!("move it to harnesses/{harness}/ and pass it in through the launcher")
        });
    }
    Ok(())
}

fn scan(
    path: &str,
    text: &str,
    harnesses: &[Harness],
    diagnostics: &mut Vec<Diagnostic>,
    remedy: impl Fn(&str) -> String,
) {
    for (index, line) in text.split('\n').enumerate() {
        for harness in harnesses {
            for term in terms_found(line, harness) {
                let message = format!(
                    "\"{term}\" belongs to {harness}; {remedy}",
                    harness = harness.name,
                    remedy = remedy(&harness.name)
                );
                diagnostics.push(Diagnostic::at_line(path, index + 1, message));
            }
        }
    }
}

fn terms_found<'harness>(line: &str, harness: &'harness Harness) -> Vec<&'harness str> {
    let matches: Vec<(usize, usize, &str)> = harness
        .neutrality
        .terms()
        .flat_map(|term| {
            line.match_indices(term)
                .filter(|(start, _)| !is_part_of_a_word(line, *start, term))
                .map(move |(start, _)| (start, start + term.len(), term))
        })
        .collect();
    matches
        .iter()
        .filter(|(start, end, _)| {
            !matches.iter().any(|(other_start, other_end, _)| {
                other_start <= start && end <= other_end && other_end - other_start > end - start
            })
        })
        .map(|(_, _, term)| *term)
        .collect()
}
