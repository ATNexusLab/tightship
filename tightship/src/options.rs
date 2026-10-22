use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::failure::Failure;

const OPTION_NAMES: &[&str] = &["contract", "memory", "status_line", "vault_folder"];
const BYTE_ORDER_MARK: char = '\u{feff}';

fn options_json() -> Result<PathBuf, Failure> {
    Ok(crate::variable("TIGHTSHIP_DATA")?.join("options.json"))
}

pub fn disabled(name: &str) -> Result<bool, Failure> {
    validate_name(name)?;
    if name == "vault_folder" {
        return Err(Failure::Usage(
            "vault_folder is a folder, not an on/off option".to_owned(),
        ));
    }
    Ok(!load()?.is_enabled(name)?)
}

fn validate_name(name: &str) -> Result<(), Failure> {
    if OPTION_NAMES.contains(&name) {
        Ok(())
    } else {
        Err(Failure::OptionsUnknown(name.to_owned()))
    }
}

pub fn set(name: &str, value: &str) -> Result<(), Failure> {
    validate_name(name)?;
    let path = options_json()?;
    let mut table = read_table(&path)?;
    let display = if name == "vault_folder" {
        let expanded = expand_tilde(Path::new(value.trim()));
        let stored = expanded.to_string_lossy().into_owned();
        table.insert(name.to_owned(), Value::String(stored.clone()));
        stored
    } else {
        let enabled = parse_boolean_value(name, value)?;
        table.insert(name.to_owned(), Value::Bool(enabled));
        enabled.to_string()
    };
    write_table(&path, &table)?;
    println!("Set {name} to {display}.");
    Ok(())
}

pub fn list() -> Result<(), Failure> {
    let table = read_table(&options_json()?)?;
    let text = serde_json::to_string_pretty(&Value::Object(table))
        .map_err(|error| Failure::OptionsIo(error.to_string()))?;
    println!("{text}");
    Ok(())
}

pub fn reset() -> Result<(), Failure> {
    let path = options_json()?;
    if path.is_file() {
        fs::remove_file(&path).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    }
    println!("Removed {}.", path.display());
    Ok(())
}

fn parse_boolean_value(name: &str, value: &str) -> Result<bool, Failure> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Failure::OptionsInvalidValue(name.to_owned())),
    }
}

fn read_table(path: &Path) -> Result<serde_json::Map<String, Value>, Failure> {
    if !path.is_file() {
        return Ok(serde_json::Map::new());
    }
    let text = fs::read_to_string(path).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    let value = serde_json::from_str::<Value>(text.trim_start_matches(BYTE_ORDER_MARK))
        .map_err(|_| Failure::OptionsJsonInvalid(path.to_path_buf()))?;
    match value {
        Value::Object(table) => Ok(table),
        _ => Err(Failure::OptionsJsonInvalid(path.to_path_buf())),
    }
}

fn write_table(path: &Path, table: &serde_json::Map<String, Value>) -> Result<(), Failure> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    }
    let text = serde_json::to_string_pretty(&Value::Object(table.clone()))
        .map_err(|error| Failure::OptionsIo(error.to_string()))?;
    fs::write(path, format!("{text}\n")).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Options {
    pub contract: bool,
    pub memory: bool,
    pub status_line: bool,
    pub vault_folder: Option<PathBuf>,
    file: PathBuf,
    table: toml::Table,
    json: Option<serde_json::Map<String, Value>>,
}

impl Options {
    pub fn is_enabled(&self, name: &str) -> Result<bool, Failure> {
        let variable = format!("TIGHTSHIP_{}", name.to_ascii_uppercase().replace('-', "_"));
        if let Some(value) = env::var_os(variable).filter(|value| !value.is_empty()) {
            return Ok(value != "false");
        }
        if let Some(table) = &self.json {
            return json_boolean(table, name, &self.file);
        }
        match self.table.get(name) {
            Some(toml::Value::Boolean(value)) => Ok(*value),
            Some(_) => Err(Failure::OptionsInvalid(self.file.clone())),
            None => Ok(true),
        }
    }
}

fn json_boolean(
    table: &serde_json::Map<String, Value>,
    name: &str,
    file: &Path,
) -> Result<bool, Failure> {
    match table.get(name) {
        None => Ok(true),
        Some(Value::Bool(false)) => Ok(false),
        Some(Value::Bool(true)) => Ok(true),
        Some(_) => Err(Failure::OptionsJsonInvalid(file.to_path_buf())),
    }
}

fn json_vault_folder(
    table: &serde_json::Map<String, Value>,
    file: &Path,
) -> Result<Option<PathBuf>, Failure> {
    match table.get("vault_folder") {
        None => Ok(None),
        Some(Value::String(value)) if value.trim().is_empty() => Ok(None),
        Some(Value::String(value)) => Ok(Some(expand_tilde(Path::new(value.trim())))),
        Some(_) => Err(Failure::OptionsJsonInvalid(file.to_path_buf())),
    }
}

pub fn load() -> Result<Options, Failure> {
    let data_dir = env::var_os("TIGHTSHIP_DATA")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let json_file = data_dir
        .as_ref()
        .map(|data| data.join("options.json"))
        .unwrap_or_default();
    let toml_file = data_dir
        .as_ref()
        .map(|data| data.join("options.toml"))
        .unwrap_or_else(|| PathBuf::from("options.toml"));

    let (table, json, file) = if json_file.is_file() {
        let text = fs::read_to_string(&json_file)
            .map_err(|error| Failure::OptionsUnreadable(json_file.clone(), error))?;
        let value = serde_json::from_str::<Value>(text.trim_start_matches(BYTE_ORDER_MARK))
            .map_err(|_| Failure::OptionsJsonInvalid(json_file.clone()))?;
        let map = match value {
            Value::Object(table) => table,
            _ => return Err(Failure::OptionsJsonInvalid(json_file)),
        };
        (toml::Table::new(), Some(map), json_file)
    } else if toml_file.is_file() {
        let text = fs::read_to_string(&toml_file)
            .map_err(|error| Failure::OptionsUnreadable(toml_file.clone(), error))?;
        let table = text
            .parse::<toml::Table>()
            .map_err(|_| Failure::OptionsInvalid(toml_file.clone()))?;
        (table, None, toml_file)
    } else {
        (toml::Table::new(), None, toml_file)
    };

    let file_vault_folder = if let Some(map) = &json {
        json_vault_folder(map, &file)?
    } else {
        match table.get("vault_folder") {
            Some(toml::Value::String(value)) if value.trim().is_empty() => None,
            Some(toml::Value::String(value)) => Some(expand_tilde(Path::new(value.trim()))),
            Some(_) => return Err(Failure::OptionsInvalid(file.clone())),
            None => None,
        }
    };
    let vault_folder = env::var_os("TIGHTSHIP_VAULT")
        .filter(|value| !value.is_empty())
        .map(|value| expand_tilde(Path::new(&value)))
        .or(file_vault_folder);

    let mut options = Options {
        contract: true,
        memory: true,
        status_line: true,
        vault_folder,
        file,
        table,
        json,
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
