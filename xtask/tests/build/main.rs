//! Tests of `cargo xtask build`, run over a fixture repository in a temporary folder.

#[path = "../common/mod.rs"]
mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use common::{Repository, assert_fails_with, assert_succeeds, harness_files, stderr_of};

mod clean;
mod content;
mod cursor;
mod manifest;
mod neutrality;
mod options;
mod references;

fn piece<'path>(path: &'path str, body: &str) -> (&'path str, String) {
    let name = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| *stem != "SKILL")
        .or_else(|| path.split('/').nth(2))
        .expect("a piece name");
    let access = if path.starts_with("content/agents/") {
        "access = \"read-only\"\n"
    } else {
        ""
    };
    let text = format!("+++\nname = \"{name}\"\ndescription = \"A piece\"\n{access}+++\n{body}");
    (path, text)
}

fn on_line(line: usize, text: &str) -> String {
    let first_body_line = 5;
    "\n".repeat(line - first_body_line) + text + "\n"
}

fn json_at(repository: &Repository, path: &str) -> serde_json::Value {
    let text = fs::read_to_string(repository.path(path)).expect("a generated JSON file");
    serde_json::from_str(&text).expect("valid JSON")
}

fn cursor_fixture_files(extra: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut files = harness_files("cursor");
    files.extend(extra);
    files
}

fn cursor_repository(extra: Vec<(String, String)>) -> Repository {
    let built = cursor_fixture_files(extra);
    let files: Vec<(&str, String)> = built
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    Repository::with(&files)
}

fn options(entries: &[(&str, &str)]) -> (&'static str, String) {
    typed_options(
        &entries
            .iter()
            .map(|(name, description)| (*name, "boolean", *description))
            .collect::<Vec<_>>(),
    )
}

fn typed_options(entries: &[(&str, &str, &str)]) -> (&'static str, String) {
    let text = entries
        .iter()
        .map(|(name, kind, description)| {
            format!("[[options]]\nname = \"{name}\"\ntitle = \"An option\"\ndescription = \"{description}\"\ntype = \"{kind}\"\n")
        })
        .collect::<Vec<String>>()
        .join("\n");
    ("content/options.toml", text)
}
