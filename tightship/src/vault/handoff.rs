//! `tightship vault handoff`: leaves the session's work as a note for the next one, or takes it.

use std::fs;
use std::io;
use std::path::Path;

use clap::Subcommand;
use jiff::Zoned;

use super::note::{self, NewNote, Note};
use crate::failure::Failure;
use crate::project;

/// The handoff subcommands.
#[derive(Subcommand)]
pub enum Command {
    Release {
        #[arg(long)]
        title: String,
    },
    Claim,
}

/// Releases or claims a handoff.
///
/// # Errors
///
/// When the title or body is invalid, or the vault cannot be read or written.
pub fn run(command: &Command) -> Result<(), Failure> {
    match command {
        Command::Release { title } => release(title),
        Command::Claim => claim(),
    }
}

/// The handoffs of the scope, the most recently updated first.
pub fn in_scope<'notes>(notes: &'notes [Note], scope: &str) -> Vec<&'notes Note> {
    let mut handoffs: Vec<&Note> = notes
        .iter()
        .filter(|note| note.field("type") == note::HANDOFF_TYPE && note.field("project") == scope)
        .collect();
    handoffs.sort_by(|first, second| {
        (second.field("updated"), second.id_order())
            .cmp(&(first.field("updated"), first.id_order()))
    });
    handoffs
}

/// The heading a handoff is shown under.
pub fn heading(handoff: &Note, scope: &str) -> String {
    format!(
        "Handoff — {scope} · {} · {}",
        handoff.field("id"),
        handoff.age()
    )
}

/// The labels a handoff body needs and does not start a line with.
pub fn missing_labels(body: &str) -> Vec<&'static str> {
    [
        "**Goal:**",
        "**Done:**",
        "**Next step:**",
        "**Open decisions:**",
        "**Repo state:**",
    ]
    .into_iter()
    .filter(|label| !body.lines().any(|line| line.starts_with(label)))
    .collect()
}

fn release(title: &str) -> Result<(), Failure> {
    let title = note::title(title)?;
    let body = note::read_body()?;
    let missing = missing_labels(&body);
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
            kind: note::HANDOFF_TYPE,
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
        let path = vault.join(&handoff.path);
        let claimed_path = path.with_extension(format!("claimed-{}", std::process::id()));
        match fs::rename(&path, &claimed_path) {
            Err(_) if !path.exists() => continue,
            Err(error) => return Err(Failure::NoteUnremovable(handoff.path.clone(), error)),
            Ok(()) => {}
        }
        match fs::remove_file(&claimed_path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(Failure::NoteUnremovable(handoff.path.clone(), error)),
            Ok(()) => {}
        }
        if claimed > 0 {
            println!();
        }
        println!("# {}\n\n{}", heading(handoff, &scope), handoff.body);
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
