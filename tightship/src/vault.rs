pub mod context;
pub mod handoff;
pub mod init;
pub mod lint;
pub mod new;
pub mod note;
pub mod path;
pub mod search;
pub mod sync;

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::failure::Failure;
use crate::options::Options;

pub fn folder() -> Result<PathBuf, Failure> {
    folder_from(&crate::options::load()?)
}

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
