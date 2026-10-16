use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub contract: bool,
    pub memory: bool,
    pub status_line: bool,
    pub vault_folder: Option<PathBuf>,
}

pub fn load() -> Result<Options, Failure> {
    let options_path = options_file();
    let table = match options_path.as_deref() {
        Some(path) if path.is_file() => {
            let text = fs::read_to_string(path)
                .map_err(|_| Failure::OptionsInvalid(path.to_path_buf()))?;
            let parsed: toml::Table = text
                .parse()
                .map_err(|_| Failure::OptionsInvalid(path.to_path_buf()))?;
            Some((path, parsed))
        }
        _ => None,
    };

    let (toml_contract, toml_memory, toml_status_line, toml_vault_folder) = match table {
        Some((path, table)) => {
            let contract = match table.get("contract") {
                Some(toml::Value::Boolean(value)) => Some(*value),
                Some(_) => return Err(Failure::OptionsInvalid(path.to_path_buf())),
                None => None,
            };
            let memory = match table.get("memory") {
                Some(toml::Value::Boolean(value)) => Some(*value),
                Some(_) => return Err(Failure::OptionsInvalid(path.to_path_buf())),
                None => None,
            };
            let status_line = match table.get("status_line") {
                Some(toml::Value::Boolean(value)) => Some(*value),
                Some(_) => return Err(Failure::OptionsInvalid(path.to_path_buf())),
                None => None,
            };
            let vault_folder = match table.get("vault_folder") {
                Some(toml::Value::String(value)) => {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(expand_tilde(Path::new(trimmed)))
                    }
                }
                Some(_) => return Err(Failure::OptionsInvalid(path.to_path_buf())),
                None => None,
            };
            (contract, memory, status_line, vault_folder)
        }
        None => (None, None, None, None),
    };

    let contract = env::var_os("TIGHTSHIP_CONTRACT")
        .filter(|value| !value.is_empty())
        .map(|value| value != "false")
        .or(toml_contract)
        .unwrap_or(true);

    let memory = env::var_os("TIGHTSHIP_MEMORY")
        .filter(|value| !value.is_empty())
        .map(|value| value != "false")
        .or(toml_memory)
        .unwrap_or(true);

    let status_line = env::var_os("TIGHTSHIP_STATUS_LINE")
        .filter(|value| !value.is_empty())
        .map(|value| value != "false")
        .or(toml_status_line)
        .unwrap_or(true);

    let vault_folder = env::var_os("TIGHTSHIP_VAULT")
        .filter(|value| !value.is_empty())
        .map(|value| expand_tilde(Path::new(&value)))
        .or(toml_vault_folder);

    Ok(Options {
        contract,
        memory,
        status_line,
        vault_folder,
    })
}

pub fn is_option_enabled(name: &str) -> Result<bool, Failure> {
    let env_name = format!("TIGHTSHIP_{}", name.to_ascii_uppercase().replace('-', "_"));
    if let Some(value) = env::var_os(&env_name).filter(|value| !value.is_empty()) {
        return Ok(value != "false");
    }

    let Some(path) = options_file() else {
        return Ok(true);
    };
    if !path.is_file() {
        return Ok(true);
    }

    let text = fs::read_to_string(&path).map_err(|_| Failure::OptionsInvalid(path.clone()))?;
    let parsed: toml::Table = text
        .parse()
        .map_err(|_| Failure::OptionsInvalid(path.clone()))?;

    match parsed.get(name) {
        Some(toml::Value::Boolean(value)) => Ok(*value),
        Some(_) => Err(Failure::OptionsInvalid(path)),
        None => Ok(true),
    }
}

fn options_file() -> Option<PathBuf> {
    env::var_os("TIGHTSHIP_DATA")
        .filter(|value| !value.is_empty())
        .map(|data| PathBuf::from(data).join("options.toml"))
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

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(profile) = env::var_os("USERPROFILE").filter(|value| !value.is_empty()) {
            return Some(PathBuf::from(profile));
        }
    }
    env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
