use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

#[derive(Debug, Clone)]
pub struct Options {
    pub contract: bool,
    pub memory: bool,
    pub status_line: bool,
    pub vault_folder: Option<PathBuf>,
    file: PathBuf,
    table: toml::Table,
}

impl Options {
    pub fn is_enabled(&self, name: &str) -> Result<bool, Failure> {
        let variable = format!("TIGHTSHIP_{}", name.to_ascii_uppercase().replace('-', "_"));
        if let Some(value) = env::var_os(variable).filter(|value| !value.is_empty()) {
            return Ok(value != "false");
        }
        match self.table.get(name) {
            Some(toml::Value::Boolean(value)) => Ok(*value),
            Some(_) => Err(Failure::OptionsInvalid(self.file.clone())),
            None => Ok(true),
        }
    }
}

pub fn load() -> Result<Options, Failure> {
    let file = env::var_os("TIGHTSHIP_DATA")
        .filter(|value| !value.is_empty())
        .map(|data| PathBuf::from(data).join("options.toml"))
        .unwrap_or_default();
    let table = if file.is_file() {
        let text = fs::read_to_string(&file)
            .map_err(|error| Failure::OptionsUnreadable(file.clone(), error))?;
        text.parse::<toml::Table>()
            .map_err(|_| Failure::OptionsInvalid(file.clone()))?
    } else {
        toml::Table::new()
    };

    let toml_vault_folder = match table.get("vault_folder") {
        Some(toml::Value::String(value)) if value.trim().is_empty() => None,
        Some(toml::Value::String(value)) => Some(expand_tilde(Path::new(value.trim()))),
        Some(_) => return Err(Failure::OptionsInvalid(file)),
        None => None,
    };
    let vault_folder = env::var_os("TIGHTSHIP_VAULT")
        .filter(|value| !value.is_empty())
        .map(|value| expand_tilde(Path::new(&value)))
        .or(toml_vault_folder);

    let mut options = Options {
        contract: true,
        memory: true,
        status_line: true,
        vault_folder,
        file,
        table,
    };
    options.contract = options.is_enabled("contract")?;
    options.memory = options.is_enabled("memory")?;
    options.status_line = options.is_enabled("status_line")?;
    Ok(options)
}

fn expand_tilde(path: &Path) -> PathBuf {
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
