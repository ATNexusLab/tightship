use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::diagnostics::naming;

pub fn staging_dir(root: &Path, harness_name: &str) -> PathBuf {
    root.join("dist").join(format!(".{harness_name}.tmp"))
}

pub fn remove_if_present(folder: &Path) -> io::Result<()> {
    match fs::remove_dir_all(folder) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(naming(folder)(error)),
        _ => Ok(()),
    }
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
            let path = relative.to_string_lossy().replace('\\', "/");
            return Err(io::Error::other(format!(
                "dist/{harness_name}/{path}: two sources write this file"
            )));
        }
    }
    let bin = staging.join("bin");
    for (file, bytes) in files {
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder).map_err(naming(folder))?;
        }
        fs::write(file, bytes).map_err(naming(file))?;
        if file.starts_with(&bin) {
            make_executable(file)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(file: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(file, fs::Permissions::from_mode(0o755)).map_err(naming(file))
}

#[cfg(not(unix))]
fn make_executable(_file: &Path) -> io::Result<()> {
    Ok(())
}

pub fn publish(root: &Path, staging: &Path, harness_name: &str) -> io::Result<()> {
    let target = root.join("dist").join(harness_name);
    remove_if_present(&target)?;
    fs::rename(staging, &target).map_err(naming(&target))
}

pub fn discard(root: &Path, harness_name: &str) -> io::Result<()> {
    let dist = root.join("dist");
    remove_if_present(&dist.join(format!(".{harness_name}.tmp")))?;
    remove_if_present(&dist.join(harness_name))
}
