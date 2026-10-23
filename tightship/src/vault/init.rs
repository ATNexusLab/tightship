use std::fs;
use std::io;
use std::path::{self, Path};

use crate::failure::Failure;
use crate::git;

pub fn run(folder: &Path) -> Result<(), Failure> {
    let unwritable = |error: io::Error| Failure::InitUnwritable(folder.to_path_buf(), error);
    if !git::is_installed() {
        return Err(Failure::InitNoGit);
    }
    let is_empty_folder = match fs::symlink_metadata(folder) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => true,
        Err(error) => return Err(unwritable(error)),
        Ok(metadata) if metadata.is_dir() => {
            fs::read_dir(folder).map_err(unwritable)?.next().is_none()
        }
        Ok(_) => false,
    };
    if !is_empty_folder {
        return Err(Failure::InitNotEmpty(folder.to_path_buf()));
    }
    fs::create_dir_all(folder.join("notes").join("all")).map_err(unwritable)?;
    fs::write(folder.join("AGENTS.md"), super::rules::block()).map_err(unwritable)?;
    git::run(folder, &["init", "--quiet"])
        .map_err(|message| Failure::InitGitFailed(folder.to_path_buf(), message))?;
    let absolute = path::absolute(folder).map_err(unwritable)?;
    println!(
        "Created the vault at {}. Commit it, push it to a private remote to share it across machines,\nand set this folder as the plugin's vault folder.",
        absolute.display()
    );
    Ok(())
}
