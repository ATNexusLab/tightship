use std::fs;
use std::path::PathBuf;

use crate::failure::Failure;

pub fn cursor_options_path() -> Result<PathBuf, Failure> {
    let home = std::env::home_dir().ok_or_else(|| Failure::OptionsIo("HOME is not set".to_owned()))?;
    Ok(home
        .join(".cursor")
        .join("tightship")
        .join("options.json"))
}

pub fn cursor_boolean(name: &str) -> Result<bool, Failure> {
    let path = cursor_options_path()?;
    if !path.is_file() {
        return Ok(true);
    }
    let text = fs::read_to_string(&path).map_err(|error| Failure::OptionsIo(error.to_string()))?;
    let value = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|error| Failure::OptionsIo(error.to_string()))?;
    match value.get(name) {
        None => Ok(true),
        Some(serde_json::Value::Bool(false)) => Ok(false),
        Some(serde_json::Value::Bool(true)) => Ok(true),
        Some(_) => Err(Failure::OptionsInvalidValue(name.to_owned())),
    }
}
