use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

pub const STAGING: &str = ".update";
const ASIDE: &str = ".tightship-old";

pub fn check_plain(staging: &Path, archive: &str) -> Result<(), Failure> {
    let mut folders = vec![staging.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let unreadable =
            |error: io::Error| Failure::UpdateExtractFailed(archive.to_owned(), error.to_string());
        for entry in fs::read_dir(&folder).map_err(unreadable)? {
            let path = entry.map_err(unreadable)?.path();
            let kind = fs::symlink_metadata(&path).map_err(unreadable)?.file_type();
            if kind.is_dir() {
                folders.push(path);
            } else if !kind.is_file() {
                let entry = path.strip_prefix(staging).unwrap_or(&path);
                return Err(Failure::UpdateArchiveUnsafe(
                    archive.to_owned(),
                    entry.display().to_string(),
                ));
            }
        }
    }
    Ok(())
}

pub fn place_windows_hooks(staging: &Path) -> Result<(), Failure> {
    let windows_hooks = staging.join("hooks.windows.json");
    if !windows_hooks.is_file() {
        return Ok(());
    }
    let placed = if cfg!(windows) {
        fs::rename(&windows_hooks, staging.join("hooks.json"))
    } else {
        fs::remove_file(&windows_hooks)
    };
    placed.map_err(|error| Failure::UpdateApplyFailed(windows_hooks, error))
}

pub fn swap(root: &Path, staging: &Path) -> Result<(), Failure> {
    remove_aside_and_empty(root);
    let incoming = files(staging, staging)?;
    let outgoing: Vec<PathBuf> = files(root, root)?
        .into_iter()
        .filter(|file| !in_dot_folder(file) || incoming.contains(file))
        .collect();
    let mut renamed: Vec<(PathBuf, PathBuf)> = Vec::new();
    let moves = outgoing
        .iter()
        .map(|file| (root.join(file), aside(&root.join(file))))
        .chain(
            incoming
                .iter()
                .map(|file| (staging.join(file), root.join(file))),
        );
    for (from, to) in moves {
        let moved = match to.parent() {
            Some(parent) => fs::create_dir_all(parent),
            None => Ok(()),
        }
        .and_then(|()| fs::rename(&from, &to));
        if let Err(error) = moved {
            for (from, to) in renamed.iter().rev() {
                let _ = fs::rename(to, from);
            }
            return Err(Failure::UpdateApplyFailed(to, error));
        }
        renamed.push((from, to));
    }
    clean(root);
    Ok(())
}

fn clean(root: &Path) {
    let _ = fs::remove_dir_all(root.join(STAGING));
    remove_aside_and_empty(root);
}

fn remove_aside_and_empty(folder: &Path) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() && entry.file_name() != ".git" {
            remove_aside_and_empty(&path);
            let _ = fs::remove_dir(&path);
        } else if kind.is_file() && entry.file_name().to_string_lossy().ends_with(ASIDE) {
            let _ = fs::remove_file(&path);
        }
    }
}

fn files(folder: &Path, root: &Path) -> Result<Vec<PathBuf>, Failure> {
    let unreadable = |error| Failure::UpdateApplyFailed(folder.to_path_buf(), error);
    let mut found = Vec::new();
    for entry in fs::read_dir(folder).map_err(unreadable)? {
        let entry = entry.map_err(unreadable)?;
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let kind = entry.file_type().map_err(unreadable)?;
        if relative == Path::new(STAGING) || relative == Path::new(".git") {
            continue;
        }
        if kind.is_dir() {
            found.extend(files(&path, root)?);
        } else if !entry.file_name().to_string_lossy().ends_with(ASIDE) {
            found.push(relative);
        }
    }
    Ok(found)
}

fn in_dot_folder(file: &Path) -> bool {
    let mut components = file.components();
    let first = components.next();
    components.next().is_some()
        && first.is_some_and(|first| first.as_os_str().to_string_lossy().starts_with('.'))
}

fn aside(file: &Path) -> PathBuf {
    let mut name = OsString::from(file.as_os_str());
    name.push(ASIDE);
    PathBuf::from(name)
}
