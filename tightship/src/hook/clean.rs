use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::failure::Failure;
use crate::status::settings;

const COMMAND_LIMIT: Duration = Duration::from_secs(30);

struct Leftovers {
    names: Vec<String>,
    previous_versions: bool,
    commands: Vec<(String, String)>,
    paths: Vec<String>,
    json: Vec<(String, String)>,
    status_lines: Vec<String>,
}

#[derive(Default)]
struct Outcome {
    removed: bool,
    failures: Vec<String>,
}

impl Outcome {
    fn record(&mut self, entry: &str, result: Result<bool, String>) {
        match result {
            Ok(removed) => self.removed |= removed,
            Err(reason) => self.failures.push(format!(
                "tightship could not remove {entry}: {reason}. It tries again at the next session."
            )),
        }
    }
}

pub fn run() -> Result<(), Failure> {
    let Some(leftovers) = env::var_os("TIGHTSHIP_CLEAN")
        .filter(|value| !value.is_empty())
        .and_then(|file| fs::read_to_string(file).ok())
        .and_then(|text| parse(&text))
    else {
        return Ok(());
    };
    let mut outcome = Outcome::default();
    for (when, line) in &leftovers.commands {
        if expand(when).is_some_and(|path| path.exists()) {
            outcome.record(when, run_command(line));
        }
    }
    for entry in &leftovers.paths {
        if let Some(path) = expand(entry) {
            outcome.record(entry, remove_path(&path));
        }
    }
    for (entry, pointer) in &leftovers.json {
        if let Some(file) = expand(entry) {
            outcome.record(entry, edit_json(&file, pointer));
        }
    }
    for contains in &leftovers.status_lines {
        clear_status_line(contains, &mut outcome);
    }
    let mut lines = Vec::new();
    if outcome.removed {
        lines.push(format!(
            "tightship removed the plugins it replaces, and everything they left: {}.",
            leftovers.names.join(", ")
        ));
    }
    if leftovers.previous_versions {
        remove_previous_versions(&mut outcome);
    }
    lines.append(&mut outcome.failures);
    super::emit(&lines.join("\n"))
}

fn parse(text: &str) -> Option<Leftovers> {
    let data: Value = serde_json::from_str(text).ok()?;
    let strings = |key: &str| -> Option<Vec<String>> {
        data.get(key)
            .map_or(Some(&Vec::new()), Value::as_array)?
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect()
    };
    let pairs = |key: &str, first: &str, second: &str| -> Option<Vec<(String, String)>> {
        data.get(key)
            .map_or(Some(&Vec::new()), Value::as_array)?
            .iter()
            .map(|item| {
                Some((
                    item.get(first)?.as_str()?.to_owned(),
                    item.get(second)?.as_str()?.to_owned(),
                ))
            })
            .collect()
    };
    Some(Leftovers {
        names: strings("names")?,
        previous_versions: data
            .get("previous-versions")
            .map_or(Some(false), Value::as_bool)?,
        commands: pairs("commands", "when", "run")?,
        paths: strings("paths")?,
        json: pairs("json", "file", "remove")?,
        status_lines: strings("status-line")?,
    })
}

fn expand(entry: &str) -> Option<PathBuf> {
    let (base, rest) = if let Some(rest) = entry.strip_prefix("~/") {
        (crate::options::home_dir()?, rest)
    } else if let Some(rest) = entry.strip_prefix("{config}/") {
        (crate::variable("TIGHTSHIP_CONFIG").ok()?, rest)
    } else {
        let rest = entry.strip_prefix("{appdata}/")?;
        (crate::variable("APPDATA").ok()?, rest)
    };
    Some(
        rest.split('/')
            .filter(|part| !part.is_empty())
            .fold(base, |path, part| path.join(part)),
    )
}

