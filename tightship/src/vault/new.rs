//! `tightship vault new`: writes a note from its fields and the body on stdin.

use std::path::{Component, Path};

use clap::Args;
use jiff::Zoned;
use jiff::civil::Date;

use super::note::{self, NewNote};
use crate::failure::Failure;
use crate::project;

/// The fields of a new note, as the command line gives them.
#[derive(Args)]
pub struct Arguments {
    #[arg(long)]
    scope: Option<String>,
    #[arg(long = "type")]
    kind: String,
    #[arg(long)]
    title: String,
    #[arg(long)]
    why: String,
    #[arg(long)]
    source: String,
    #[arg(long)]
    tags: Option<String>,
    #[arg(long)]
    pin: bool,
    #[arg(long)]
    created: Option<String>,
}

/// Validates the fields, writes the note under its scope, and prints its path.
///
/// # Errors
///
/// When a field or the body is invalid, or the vault cannot be read or written.
pub fn run(arguments: &Arguments) -> Result<(), Failure> {
    if let Some(scope) = &arguments.scope {
        validate_scope(scope)?;
    }
    if !note::TYPES.contains(&arguments.kind.as_str()) {
        return Err(Failure::NoteTypeInvalid);
    }
    let title = note::title(&arguments.title)?;
    let why = note::one_line(&arguments.why);
    let why_words = why.split_whitespace().count();
    if why_words < note::WHY_MIN_WORDS {
        return Err(Failure::NoteWhyShort(why_words));
    }
    if !note::SOURCES.contains(&arguments.source.as_str()) {
        return Err(Failure::NoteSourceInvalid);
    }
    let tags: Vec<String> = arguments
        .tags
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .map(note::one_line)
        .filter(|tag| !tag.is_empty())
        .collect();
    let today = Zoned::now().date().to_string();
    let created = match &arguments.created {
        None => today.clone(),
        Some(text) => parse_created(text)?,
    };
    let body = note::read_body()?;

    let vault = super::folder()?;
    let scope = match &arguments.scope {
        Some(scope) => project::normalize(scope),
        None => project::scope(&vault)?,
    };
    let relative = note::write(
        &vault,
        &NewNote {
            scope: &scope,
            kind: &arguments.kind,
            title: &title,
            why: &why,
            tags: tags.iter().map(String::as_str).collect(),
            pinned: arguments.pin,
            source: &arguments.source,
            created,
            updated: today,
            body: &body,
        },
    )?;
    println!("{}", relative.display());
    Ok(())
}

fn validate_scope(scope: &str) -> Result<(), Failure> {
    let scope_length = scope.chars().count();
    let components: Vec<Component> = Path::new(scope).components().collect();
    let is_one_folder = matches!(components[..], [Component::Normal(_)]);
    if !(1..=100).contains(&scope_length)
        || scope.contains(['/', '\\'])
        || scope.starts_with('.')
        || scope.contains(char::is_control)
        || !is_one_folder
    {
        return Err(Failure::NoteScopeInvalid);
    }
    Ok(())
}

fn parse_created(text: &str) -> Result<String, Failure> {
    text.parse::<Date>()
        .ok()
        .map(|date| date.to_string())
        .filter(|date| date == text)
        .ok_or(Failure::NoteCreatedInvalid)
}
