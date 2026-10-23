use std::fs;
use std::path::Path;

use jiff::ToSpan;
use jiff::civil::Date;

use super::note::{self, Note};
use crate::failure::Failure;

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
    for file in note::markdown_files(vault, &vault.join("notes"))? {
        let path = vault.join(&file);
        let text = fs::read_to_string(&path)
            .map_err(|error| Failure::NoteUnreadable(file.clone(), error))?;
        let unlinked = titles.iter().fold(text.clone(), |text, (stem, title)| {
            unlink(&text, stem, title)
        });
        if unlinked != text {
            fs::write(&path, unlinked).map_err(|error| Failure::NoteUnwritable(file, error))?;
        }
    }
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
