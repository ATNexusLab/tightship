//! Reading the environment: the plugin's variables and the user's home folder.

use std::env;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

/// The value of a variable the command needs, as a path.
///
/// # Errors
///
/// When the variable is unset or empty.
pub fn variable(name: &'static str) -> Result<PathBuf, Failure> {
    optional_variable(name).ok_or(Failure::VariableNotSet(name))
}

/// The value of a variable as a path, or `None` when it is unset or empty.
pub fn optional_variable(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// The path with a leading `~` replaced by the home folder, when there is one.
pub fn expand_tilde(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str == "~"
        && let Some(home) = home_dir()
    {
        return home;
    } else if let Some(rest) = path_str
        .strip_prefix("~/")
        .or_else(|| path_str.strip_prefix(r"~\"))
        && let Some(home) = home_dir()
    {
        return home.join(rest);
    }
    path.to_path_buf()
}

/// The user's home folder: `USERPROFILE` on Windows, then `HOME`.
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(profile) = optional_variable("USERPROFILE") {
            return Some(profile);
        }
    }
    optional_variable("HOME")
}
