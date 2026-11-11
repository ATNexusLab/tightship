//! The hooks a harness runs, and how their text reaches it: plain or inside the harness's JSON shape.

pub mod checkpoint;
pub mod clean;
pub mod contract_part;
pub mod export;
pub mod map;
pub mod marketplace_auto_update;
pub mod settings;
pub mod start;
pub mod status_line;
pub mod update;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::ops::BitOr;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::system::env::optional_variable;

/// Prints the text for the harness, inside the JSON shape `TIGHTSHIP_HOOK_OUTPUT` names when it is set.
///
/// # Errors
///
/// When the shape file cannot be read or has no `{text}` slot.
pub fn emit(text: &str) -> Result<(), Failure> {
    emit_shaped(text, template("TIGHTSHIP_HOOK_OUTPUT"))
}

/// Prints the memory checkpoint, in the shape `TIGHTSHIP_CHECKPOINT_OUTPUT` names, or else as `emit` does.
///
/// # Errors
///
/// When the shape file cannot be read or has no `{text}` slot.
pub fn emit_checkpoint(text: &str) -> Result<(), Failure> {
    let shape =
        template("TIGHTSHIP_CHECKPOINT_OUTPUT").or_else(|| template("TIGHTSHIP_HOOK_OUTPUT"));
    emit_shaped(text, shape)
}

fn template(variable: &str) -> Option<PathBuf> {
    optional_variable(variable)
}

fn emit_shaped(text: &str, template: Option<PathBuf>) -> Result<(), Failure> {
    if text.is_empty() {
        return Ok(());
    }
    let Some(template) = template else {
        if text.ends_with('\n') {
            print!("{text}");
        } else {
            println!("{text}");
        }
        return Ok(());
    };
    let shape = fs::read_to_string(&template)
        .map_err(|error| Failure::HookOutputUnreadable(template.clone(), error))?;
    let mut payload: serde_json::Value =
        serde_json::from_str(&shape).map_err(|_| Failure::HookOutputInvalid(template.clone()))?;
    if !fill(&mut payload, text) {
        return Err(Failure::HookOutputInvalid(template));
    }
    println!("{payload}");
    Ok(())
}

fn fill(value: &mut serde_json::Value, text: &str) -> bool {
    match value {
        serde_json::Value::String(slot) if slot == "{text}" => {
            *slot = text.to_owned();
            true
        }
        serde_json::Value::Array(items) => items
            .iter_mut()
            .map(|item| fill(item, text))
            .fold(false, BitOr::bitor),
        serde_json::Value::Object(fields) => fields
            .values_mut()
            .map(|field| fill(field, text))
            .fold(false, BitOr::bitor),
        _ => false,
    }
}

fn export_vault(folder: &Path) -> Result<(), Failure> {
    let Some(file) = optional_variable("TIGHTSHIP_ENV_FILE") else {
        return Ok(());
    };
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .and_then(|mut opened| writeln!(opened, "export TIGHTSHIP_VAULT={}", quoted(folder)))
        .map_err(|error| Failure::EnvUnwritable(file, error))
}

fn quoted(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}
