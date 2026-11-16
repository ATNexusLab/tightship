//! `tightship status-line render`: reads the session from stdin and prints the status line.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::layout::{rows, text};
use super::rate_limits::remember_from_cache;
use crate::failure::Failure;
use crate::project::repository_root;

/// Prints the status line rows for the session the harness sends on stdin.
///
/// # Errors
///
/// When stdin cannot be read or is not a JSON object.
pub fn run() -> Result<(), Failure> {
    let input = io::read_to_string(io::stdin())
        .map_err(|error| Failure::StatusInputInvalid(error.to_string()))?;
    let mut session: Value = serde_json::from_str(&input)
        .map_err(|error| Failure::StatusInputInvalid(error.to_string()))?;
    if !session.is_object() {
        return Err(Failure::StatusInputInvalid("expected an object".to_owned()));
    }
    let prefix = env::var("TIGHTSHIP_MODEL_PREFIX").unwrap_or_default();
    if let Some(Value::String(model)) = session.pointer_mut("/model/display_name")
        && let Some(short) = model.strip_prefix(prefix.as_str())
    {
        *model = short.to_owned();
    }
    remember_from_cache(&mut session);
    let branch =
        text(&session, "/workspace/current_dir").and_then(|folder| read_branch(Path::new(folder)));
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|columns| columns.parse::<usize>().ok())
        .filter(|columns| *columns > 0)
        .map(|columns| columns.saturating_sub(4));
    for row in rows(&session, branch.as_deref(), width) {
        println!("{row}");
    }
    Ok(())
}

fn read_branch(folder: &Path) -> Option<String> {
    let repository = repository_root(folder)?;
    let mut git_folder: PathBuf = repository.join(".git");
    if git_folder.is_file() {
        let pointer = fs::read_to_string(&git_folder).ok()?;
        let target = pointer.trim();
        git_folder = repository.join(target.strip_prefix("gitdir: ").unwrap_or(target));
    }
    let head = fs::read_to_string(git_folder.join("HEAD")).ok()?;
    head.trim()
        .strip_prefix("ref: refs/heads/")
        .map(str::to_owned)
}
