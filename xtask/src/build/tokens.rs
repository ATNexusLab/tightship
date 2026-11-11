//! Finds and substitutes the `{{<kind>:<name>}}` tokens of the canonical content.

use std::ops::Range;

use super::content::{Kind, is_valid_name};
use super::harness::Harness;
use super::options::is_option_name;

/// What a token refers to.
pub enum Form<'text> {
    Piece(Kind, &'text str),
    Capability(&'text str),
    Option(&'text str),
    UnknownKind(&'text str),
    Malformed,
}

/// A token, where it is, and what it refers to.
pub struct Token<'text> {
    pub line: usize,
    pub range: Range<usize>,
    pub text: &'text str,
    pub form: Form<'text>,
}

/// Every token of a text, in order; a token never spans two lines.
pub fn find(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut line_start = 0;
    for (index, line) in text.split('\n').enumerate() {
        let mut search_from = 0;
        while let Some(found) = line[search_from..].find("{{") {
            let open = search_from + found;
            let Some(inner_length) = line[open + 2..].find("}}") else {
                break;
            };
            let close = open + 2 + inner_length + 2;
            tokens.push(Token {
                line: index + 1,
                range: line_start + open..line_start + close,
                text: &line[open..close],
                form: form_of(&line[open + 2..close - 2]),
            });
            search_from = close;
        }
        line_start += line.len() + 1;
    }
    tokens
}

fn replace<'text>(
    text: &'text str,
    mut replacement: impl FnMut(&Token<'text>) -> String,
) -> String {
    let mut replaced = String::with_capacity(text.len());
    let mut copied_up_to = 0;
    for token in find(text) {
        replaced.push_str(&text[copied_up_to..token.range.start]);
        replaced.push_str(&replacement(&token));
        copied_up_to = token.range.end;
    }
    replaced.push_str(&text[copied_up_to..]);
    replaced
}

/// The text with each token replaced by the harness's value, and the tokens left unresolved.
pub fn substitute<'text>(text: &'text str, harness: &Harness) -> (String, Vec<&'text str>) {
    let mut unresolved = Vec::new();
    let rendered = replace(text, |token| {
        let replacement = match token.form {
            Form::Piece(kind, name) => Some(
                harness
                    .references
                    .template(kind)
                    .replace("{plugin}", &harness.plugin.name)
                    .replace("{name}", name),
            ),
            Form::Capability(name) => harness.capabilities.get(name).cloned(),
            Form::Option(name) => Some(harness.references.option(name)),
            Form::UnknownKind(_) | Form::Malformed => None,
        };
        replacement.unwrap_or_else(|| {
            unresolved.push(token.text);
            token.text.to_owned()
        })
    });
    (rendered, unresolved)
}

fn form_of(inner: &str) -> Form<'_> {
    let Some((kind, name)) = inner.split_once(':') else {
        return Form::Malformed;
    };
    let kind_is_word = !kind.is_empty() && kind.bytes().all(|byte| byte.is_ascii_lowercase());
    let name_is_valid = if kind == "option" {
        is_option_name(name)
    } else {
        is_valid_name(name)
    };
    if !kind_is_word || !name_is_valid {
        return Form::Malformed;
    }
    match kind {
        "skill" => Form::Piece(Kind::Skill, name),
        "command" => Form::Piece(Kind::Command, name),
        "agent" => Form::Piece(Kind::Agent, name),
        "capability" => Form::Capability(name),
        "option" => Form::Option(name),
        _ => Form::UnknownKind(kind),
    }
}
