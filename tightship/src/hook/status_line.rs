use std::env;
use std::path::Path;

use serde_json::Value;

use crate::failure::Failure;
use crate::status::settings::{self, Record};

pub fn run() -> Result<(), Failure> {
    let settings_path = settings::variable("TIGHTSHIP_SETTINGS")?;
    let data = settings::variable("TIGHTSHIP_DATA")?;
    let root = settings::variable("TIGHTSHIP_ROOT")?;
    let record = settings::read_record(&data)?;
    let mut user_settings = settings::read_settings(&settings_path)?;
    let ours = user_settings
        .get("statusLine")
        .is_some_and(settings::is_ours);
    if env::var_os("TIGHTSHIP_STATUS_LINE").is_some_and(|value| value == "false") {
        if ours {
            match record {
                Some(Record::On(Some(previous))) => {
                    user_settings.insert("statusLine".to_owned(), previous);
                }
                _ => {
                    user_settings.shift_remove("statusLine");
                }
            }
            settings::write_settings(&settings_path, &user_settings)?;
        }
        return settings::remove_record(&data);
    }
    if ours {
        let command = Value::String(settings::command_for(&root));
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
    match record {
        None => println!("{}", offer(&settings_path, &data, &root)),
        Some(Record::Declined) => {}
        Some(Record::On(_)) => settings::write_record(&data, &Record::Declined)?,
    }
    Ok(())
}

fn offer(settings_path: &Path, data: &Path, root: &Path) -> String {
    let settings_path = super::quoted(settings_path);
    let data = super::quoted(data);
    let root = super::quoted(root);
    format!(
        "tightship can show its status line in this session. In your first reply, ask the adopter, word for word:
\"tightship can show its status line under the prompt. It replaces the status line in your user settings, and turning the plugin's status_line option off brings yours back. Turn it on?\"
On yes, run: TIGHTSHIP_SETTINGS={settings_path} TIGHTSHIP_DATA={data} TIGHTSHIP_ROOT={root} tightship status-line enable
On no, run: TIGHTSHIP_DATA={data} tightship status-line decline"
    )
}
