//! The rate limits a session last reported, cached so a session that reports none still shows them.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use jiff::Timestamp;
use serde_json::{Map, Value};

use crate::system::env::optional_variable;

const CACHE: &str = "rate-limits.json";
const WINDOWS: [&str; 2] = ["five_hour", "seven_day"];

/// Fills the session's missing rate limits from the cache while they are current, and caches the ones it has.
pub fn remember_from_cache(session: &mut Value) {
    let Some(folder) = optional_variable("TIGHTSHIP_DATA").filter(|folder| folder.is_dir()) else {
        return;
    };
    let file = folder.join(CACHE);
    let cached: Map<String, Value> = fs::read_to_string(&file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    let remembered = remember_windows(session, cached.clone(), Timestamp::now().as_second());
    if remembered != cached {
        // The cache only fills a session that reports no rate limits; a failed write must not break the line.
        let _ = write_cache(&file, &remembered);
    }
}

fn remember_windows(
    session: &mut Value,
    mut cached: Map<String, Value>,
    now: i64,
) -> Map<String, Value> {
    let mut remembered = Map::new();
    for window in WINDOWS {
        let pointer = format!("/rate_limits/{window}");
        if let Some(reading) = session
            .pointer(&pointer)
            .filter(|reading| reading.is_object())
        {
            remembered.insert(window.to_owned(), reading.clone());
            continue;
        }
        let Some(reading) = cached.remove(window) else {
            continue;
        };
        let current = reading
            .get("resets_at")
            .and_then(Value::as_i64)
            .is_some_and(|reset| reset > now);
        if !current || !reading.get("used_percentage").is_some_and(Value::is_number) {
            continue;
        }
        let limits = session
            .as_object_mut()
            .map(|fields| {
                fields
                    .entry("rate_limits")
                    .or_insert_with(|| Value::Object(Map::new()))
            })
            .and_then(Value::as_object_mut);
        if let Some(limits) = limits {
            limits.insert(window.to_owned(), reading.clone());
            remembered.insert(window.to_owned(), reading);
        }
    }
    remembered
}

fn write_cache(file: &Path, remembered: &Map<String, Value>) -> io::Result<()> {
    let text = serde_json::to_string(remembered).map_err(io::Error::other)?;
    let mut temporary = file.as_os_str().to_owned();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let written = fs::write(&temporary, text).and_then(|()| fs::rename(&temporary, file));
    if written.is_err() {
        // The write already failed; that error is the one returned.
        let _ = fs::remove_file(&temporary);
    }
    written
}
