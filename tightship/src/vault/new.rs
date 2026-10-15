use std::fs::{self, OpenOptions};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Component, Path, PathBuf};

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
    let one_line = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
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
    let title = one_line(&arguments.title);
    if !(1..=120).contains(&title.chars().count()) {
        return Err(Failure::NoteTitleInvalid);
    }
    let why = one_line(&arguments.why);
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
        .map(one_line)
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
    let mut body = String::new();
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        stdin
            .lock()
            .read_to_string(&mut body)
            .map_err(|_| Failure::NoteBodyMissing)?;
    }
    let body = body.trim();
    if body.is_empty() {
        return Err(Failure::NoteBodyMissing);
    }

    let vault = super::folder()?;
    let id = note::next_id(&note::read_all(&vault)?);
    let slug = note::slug(&title);
    let file_name = if slug.is_empty() {
        format!("{id}.md")
    } else {
        format!("{id}-{slug}.md")
    };
    let relative = PathBuf::from("notes").join(scope).join(file_name);
    let text = NewNote {
        scope,
        kind: &arguments.kind,
        title: &title,
        why: &why,
        tags: tags.iter().map(String::as_str).collect(),
        pinned: arguments.pin,
        source: &arguments.source,
        created,
        updated: today,
        body,
    }
    .text(&id);
    let unwritable = |error| Failure::NoteUnwritable(relative.clone(), error);
    fs::create_dir_all(vault.join("notes").join(scope)).map_err(unwritable)?;
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(vault.join(&relative))
    {
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(Failure::NoteExists(relative));
        }
        opened => opened.map_err(unwritable)?,
    };
    file.write_all(text.as_bytes()).map_err(unwritable)?;
    println!("{}", relative.display());
    Ok(())
}
