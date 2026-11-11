//! The plugin's options: read from the environment, then `options.json` or `options.toml`, and set by the user.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::env::{expand_tilde, optional_variable};
use crate::failure::Failure;
use crate::files::{pretty_json, read_if_present, strip_byte_order_mark};

const OPTION_NAMES: &[&str] = &[
    "contract",
    "memory",
    "status_line",
    "auto_update",
    "vault_folder",
];

fn options_json() -> Result<PathBuf, Failure> {
    Ok(crate::env::variable("TIGHTSHIP_DATA")?.join("options.json"))
}

/// Whether an on/off option is off, for `tightship options disabled`.
///
/// # Errors
///
/// When the name is unknown or names the vault folder, or the options are invalid.
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

/// Stores an option's value in `options.json`.
///
/// # Errors
///
/// When the name is unknown, an on/off value is not `true` or `false`, or the file cannot be read or written.
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

/// Prints the options stored in `options.json`.
///
/// # Errors
///
/// When the file cannot be read or is invalid.
pub fn list() -> Result<(), Failure> {
    let table = read_table(&options_json()?)?;
    let text = pretty_json(&table).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    print!("{text}");
    Ok(())
}

/// Removes `options.json`, so every option goes back to its default.
///
/// # Errors
///
/// When the file exists and cannot be removed.
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
        _ => Err(Failure::OptionsNotBoolean(name.to_owned())),
    }
}

fn read_table(path: &Path) -> Result<serde_json::Map<String, Value>, Failure> {
    Ok(read_file(path)?.unwrap_or_default())
}

fn read_file(path: &Path) -> Result<Option<serde_json::Map<String, Value>>, Failure> {
    let Some(text) = read_if_present(path)
        .map_err(|error| Failure::OptionsUnreadable(path.to_path_buf(), error))?
    else {
        return Ok(None);
    };
    let text = strip_byte_order_mark(&text);
    let value = if path
        .extension()
        .is_some_and(|extension| extension == "toml")
    {
        let invalid = |_| Failure::OptionsTomlInvalid(path.to_path_buf());
        let table = text.parse::<toml::Table>().map_err(invalid)?;
        serde_json::to_value(table).map_err(|_| Failure::OptionsTomlInvalid(path.to_path_buf()))?
    } else {
        serde_json::from_str(text).map_err(|_| Failure::OptionsJsonInvalid(path.to_path_buf()))?
    };
    match value {
        Value::Object(table) => Ok(Some(table)),
        _ => Err(Failure::OptionsJsonInvalid(path.to_path_buf())),
    }
}

fn write_table(path: &Path, table: &serde_json::Map<String, Value>) -> Result<(), Failure> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    }
    let text = pretty_json(table).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    fs::write(path, text).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    Ok(())
}

/// The options a command runs with.
pub struct Options {
    pub contract: bool,
    pub memory: bool,
    pub status_line: bool,
    pub auto_update: bool,
    pub vault_folder: Option<PathBuf>,
    file: PathBuf,
    table: serde_json::Map<String, Value>,
}

impl Options {
    /// Whether an on/off option is on: its `TIGHTSHIP_<NAME>` variable first, then the file, then on.
    ///
    /// # Errors
    ///
    /// When the file holds a value that is not a boolean.
    pub fn is_enabled(&self, name: &str) -> Result<bool, Failure> {
        let variable = format!("TIGHTSHIP_{}", name.to_ascii_uppercase().replace('-', "_"));
        if let Some(value) = env::var_os(variable).filter(|value| !value.is_empty()) {
            return Ok(value != "false");
        }
        match self.table.get(name) {
            None => Ok(true),
            Some(Value::Bool(value)) => Ok(*value),
            Some(_) => Err(Failure::OptionsFileValueInvalid(
                self.file.clone(),
                name.to_owned(),
            )),
        }
    }
}

/// Reads the options, with the environment over the file and the file over the defaults.
///
/// # Errors
///
/// When the options file cannot be read or holds an invalid value.
pub fn load() -> Result<Options, Failure> {
    let (file, table) = match optional_variable("TIGHTSHIP_DATA") {
        None => (PathBuf::new(), serde_json::Map::new()),
        Some(data) => {
            let json_file = data.join("options.json");
            let toml_file = data.join("options.toml");
            match read_file(&json_file)? {
                Some(table) => (json_file, table),
                None => {
                    let table = read_file(&toml_file)?.unwrap_or_default();
                    (toml_file, table)
                }
            }
        }
    };

    let file_vault_folder = match table.get("vault_folder") {
        None => None,
        Some(Value::String(value)) if value.trim().is_empty() => None,
        Some(Value::String(value)) => Some(expand_tilde(Path::new(value.trim()))),
        Some(_) => {
            return Err(Failure::OptionsFileValueInvalid(
                file,
                "vault_folder".to_owned(),
            ));
        }
    };
    let vault_folder = optional_variable("TIGHTSHIP_VAULT")
        .map(|value| expand_tilde(&value))
        .or(file_vault_folder);

    let mut options = Options {
        contract: true,
        memory: true,
        status_line: true,
        auto_update: true,
        vault_folder,
        file,
        table,
    };
    options.contract = options.is_enabled("contract")?;
    options.memory = options.is_enabled("memory")?;
    options.status_line = options.is_enabled("status_line")?;
    options.auto_update = options.is_enabled("auto_update")?;
    Ok(options)
}
