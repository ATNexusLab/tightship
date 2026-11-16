//! Running `git`, never prompting and printing paths as they are.

use std::io;
use std::path::Path;
use std::process::Command;

/// Whether a `git` executable is on the `PATH`.
pub fn is_installed() -> bool {
    let probe = Command::new("git").arg("--version").output();
    !matches!(probe, Err(error) if error.kind() == io::ErrorKind::NotFound)
}

/// A `git` command that runs in the folder, never asks for credentials, and prints paths unquoted.
pub fn command(folder: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(folder)
        .args(["-c", "core.quotePath=false"])
        .env("GIT_TERMINAL_PROMPT", "0");
    command
}

/// Runs `git` in the folder and returns what it printed.
///
/// # Errors
///
/// When `git` cannot start or exits with an error; the message is what it printed to stderr.
pub fn run(folder: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = command(folder)
        .args(arguments)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
