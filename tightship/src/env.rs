use std::env;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

pub fn variable(name: &'static str) -> Result<PathBuf, Failure> {
    optional_variable(name).ok_or(Failure::VariableNotSet(name))
}

pub fn optional_variable(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

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

pub fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(profile) = optional_variable("USERPROFILE") {
            return Some(profile);
        }
    }
    optional_variable("HOME")
}
