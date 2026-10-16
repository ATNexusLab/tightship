use serde_json::Value;

use crate::failure::Failure;
use crate::status::settings;

pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    let settings_path = crate::variable("TIGHTSHIP_SETTINGS")?;
    let data = crate::variable("TIGHTSHIP_DATA")?;
    let root = crate::variable("TIGHTSHIP_ROOT")?;
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
    let command = Value::String(settings::command_for(&root));
    if ours {
        let status_line = user_settings
            .get_mut("statusLine")
            .and_then(Value::as_object_mut);
        if let Some(status_line) = status_line
            && status_line.get("command") != Some(&command)
        {
            status_line.insert("command".to_owned(), command);
            settings::write_settings(&settings_path, &user_settings)?;
        }
        return Ok(());
    }
    settings::write_record(&data, user_settings.get("statusLine"))?;
    user_settings.insert(
        "statusLine".to_owned(),
        serde_json::json!({ "type": "command", "command": command }),
    );
    if let Err(failure) = settings::write_settings(&settings_path, &user_settings) {
        settings::remove_record(&data)?;
        return Err(failure);
    }
    Ok(())
}
