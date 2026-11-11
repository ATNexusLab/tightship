//! The harness's settings file and the record of the status line tightship replaced in it.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::failure::Failure;
use crate::files::{pretty_json, read_if_present, remove_file_if_present};

const RECORD: &str = "status-line.json";

/// Whether the status line runs a `tightship` binary from a plugin's `bin/` folder.
pub fn is_ours(status_line: &Value) -> bool {
    let Some(command) = status_line.get("command").and_then(Value::as_str) else {
        return false;
    };
    let Some(path) = command.strip_suffix(" status-line render") else {
        return false;
    };
    let path = path.strip_prefix("sh ").unwrap_or(path);
    let path = path.strip_prefix('"').unwrap_or(path);
    let path = path.strip_suffix('"').unwrap_or(path);
    let path = path.replace('\\', "/");
    let mut components = path.rsplit('/');
    let file = components.next();
    let folder = components.next();
    let above = components.next();
    matches!(file, Some("tightship" | "tightship.cmd" | "tightship.exe"))
        && (folder == Some("bin") || above == Some("bin"))
}

/// The command that renders tightship's status line from the plugin at the root.
///
/// # Errors
///
/// On Windows, when the launcher's path has characters `cmd` reads and no short form without them.
pub fn command_for(root: &Path) -> Result<String, Failure> {
    if let Some(command) = env::var("TIGHTSHIP_STATUS_LINE_COMMAND")
        .ok()
        .filter(|command| !command.is_empty())
    {
        return Ok(command);
    }
    #[cfg(windows)]
    {
        let cmd = root.join("bin").join("tightship.cmd");
        if cmd.exists() {
            let launcher = readable_unquoted(&cmd)?.replace('\\', "/");
            return Ok(format!("{launcher} status-line render"));
        }
    }
    Ok(format!(
        "sh \"{}/bin/tightship\" status-line render",
        root.display()
    ))
}

#[cfg(windows)]
fn readable_unquoted(path: &Path) -> Result<String, Failure> {
    use std::os::windows::process::CommandExt;

    let read_by_cmd =
        |text: &str| text.contains([' ', '&', '^', '(', ')', '%', '!', ';', '=', ',']);
    let path = path.to_string_lossy().replace('/', "\\");
    if !read_by_cmd(&path) {
        return Ok(path);
    }
    let unreadable = || Failure::StatusLinePathUnreadable(PathBuf::from(&path));
    let output = std::process::Command::new("cmd")
        .raw_arg(format!(r#"/d /u /c for %I in ("{path}") do @echo %~sI"#))
        .output()
        .map_err(|_| unreadable())?;
    let units: Vec<u16> = output
        .stdout
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .collect();
    let short = String::from_utf16(&units).map_err(|_| unreadable())?;
    let short = short.trim();
    if !output.status.success() || read_by_cmd(short) || !Path::new(short).is_file() {
        return Err(unreadable());
    }
    Ok(short.to_owned())
}

/// The settings as a JSON object, empty when the file does not exist.
///
/// # Errors
///
/// When the file cannot be read or is not a JSON object.
pub fn read_settings(path: &Path) -> Result<Map<String, Value>, Failure> {
    let Some(text) = read_if_present(path)
        .map_err(|error| Failure::SettingsUnreadable(path.to_path_buf(), error))?
    else {
        return Ok(Map::new());
    };
    match serde_json::from_str(&text) {
        Ok(Value::Object(settings)) => Ok(settings),
        _ => Err(Failure::SettingsInvalid(path.to_path_buf())),
    }
}

/// Writes the settings through a temporary file, keeping the file's permissions and a symlink's target.
///
/// # Errors
///
/// When the file or its folder cannot be written.
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
    let text = pretty_json(settings).map_err(unwritable)?;
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

/// The file that records the status line tightship replaced.
pub fn record_path(data: &Path) -> PathBuf {
    data.join(RECORD)
}

/// The status line tightship replaced, or `None` when there was none or no record exists.
///
/// # Errors
///
/// When the record cannot be read or is not in the shape `write_record` leaves.
pub fn read_previous(data: &Path) -> Result<Option<Value>, Failure> {
    let path = record_path(data);
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

/// Records that tightship's status line is on, and the one it replaced.
///
/// # Errors
///
/// When the record cannot be written.
pub fn write_record(data: &Path, previous: Option<&Value>) -> Result<(), Failure> {
    let path = record_path(data);
    let mut fields = Map::new();
    fields.insert("state".to_owned(), "on".into());
    if let Some(previous) = previous {
        fields.insert("previous".to_owned(), previous.clone());
    }
    let unwritable = |error| Failure::StatusRecordUnwritable(path.clone(), error);
    let text = pretty_json(&fields).map_err(unwritable)?;
    fs::create_dir_all(data)
        .and_then(|()| fs::write(&path, text))
        .map_err(unwritable)
}

/// Removes the record, doing nothing when there is none.
///
/// # Errors
///
/// When the record exists and cannot be removed.
pub fn remove_record(data: &Path) -> Result<(), Failure> {
    let path = record_path(data);
    remove_file_if_present(&path).map_err(|error| Failure::StatusRecordUnwritable(path, error))
}
