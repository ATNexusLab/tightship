//! Checks every token and every literal reference to a piece in `content/`.

use super::content::{Content, Kind, SupportingBody};
use super::harness::Harness;
use super::tokens::{self, Form, Token};
use crate::diagnostics::Diagnostic;

struct Text<'content> {
    path: &'content str,
    text: &'content str,
    first_line: usize,
    in_piece: bool,
}

/// Reports each malformed or unknown token, and each piece named without one.
pub fn check(content: &Content, harnesses: &[Harness], diagnostics: &mut Vec<Diagnostic>) {
    let pieces: Vec<(Kind, &str)> = content
        .pieces
        .iter()
        .map(|piece| (piece.kind, piece.name.as_str()))
        .collect();
    let boolean_options: Vec<&str> = content
        .options
        .iter()
        .flat_map(|file| &file.declared)
        .filter(|option| option.kind == "boolean")
        .map(|option| option.name.as_str())
        .collect();
    let forms = literal_forms(&pieces);
    for text in texts(content) {
        let report = |line: usize, message: String| {
            Diagnostic::at_line(text.path, text.first_line + line - 1, message)
        };
        let mut blanked = text.text.as_bytes().to_vec();
        for token in tokens::find(text.text) {
            blanked[token.range.clone()].fill(b' ');
            let message = token_error(&token, text.in_piece, &pieces, &boolean_options, harnesses);
            diagnostics.extend(
                message
                    .into_iter()
                    .map(|message| report(token.line, message)),
            );
        }
        let blanked = String::from_utf8_lossy(&blanked);
        for (index, line) in blanked.split('\n').enumerate() {
            let broken = line
                .split_whitespace()
                .filter(|word| word.contains("{{") || word.contains("}}"));
            for word in broken {
                let message =
                    format!("\"{word}\" is not a token; write {{{{<kind>:<name>}}}} on one line");
                diagnostics.push(report(index + 1, message));
            }
            for (literal, kind, name) in literal_references(line, &pieces, &forms) {
                let message = format!(
                    "\"{literal}\" refers to the {noun} \"{name}\"; write {{{{{noun}:{name}}}}}",
                    noun = kind.noun()
                );
                diagnostics.push(report(index + 1, message));
            }
        }
    }
}

fn texts(content: &Content) -> Vec<Text<'_>> {
    let whole_file = |path, text| Text {
        path,
        text,
        first_line: 1,
        in_piece: false,
    };
    let bodies = content.pieces.iter().map(|piece| Text {
        path: &piece.path,
        text: &piece.body,
        first_line: piece.body_line,
        in_piece: true,
    });
    let supporting = content
        .supporting
        .iter()
        .filter_map(|supporting| match &supporting.body {
            SupportingBody::Markdown(text) => Some(whole_file(&supporting.path, text)),
            SupportingBody::Copied(_) => None,
        });
    let options = content
        .options
        .iter()
        .map(|file| whole_file(&file.path, &file.text));
    let contract = content
        .contract
        .iter()
        .map(|contract| whole_file(&contract.path, &contract.text));
    let vault_template = content
        .vault_template
        .iter()
        .map(|text| whole_file("content/vault-template/AGENTS.md", text));
    bodies
        .chain(supporting)
        .chain(options)
        .chain(contract)
        .chain(vault_template)
        .collect()
}

fn token_error(
    token: &Token<'_>,
    in_piece: bool,
    pieces: &[(Kind, &str)],
    boolean_options: &[&str],
    harnesses: &[Harness],
) -> Vec<String> {
    let text = token.text;
    match &token.form {
        Form::Piece(kind, name) if !pieces.contains(&(*kind, name)) => {
            vec![format!("{text} names no {} in content/", kind.noun())]
        }
        Form::Piece(..) => Vec::new(),
        Form::Capability(name) => harnesses
            .iter()
            .filter(|harness| !harness.capabilities.contains_key(*name))
            .map(|harness| {
                format!(
                    "capability \"{name}\" has no entry in harnesses/{}/harness.toml",
                    harness.name
                )
            })
            .collect(),
        Form::Option(_) if !in_piece => {
            vec![format!(
                "{text} works only in a skill, a command, or an agent"
            )]
        }
        Form::Option(name) if !boolean_options.contains(name) => {
            vec![format!(
                "{text} names no boolean option in content/options.toml"
            )]
        }
        Form::Option(_) => Vec::new(),
        Form::UnknownKind(kind) => vec![format!("unknown token kind \"{kind}\" in {text}")],
        Form::Malformed => vec![format!(
            "\"{text}\" is not a token; write {{{{<kind>:<name>}}}}"
        )],
    }
}

fn literal_forms<'name>(pieces: &[(Kind, &'name str)]) -> Vec<(String, Kind, &'name str)> {
    pieces
        .iter()
        .flat_map(|(kind, name)| {
            let forms = match kind {
                Kind::Skill => vec![format!("`{name}`"), format!("skills/{name}/")],
                Kind::Agent => vec![format!("`{name}`"), format!("agents/{name}.md")],
                Kind::Command => vec![format!("commands/{name}.md")],
            };
            forms.into_iter().map(move |form| (form, *kind, *name))
        })
        .collect()
}

fn literal_references<'name>(
    line: &str,
    pieces: &[(Kind, &'name str)],
    forms: &[(String, Kind, &'name str)],
) -> Vec<(String, Kind, &'name str)> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    let continues_name = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'-';
    for (slash, _) in line.match_indices('/') {
        let before = slash.checked_sub(1).map(|index| bytes[index]);
        if before.is_some_and(|byte| continues_name(byte) || byte == b'/' || byte == b'.') {
            continue;
        }
        let name_length = bytes[slash + 1..]
            .iter()
            .position(|byte| !continues_name(*byte))
            .unwrap_or(bytes.len() - slash - 1);
        let name_end = slash + 1 + name_length;
        if bytes.get(name_end) == Some(&b'/') {
            continue;
        }
        let name = &line[slash + 1..name_end];
        let command = pieces
            .iter()
            .find(|(kind, piece)| *kind == Kind::Command && *piece == name);
        if let Some((kind, piece)) = command {
            found.push((format!("/{piece}"), *kind, *piece));
        }
    }
    for (form, kind, name) in forms {
        let occurrences = line.matches(form.as_str()).count();
        found.extend((0..occurrences).map(|_| (form.clone(), *kind, *name)));
    }
    found
}
