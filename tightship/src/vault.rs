//! The memory vault: its folder and the `tightship vault` commands.

pub mod context;
pub mod expire;
pub mod handoff;
pub mod init;
pub mod lint;
pub mod new;
pub mod note;
pub mod path;
pub mod rules;
pub mod search;
pub mod sync;

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::failure::Failure;
use crate::options::Options;

/// The vault folder the options set.
///
/// # Errors
///
/// When the options are invalid or `folder_from` fails.
pub fn folder() -> Result<PathBuf, Failure> {
    folder_from(&crate::options::load()?)
}

/// The vault folder of the options, checked to be an absolute path to an existing folder.
///
/// # Errors
///
/// When no folder is set, it is relative, it is missing or not a folder, or it cannot be read.
pub fn folder_from(options: &Options) -> Result<PathBuf, Failure> {
    let folder = options.vault_folder.clone().ok_or(Failure::VaultNotSet)?;
    if !folder.is_absolute() {
        return Err(Failure::VaultNotAbsolute(folder));
    }
    match fs::metadata(&folder) {
        Ok(metadata) if metadata.is_dir() => Ok(folder),
        Ok(_) => Err(Failure::VaultMissing(folder)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(Failure::VaultMissing(folder)),
        Err(error) => Err(Failure::VaultUnreadable(folder, error)),
    }
}
