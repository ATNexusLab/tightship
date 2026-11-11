//! File system helpers the commands share.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::diagnostics::with_path;

/// Every file under a folder, sorted.
///
/// # Errors
///
/// When a folder cannot be read, or an entry is a symbolic link.
pub fn files_under(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(directory)
        .map_err(with_path(directory))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<PathBuf>>>()?;
    entries.sort();
    let mut files = Vec::new();
    for entry in entries {
        let file_type = fs::symlink_metadata(&entry)
            .map_err(with_path(&entry))?
            .file_type();
        if file_type.is_symlink() {
            return Err(io::Error::other(format!(
                "{} is a symbolic link; the build reads only real files",
                entry.display()
            )));
        }
        if file_type.is_dir() {
            files.extend(files_under(&entry)?);
        } else {
            files.push(entry);
        }
    }
    Ok(files)
}

/// Removes a folder and everything in it; a folder that is not there is fine.
///
/// # Errors
///
/// When the folder is there but cannot be removed.
pub fn remove_if_present(folder: &Path) -> io::Result<()> {
    match fs::remove_dir_all(folder) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(with_path(folder)(error)),
        _ => Ok(()),
    }
}

/// A path with `/` between its parts on every platform.
pub fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
