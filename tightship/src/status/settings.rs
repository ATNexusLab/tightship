use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::failure::Failure;

const RECORD: &str = "status-line.json";

pub fn is_ours(status_line: &Value) -> bool {
    let Some(command) = status_line.get("command").and_then(Value::as_str) else {
        return false;
    };
    let Some(path) = command.strip_suffix("\" status-line render") else {
        return false;
    };
    let path = path.replace('\\', "/");
    let mut components = path.rsplit('/');
    let file = components.next();
    let folder = components.next();
    let above = components.next();
    matches!(file, Some("tightship" | "tightship.cmd" | "tightship.exe"))
        && (folder == Some("bin") || above == Some("bin"))
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

pub fn read_previous(data: &Path) -> Result<Option<Value>, Failure> {
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
        (Some("on"), previous) => Ok(previous),
        (Some("declined"), None) => Ok(None),
        _ => Err(invalid()),
    }
}

pub fn write_record(data: &Path, previous: Option<&Value>) -> Result<(), Failure> {
    let path = data.join(RECORD);
    let mut fields = Map::new();
    fields.insert("state".to_owned(), "on".into());
    if let Some(previous) = previous {
        fields.insert("previous".to_owned(), previous.clone());
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
