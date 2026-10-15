use std::path::{Component, Path};

use clap::Args;
use jiff::Zoned;
use jiff::civil::Date;

use super::note::{self, NewNote};
use crate::failure::Failure;

#[derive(Args)]
pub struct Arguments {
    #[arg(long)]
    scope: String,
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

pub fn run(arguments: &Arguments) -> Result<(), Failure> {
    let scope = &arguments.scope;
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
    if !["preference", "decision", "gotcha", "project"].contains(&arguments.kind.as_str()) {
        return Err(Failure::NoteTypeInvalid);
    }
    let title = note::title(&arguments.title)?;
    let why = note::one_line(&arguments.why);
    let why_words = why.split_whitespace().count();
    if why_words < 5 {
        return Err(Failure::NoteWhyShort(why_words));
    }
    if !["user", "session", "repo"].contains(&arguments.source.as_str()) {
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
        Some(text) => text
            .parse::<Date>()
            .ok()
            .map(|date| date.to_string())
            .filter(|date| date == text)
            .ok_or(Failure::NoteCreatedInvalid)?,
    };
    let body = note::read_body()?;

    let vault = super::folder()?;
    let relative = note::write(
        &vault,
        &NewNote {
            scope,
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
