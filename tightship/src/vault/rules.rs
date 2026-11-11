use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::files::read_if_present;

pub const START: &str = "<!-- tightship:rules:start -->";
pub const END: &str = "<!-- tightship:rules:end -->";
const TEMPLATE: &str = include_str!("../../../content/vault-template/AGENTS.md");

pub fn block() -> String {
    format!("{START}\n{TEMPLATE}{END}\n")
}

pub fn run(vault: &Path) -> Result<bool, Failure> {
    let path = vault.join("AGENTS.md");
    let file = PathBuf::from("AGENTS.md");
    let text = read_if_present(&path)
        .map_err(|error| Failure::NoteUnreadable(file.clone(), error))?
        .unwrap_or_default();
    let Some(updated) = with_block(&text)? else {
        return Ok(false);
    };
    println!("> Updating the vault's rules");
    fs::write(&path, updated).map_err(|error| Failure::NoteUnwritable(file, error))?;
    Ok(true)
}

fn with_block(text: &str) -> Result<Option<String>, Failure> {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let marked = |marker: &str| -> Vec<usize> {
        lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.trim() == marker)
            .map(|(index, _)| index)
            .collect()
    };
    match (marked(START).as_slice(), marked(END).as_slice()) {
        ([], []) if text.is_empty() => Ok(Some(block())),
        ([], []) => Ok(Some(format!("{}\n{text}", block()))),
        ([start], [end]) if start < end => {
            if lines[start + 1..*end].concat().replace('\r', "") == TEMPLATE {
                return Ok(None);
            }
            Ok(Some(format!(
                "{}{}{}",
                lines[..*start].concat(),
                block(),
                lines[end + 1..].concat()
            )))
        }
        _ => Err(Failure::RulesBlockBroken),
    }
}
