//! File helpers shared by every area: reading and removing what may be absent, and JSON text.

use std::fs;
use std::io;
use std::path::Path;

use serde_json::{Map, Value};

/// The file's text, or `None` when it does not exist.
///
/// # Errors
///
/// When the file exists and cannot be read as UTF-8 text.
pub fn read_if_present(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Removes a file, doing nothing when it does not exist.
///
/// # Errors
///
/// When the file exists and cannot be removed.
pub fn remove_file_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

/// Removes a folder and everything in it, doing nothing when it does not exist.
///
/// # Errors
///
/// When the folder exists and cannot be removed.
pub fn remove_folder_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

/// The object as indented JSON ending in a newline.
///
/// # Errors
///
/// When the object cannot be serialized.
pub fn pretty_json(value: &Map<String, Value>) -> io::Result<String> {
    Ok(serde_json::to_string_pretty(value).map_err(io::Error::other)? + "\n")
}

/// The text without the byte order mark some Windows editors put at its start.
pub fn strip_byte_order_mark(text: &str) -> &str {
    text.trim_start_matches('\u{feff}')
}
