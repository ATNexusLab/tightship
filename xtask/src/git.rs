//! Runs git in the repository.

use std::io;
use std::path::Path;
use std::process::Command;

/// Runs git with these arguments and returns what it printed.
///
/// # Errors
///
/// When git cannot run or exits with an error.
pub fn run(root: &Path, arguments: &[&str]) -> io::Result<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| io::Error::new(error.kind(), format!("git: {error}")))?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git {}: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