fn run_command(line: &str) -> Result<bool, String> {
    let mut command = if cfg!(windows) {
        let mut command = Command::new("powershell");
        command.args(["-NoProfile", "-Command", line]);
        command
    } else {
        let mut command = Command::new("sh");
        command.args(["-c", line]);
        command
    };
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("`{line}` did not start ({error})"))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(true),
            Ok(Some(status)) => return Err(format!("`{line}` exited with {status}")),
            Ok(None) if started.elapsed() < COMMAND_LIMIT => {
                thread::sleep(Duration::from_millis(50));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "`{line}` did not finish in {} seconds",
                    COMMAND_LIMIT.as_secs()
                ));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

fn remove_path(path: &Path) -> Result<bool, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    let removed = if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    match removed {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}

fn edit_json(file: &Path, pointer: &str) -> Result<bool, String> {
    let mut root = Value::Object(settings::read_settings(file).map_err(reason)?);
    if !remove_member(&mut root, pointer) {
        return Ok(false);
    }
    let Value::Object(fields) = root else {
        return Ok(false);
    };
    settings::write_settings(file, &fields).map_err(reason)?;
    Ok(true)
}

fn remove_member(root: &mut Value, pointer: &str) -> bool {
    if let Some((array, filter)) = pointer.split_once("[name=") {
        let Some(name) = filter.strip_suffix(']') else {
            return false;
        };
        let Some(Value::Array(items)) = root.pointer_mut(array) else {
            return false;
        };
        let before = items.len();
        items.retain(|item| item.get("name").and_then(Value::as_str) != Some(name));
        return items.len() != before;
    }
    let Some((parent, key)) = pointer.rsplit_once('/') else {
        return false;
    };
    let key = key.replace("~1", "/").replace("~0", "~");
    match root.pointer_mut(parent) {
        Some(Value::Object(fields)) => fields.shift_remove(&key).is_some(),
        _ => false,
    }
}

fn clear_status_line(contains: &str, outcome: &mut Outcome) {
    let names = |status_line: &Value| {
        status_line
            .get("command")
            .and_then(Value::as_str)
            .is_some_and(|command| command.contains(contains))
    };
    if let Ok(path) = crate::variable("TIGHTSHIP_SETTINGS") {
        let removed = settings::read_settings(&path).and_then(|mut fields| {
            if !fields.get("statusLine").is_some_and(names) {
                return Ok(false);
            }
            fields.shift_remove("statusLine");
            settings::write_settings(&path, &fields).map(|()| true)
        });
        outcome.record(&path.display().to_string(), removed.map_err(reason));
    }
    if let Ok(data) = crate::variable("TIGHTSHIP_DATA") {
        let removed = settings::read_previous(&data).and_then(|previous| {
            if !previous.as_ref().is_some_and(names) {
                return Ok(false);
            }
            settings::write_record(&data, None).map(|()| true)
        });
        let record = data.join("status-line.json");
        outcome.record(&record.display().to_string(), removed.map_err(reason));
    }
}

fn remove_previous_versions(outcome: &mut Outcome) {
    let Some(running) = crate::variable("TIGHTSHIP_ROOT")
        .ok()
        .and_then(|root| fs::canonicalize(root).ok())
    else {
        return;
    };
    let Some(Ok(entries)) = running.parent().map(fs::read_dir) else {
        return;
    };
    for folder in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        if !folder.is_dir()
            || fs::canonicalize(&folder).is_ok_and(|folder| folder == running)
            || !folder.join(".orphaned_at").is_file()
        {
            continue;
        }
        let in_use = fs::read_dir(folder.join(".in_use"))
            .is_ok_and(|mut sessions| sessions.next().is_some());
        if in_use {
            continue;
        }
        if let Err(error) = fs::remove_dir_all(&folder) {
            outcome.record(&folder.display().to_string(), Err(error.to_string()));
        }
    }
}

fn reason(failure: Failure) -> String {
    match failure {
        Failure::SettingsUnreadable(_, error) | Failure::SettingsUnwritable(_, error) => {
            error.to_string()
        }
        Failure::SettingsInvalid(_) => "it is not a JSON object".to_owned(),
        other => other.to_string(),
    }
}
