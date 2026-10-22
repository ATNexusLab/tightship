use std::env;
use std::fs;
use std::io;
use std::path::Path;

use crate::failure::Failure;

pub fn scope(vault: &Path) -> Result<String, Failure> {
    let working_directory = env::current_dir().expect("the hook's working directory");
    let top = working_directory
        .ancestors()
        .find(|folder| folder.join(".git").exists())
        .unwrap_or(&working_directory);
    let name = normalize(&top.file_name().unwrap_or_default().to_string_lossy());
    let scopes = scope_folders(vault)?;
    if scopes.contains(&name) {
        return Ok(name);
    }
    let prefix = scopes
        .into_iter()
        .filter(|scope| scope != "all")
        .filter(|scope| {
            name.strip_prefix(scope.as_str())
                .is_some_and(|rest| rest.starts_with(['-', '_']))
        })
        .max_by_key(String::len);
    Ok(prefix.unwrap_or(name))
}

pub fn normalize(name: &str) -> String {
    name.to_lowercase().trim_start_matches('.').to_owned()
}

fn scope_folders(vault: &Path) -> Result<Vec<String>, Failure> {
    let unreadable = |error| Failure::VaultUnreadable(vault.to_path_buf(), error);
    let entries = match fs::read_dir(vault.join("notes")) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        entries => entries.map_err(unreadable)?,
    };
    let mut scopes = Vec::new();
    for entry in entries {
        let entry = entry.map_err(unreadable)?;
        if entry.file_type().map_err(unreadable)?.is_dir() {
            scopes.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(scopes)
}
