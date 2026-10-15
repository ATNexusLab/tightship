use std::io;
use std::path::Path;
use std::process::Command;

pub fn is_installed() -> bool {
    let probe = Command::new("git").arg("--version").output();
    !matches!(probe, Err(error) if error.kind() == io::ErrorKind::NotFound)
}

pub fn run(folder: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(folder)
        .args(arguments)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
