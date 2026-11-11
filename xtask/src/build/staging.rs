use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::diagnostics::with_path;
use crate::files::{remove_if_present, slash_path};

pub fn staging_dir(root: &Path, harness_name: &str) -> PathBuf {
    root.join("dist").join(format!(".{harness_name}.tmp"))
}

pub fn write_files(
    staging: &Path,
    harness_name: &str,
    files: &[(PathBuf, Vec<u8>)],
) -> io::Result<()> {
    let mut paths = HashSet::new();
    for (file, _) in files {
        if !paths.insert(file) {
            let relative = file.strip_prefix(staging).map_err(io::Error::other)?;
            let path = slash_path(relative);
            return Err(io::Error::other(format!(
                "dist/{harness_name}/{path}: two sources write this file"
            )));
        }
    }
    let bin = staging.join("bin");
    for (file, bytes) in files {
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder).map_err(with_path(folder))?;
        }
        fs::write(file, bytes).map_err(with_path(file))?;
        if file.starts_with(&bin) {
            make_executable(file)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(file: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(file, fs::Permissions::from_mode(0o755)).map_err(with_path(file))
}

#[cfg(not(unix))]
fn make_executable(_file: &Path) -> io::Result<()> {
    Ok(())
}

pub fn publish(root: &Path, staging: &Path, harness_name: &str) -> io::Result<()> {
    let target = root.join("dist").join(harness_name);
    remove_if_present(&target)?;
    fs::rename(staging, &target).map_err(with_path(&target))
}

pub fn discard(root: &Path, harness_name: &str) -> io::Result<()> {
    let dist = root.join("dist");
    remove_if_present(&staging_dir(root, harness_name))?;
    remove_if_present(&dist.join(harness_name))
}
