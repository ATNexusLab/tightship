use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::failure::Failure;

const RECORD: &str = "status-line.json";

pub enum Record {
    On(Option<Value>),
    Declined,
}

pub fn enable() -> Result<(), Failure> {
    let settings_path = crate::variable("TIGHTSHIP_SETTINGS")?;
    let data = crate::variable("TIGHTSHIP_DATA")?;
    let root = crate::variable("TIGHTSHIP_ROOT")?;
    let mut settings = read_settings(&settings_path)?;
    if settings.get("statusLine").is_some_and(is_ours) {
        println!("tightship's status line is already on.");
        return Ok(());
    }
    write_record(&data, &Record::On(settings.get("statusLine").cloned()))?;
    settings.insert(
        "statusLine".to_owned(),
        serde_json::json!({ "type": "command", "command": command_for(&root) }),
    );
    if let Err(failure) = write_settings(&settings_path, &settings) {
        remove_record(&data)?;
        return Err(failure);
    }
    println!(
        "tightship's status line is on. Turn the plugin's status_line option off to bring back the one you had."
    );
    Ok(())
}

pub fn decline() -> Result<(), Failure> {
    let data = crate::variable("TIGHTSHIP_DATA")?;
    if let Some(Record::On(_)) = read_record(&data)? {
        println!(
            "tightship's status line is on; turn the plugin's status_line option off to bring back the one you had."
        );
        return Ok(());
    }
    write_record(&data, &Record::Declined)?;
    println!(
        "The status line stays as it is. tightship will not ask again until its status_line option is turned off and on."
    );
    Ok(())
}

pub fn is_ours(status_line: &Value) -> bool {
    status_line
        .get("command")
        .and_then(Value::as_str)
        .is_some_and(|command| {
            command.ends_with("/bin/tightship\" status-line render")
                || command.ends_with("\\bin\\tightship.cmd\" status-line render")
                || command.ends_with("/bin/tightship.cmd\" status-line render")
        })
}

pub fn command_for(root: &Path) -> String {
    if let Some(command) = env::var("TIGHTSHIP_STATUS_LINE_COMMAND")
        .ok()
        .filter(|command| !command.is_empty())
    {
        return command;
    }
    #[cfg(windows)]
    {
        let cmd = root.join("bin").join("tightship.cmd");
        if cmd.exists() {
            return format!("\"{}\" status-line render", cmd.display());
        }
    }
    format!("sh \"{}/bin/tightship\" status-line render", root.display())
}

pub fn read_settings(path: &Path) -> Result<Map<String, Value>, Failure> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Map::new()),
        Err(error) => return Err(Failure::SettingsUnreadable(path.to_path_buf(), error)),
    };
    match serde_json::from_str(&text) {
        Ok(Value::Object(settings)) => Ok(settings),
        _ => Err(Failure::SettingsInvalid(path.to_path_buf())),
    }
}

pub fn write_settings(path: &Path, settings: &Map<String, Value>) -> Result<(), Failure> {
    let unwritable = |error| Failure::SettingsUnwritable(path.to_path_buf(), error);
    let target = match fs::canonicalize(path) {
        Ok(target) => target,
        Err(error) if error.kind() == io::ErrorKind::NotFound => path.to_path_buf(),
        Err(error) => return Err(unwritable(error)),
    };
    let mut temporary = target.clone().into_os_string();
    temporary.push(".tightship.tmp");
    let temporary = PathBuf::from(temporary);
    let text = serde_json::to_string_pretty(settings)
        .map_err(|error| unwritable(io::Error::other(error)))?
        + "\n";
    let permissions = match fs::metadata(&target) {
        Ok(metadata) => Some(metadata.permissions()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(unwritable(error)),
    };
    let folder = target.parent().unwrap_or(Path::new("."));
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let written = fs::create_dir_all(folder)
        .and_then(|()| options.open(&temporary))
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .and_then(|()| match permissions {
            Some(permissions) => fs::set_permissions(&temporary, permissions),
            None => Ok(()),
        })
        .and_then(|()| fs::rename(&temporary, &target));
    if let Err(error) = written {
        if temporary.exists() {
            fs::remove_file(&temporary).map_err(unwritable)?;
        }
        return Err(unwritable(error));
    }
    Ok(())
}

pub fn read_record(data: &Path) -> Result<Option<Record>, Failure> {
    let path = data.join(RECORD);
    let invalid = || Failure::StatusRecordInvalid(path.clone());
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(invalid()),
    };
    let Ok(Value::Object(mut fields)) = serde_json::from_str(&text) else {
        return Err(invalid());
    };
    let state = fields.shift_remove("state");
    let previous = fields.shift_remove("previous");
    if !fields.is_empty() {
        return Err(invalid());
    }
    match (state.as_ref().and_then(Value::as_str), previous) {
        (Some("on"), previous) => Ok(Some(Record::On(previous))),
        (Some("declined"), None) => Ok(Some(Record::Declined)),
        _ => Err(invalid()),
    }
}

pub fn write_record(data: &Path, record: &Record) -> Result<(), Failure> {
    let path = data.join(RECORD);
    let mut fields = Map::new();
    match record {
        Record::On(previous) => {
            fields.insert("state".to_owned(), "on".into());
            if let Some(previous) = previous {
                fields.insert("previous".to_owned(), previous.clone());
            }
        }
        Record::Declined => {
            fields.insert("state".to_owned(), "declined".into());
        }
    }
    let unwritable = |error| Failure::StatusRecordUnwritable(path.clone(), error);
    let text = serde_json::to_string_pretty(&fields)
        .map_err(|error| unwritable(io::Error::other(error)))?
        + "\n";
    fs::create_dir_all(data)
        .and_then(|()| fs::write(&path, text))
        .map_err(unwritable)
}

pub fn remove_record(data: &Path) -> Result<(), Failure> {
    let path = data.join(RECORD);
    match fs::remove_file(&path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            Err(Failure::StatusRecordUnwritable(path, error))
        }
        _ => Ok(()),
    }
}
