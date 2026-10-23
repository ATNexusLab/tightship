use std::fs;
use std::io;
use std::path::Path;

use clap::Subcommand;
use jiff::Zoned;

use super::note::{self, NewNote, Note};
use crate::failure::Failure;
use crate::project;

#[derive(Subcommand)]
pub enum Command {
    Release {
        #[arg(long)]
        title: String,
    },
    Claim,
}

pub fn run(command: &Command) -> Result<(), Failure> {
    match command {
        Command::Release { title } => release(title),
        Command::Claim => claim(),
    }
}

pub fn in_scope<'notes>(notes: &'notes [Note], scope: &str) -> Vec<&'notes Note> {
    let mut handoffs: Vec<&Note> = notes
        .iter()
        .filter(|note| note.field("type") == "handoff" && note.field("project") == scope)
        .collect();
    handoffs.sort_by(|first, second| {
        (second.field("updated"), second.id_order())
            .cmp(&(first.field("updated"), first.id_order()))
    });
    handoffs
}

fn release(title: &str) -> Result<(), Failure> {
    let title = note::title(title)?;
    let body = note::read_body()?;
    let missing: Vec<&'static str> = [
        "**Goal:**",
        "**Done:**",
        "**Next step:**",
        "**Open decisions:**",
        "**Repo state:**",
    ]
    .into_iter()
    .filter(|label| !body.lines().any(|line| line.starts_with(label)))
    .collect();
    if !missing.is_empty() {
        return Err(Failure::HandoffBodyIncomplete(missing));
    }

    let vault = super::folder()?;
    let scope = project::scope(&vault)?;
    let notes = note::read_all(&vault)?;
    let mut earlier: Vec<&Path> = in_scope(&notes, &scope)
        .into_iter()
        .map(|handoff| handoff.path.as_path())
        .collect();
    earlier.sort();
    let today = Zoned::now().date().to_string();
    let written = note::write(
        &vault,
        &NewNote {
            scope: &scope,
            kind: "handoff",
            title: &title,
            why: "without it the next session, on another machine or harness, starts blind on work already in flight",
            tags: Vec::new(),
            pinned: false,
            source: "session",
            created: today.clone(),
            updated: today,
            body: &body,
        },
    )?;
    println!("{}", written.display());
    for path in earlier {
        remove(&vault, path)?;
        println!("replaced {}", path.display());
    }
    Ok(())
}

fn claim() -> Result<(), Failure> {
    let vault = super::folder()?;
    let scope = project::scope(&vault)?;
    let notes = note::read_all(&vault)?;
    let mut claimed = 0;
    for handoff in in_scope(&notes, &scope) {
        match fs::remove_file(vault.join(&handoff.path)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(Failure::NoteUnremovable(handoff.path.clone(), error)),
            Ok(()) => {}
        }
        if claimed > 0 {
            println!();
        }
        println!(
            "# Handoff — {scope} · {} · {}\n\n{}",
            handoff.field("id"),
            handoff.age(),
            handoff.body
        );
        claimed += 1;
    }
    if claimed == 0 {
        println!("no handoff for {scope}");
    }
    Ok(())
}

fn remove(vault: &Path, relative: &Path) -> Result<(), Failure> {
    fs::remove_file(vault.join(relative))
        .map_err(|error| Failure::NoteUnremovable(relative.to_path_buf(), error))
}
