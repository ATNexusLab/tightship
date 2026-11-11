//! Removing notes past their age, and the links other notes had to them.

use std::fs;
use std::path::Path;

use jiff::ToSpan;
use jiff::civil::Date;

use super::note::{self, Note};
use crate::failure::Failure;

/// Removes the expired notes, unlinks them from the others, and returns how many it removed.
///
/// # Errors
///
/// When a note cannot be read, removed, or rewritten.
pub fn run(vault: &Path, today: Date) -> Result<usize, Failure> {
    let notes = note::read_all(vault)?;
    let expired: Vec<&Note> = notes
        .iter()
        .filter(|note| is_expired(note, today))
        .collect();
    if expired.is_empty() {
        return Ok(0);
    }
    println!("> Removing expired notes");
    let mut titles = Vec::new();
    for note in &expired {
        fs::remove_file(vault.join(&note.path))
            .map_err(|error| Failure::NoteUnremovable(note.path.clone(), error))?;
        println!(
            "expired {} (updated {})",
            note.path.display(),
            note.field("updated")
        );
        titles.push((note.stem(), note.title()));
    }
    note::rewrite_all(vault, |_, text| {
        titles.iter().fold(text.to_owned(), |text, (stem, title)| {
            unlink(&text, stem, title)
        })
    })?;
    Ok(expired.len())
}

fn is_expired(note: &Note, today: Date) -> bool {
    ["gotcha", "project"].contains(&note.field("type"))
        && note.field("source") != "user"
        && !note.is_pinned()
        && note
            .updated()
            .is_some_and(|updated| updated.saturating_add(30.days()) < today)
}

fn unlink(text: &str, stem: &str, title: &str) -> String {
    let opening = format!("[[{stem}");
    let mut unlinked = String::new();
    let mut rest = text;
    while let Some(start) = rest.find(&opening) {
        let after = &rest[start + opening.len()..];
        let link = after
            .find("]]")
            .map(|end| &after[..end])
            .filter(|inside| inside.is_empty() || inside.starts_with(['#', '|']));
        let Some(inside) = link else {
            unlinked += &rest[..start + opening.len()];
            rest = after;
            continue;
        };
        unlinked += &rest[..start];
        unlinked += inside.split_once('|').map_or(title, |(_, shown)| shown);
        rest = &after[inside.len() + 2..];
    }
    unlinked + rest
}
