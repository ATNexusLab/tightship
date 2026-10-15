pub mod context;
pub mod init;
pub mod lint;
pub mod new;
pub mod note;
pub mod path;
pub mod search;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::failure::Failure;

pub fn folder() -> Result<PathBuf, Failure> {
    let folder = env::var_os("TIGHTSHIP_VAULT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or(Failure::VaultNotSet)?;
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
