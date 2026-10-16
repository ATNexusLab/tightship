use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::failure::Failure;

pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    if !options.memory || options.vault_folder.is_none() {
        return Ok(());
    }
    let mut input = String::new();
    let session = match io::stdin().read_to_string(&mut input) {
        Ok(_) => session_file_name(&input),
        Err(_) => "unknown".to_owned(),
    };
    let checkpoints = env::var_os("TIGHTSHIP_DATA")
        .filter(|value| !value.is_empty())
        .map(|data| PathBuf::from(data).join("checkpoints"));
    let Some(checkpoints) = checkpoints else {
        return Err(Failure::DataUnwritable(
            PathBuf::from("checkpoints").join(session),
            io::Error::other("TIGHTSHIP_DATA is not set"),
        ));
    };
    let counter = checkpoints.join(&session);
    let unwritable = |error| Failure::DataUnwritable(counter.clone(), error);
    fs::create_dir_all(&checkpoints).map_err(unwritable)?;
    sweep_stale(&checkpoints)?;
    let previous: u64 = match fs::read_to_string(&counter) {
        Ok(text) => text.trim().parse().unwrap_or(0),
        Err(error) if error.kind() == io::ErrorKind::NotFound => 0,
        Err(error) => return Err(unwritable(error)),
    };
    let count = previous + 1;
    fs::write(&counter, count.to_string()).map_err(unwritable)?;
    if count.is_multiple_of(10) {
        super::emit(super::CHECKPOINT);
    }
    Ok(())
}

fn session_file_name(input: &str) -> String {
    let parsed: Option<serde_json::Value> = serde_json::from_str(input).ok();
    let session_id = parsed
        .as_ref()
        .and_then(|value| {
            value
                .get("session_id")
                .or_else(|| value.get("conversationId"))
        })
        .and_then(serde_json::Value::as_str)
        .filter(|session_id| !session_id.is_empty())
        .unwrap_or("unknown");
    session_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn sweep_stale(checkpoints: &Path) -> Result<(), Failure> {
    let week_ago = SystemTime::now() - Duration::from_secs(7 * 24 * 60 * 60);
    let entries = fs::read_dir(checkpoints)
        .map_err(|error| Failure::DataUnwritable(checkpoints.to_path_buf(), error))?;
    for entry in entries {
        let path = entry
            .map_err(|error| Failure::DataUnwritable(checkpoints.to_path_buf(), error))?
            .path();
        let stale = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .map_err(|error| Failure::DataUnwritable(path.clone(), error))?
            < week_ago;
        if stale {
            fs::remove_file(&path).map_err(|error| Failure::DataUnwritable(path, error))?;
        }
    }
    Ok(())
}
