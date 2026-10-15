use crate::content::Content;
use crate::diagnostics::Diagnostic;
use crate::harness::Harness;

pub fn check(content: &Content, harnesses: &[Harness], diagnostics: &mut Vec<Diagnostic>) {
    for file in &content.files {
        let text = String::from_utf8_lossy(&file.bytes);
        for (index, line) in text.split('\n').enumerate() {
            for harness in harnesses {
                for term in terms_found(line, harness) {
                    let message = format!(
                        "\"{term}\" belongs to {harness}; use a {{{{capability:…}}}} token defined in harnesses/{harness}/harness.toml",
                        harness = harness.name
                    );
                    diagnostics.push(Diagnostic::at_line(&file.path, index + 1, message));
                }
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

fn is_part_of_a_word(line: &str, start: usize, term: &str) -> bool {
    let is_word_character = |character: char| character.is_alphanumeric() || character == '_';
    let whole_word_only =
        term.starts_with(char::is_alphanumeric) && term.ends_with(char::is_alphanumeric);
    if !whole_word_only {
        return false;
    }
    let before = line[..start].chars().next_back();
    let after = line[start + term.len()..].chars().next();
    before.is_some_and(is_word_character) || after.is_some_and(is_word_character)
}
