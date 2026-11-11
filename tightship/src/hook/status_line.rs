//! The status-line hook: puts tightship's status line in the harness's settings, or gives the user's back.

use std::env;
use std::path::Path;

use serde_json::{Map, Value};

use super::settings;
use crate::failure::Failure;

/// Turns the status line on, refreshes it, or puts back the one it replaced, as the option says.
///
/// # Errors
///
/// When the options, the settings, or the record are invalid, or the settings cannot be written.
pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    let settings_path = crate::env::variable("TIGHTSHIP_SETTINGS")?;
    let data = crate::env::variable("TIGHTSHIP_DATA")?;
    let root = crate::env::variable("TIGHTSHIP_ROOT")?;
    let previous = settings::read_previous(&data)?;
    let user_settings = settings::read_settings(&settings_path)?;
    let ours = user_settings
        .get("statusLine")
        .is_some_and(settings::is_ours);
    if !options.status_line {
        return turn_off(&settings_path, &data, user_settings, previous, ours);
    }
    let command = Value::String(settings::command_for(&root)?);
    let refresh_interval = env::var("TIGHTSHIP_STATUS_LINE_REFRESH")
        .ok()
        .and_then(|seconds| seconds.parse::<u64>().ok());
    if ours {
        return refresh(&settings_path, user_settings, command, refresh_interval);
    }
    take_over(
        &settings_path,
        &data,
        user_settings,
        command,
        refresh_interval,
    )
}

fn turn_off(
    settings_path: &Path,
    data: &Path,
    mut user_settings: Map<String, Value>,
    previous: Option<Value>,
    ours: bool,
) -> Result<(), Failure> {
    if ours {
        match previous {
            Some(previous) => {
                user_settings.insert("statusLine".to_owned(), previous);
            }
            None => {
                user_settings.shift_remove("statusLine");
            }
        }
        settings::write_settings(settings_path, &user_settings)?;
    }
    settings::remove_record(data)
}

fn refresh(
    settings_path: &Path,
    mut user_settings: Map<String, Value>,
    command: Value,
    refresh_interval: Option<u64>,
) -> Result<(), Failure> {
    let status_line = user_settings
        .get_mut("statusLine")
        .and_then(Value::as_object_mut);
    if let Some(status_line) = status_line {
        let stale = status_line.get("command") != Some(&command);
        let unpaced = refresh_interval.filter(|_| !status_line.contains_key("refreshInterval"));
        if stale {
            status_line.insert("command".to_owned(), command);
        }
        if let Some(seconds) = unpaced {
            status_line.insert("refreshInterval".to_owned(), seconds.into());
        }
        if stale || unpaced.is_some() {
            settings::write_settings(settings_path, &user_settings)?;
        }
    }
    Ok(())
}

fn take_over(
    settings_path: &Path,
    data: &Path,
    mut user_settings: Map<String, Value>,
    command: Value,
    refresh_interval: Option<u64>,
) -> Result<(), Failure> {
    // The record goes first: settings pointing at tightship without it could never be put back.
    settings::write_record(data, user_settings.get("statusLine"))?;
    let mut status_line = serde_json::json!({ "type": "command", "command": command });
    if let Some(seconds) = refresh_interval {
        status_line["refreshInterval"] = seconds.into();
    }
    user_settings.insert("statusLine".to_owned(), status_line);
    if let Err(failure) = settings::write_settings(settings_path, &user_settings) {
        settings::remove_record(data)?;
        return Err(failure);
    }
    Ok(())
}
