use std::env;

use serde_json::Value;

use super::settings;
use crate::failure::Failure;

pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    let settings_path = crate::env::variable("TIGHTSHIP_SETTINGS")?;
    let data = crate::env::variable("TIGHTSHIP_DATA")?;
    let root = crate::env::variable("TIGHTSHIP_ROOT")?;
    let previous = settings::read_previous(&data)?;
    let mut user_settings = settings::read_settings(&settings_path)?;
    let ours = user_settings
        .get("statusLine")
        .is_some_and(settings::is_ours);
    if !options.status_line {
        if ours {
            match previous {
                Some(previous) => {
                    user_settings.insert("statusLine".to_owned(), previous);
                }
                None => {
                    user_settings.shift_remove("statusLine");
                }
            }
            settings::write_settings(&settings_path, &user_settings)?;
        }
        return settings::remove_record(&data);
    }
    let command = Value::String(settings::command_for(&root)?);
    let refresh_interval = env::var("TIGHTSHIP_STATUS_LINE_REFRESH")
        .ok()
        .and_then(|seconds| seconds.parse::<u64>().ok());
    if ours {
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
                settings::write_settings(&settings_path, &user_settings)?;
            }
        }
        return Ok(());
    }
    settings::write_record(&data, user_settings.get("statusLine"))?;
    let mut status_line = serde_json::json!({ "type": "command", "command": command });
    if let Some(seconds) = refresh_interval {
        status_line["refreshInterval"] = seconds.into();
    }
    user_settings.insert("statusLine".to_owned(), status_line);
    if let Err(failure) = settings::write_settings(&settings_path, &user_settings) {
        settings::remove_record(&data)?;
        return Err(failure);
    }
    Ok(())
}
