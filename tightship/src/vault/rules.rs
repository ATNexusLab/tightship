use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

pub const START: &str = "<!-- tightship:rules:start -->";
pub const END: &str = "<!-- tightship:rules:end -->";
const TEMPLATE: &str = include_str!("../../../content/vault-template/AGENTS.md");

pub fn block() -> String {
    format!("{START}\n{TEMPLATE}{END}\n")
}

pub fn run(vault: &Path) -> Result<bool, Failure> {
    let path = vault.join("AGENTS.md");
    let file = PathBuf::from("AGENTS.md");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(Failure::NoteUnreadable(file, error)),
    };
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
