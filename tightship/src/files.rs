use std::fs;
use std::io;
use std::path::Path;

use serde_json::{Map, Value};

pub fn read_if_present(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn remove_file_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

pub fn remove_folder_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

pub fn pretty_json(value: &Map<String, Value>) -> io::Result<String> {
    Ok(serde_json::to_string_pretty(value).map_err(io::Error::other)? + "\n")
}

pub fn strip_byte_order_mark(text: &str) -> &str {
    text.trim_start_matches('\u{feff}')
}
