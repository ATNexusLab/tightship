//! Tests of the status line: rendering it and the hook that puts it in the harness's settings.

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

use std::fs;
use std::path::Path;
use std::process::Output;

use serde_json::json;
use unicode_width::UnicodeWidthStr;

use common::{Sandbox, assert_fails_with, assert_succeeds, run, stderr_of, stdout_of, with_stdin};

mod cursor;
mod hook;
mod render;

const ROOT: &str = "/plugins/0.1.0";

const OURS: &str = "sh \"/plugins/0.1.0/bin/tightship\" status-line render";

const MINE: &str = r#"{"statusLine": {"type": "command", "command": "mine"}}"#;

fn checkout(sandbox: &Sandbox, branch: &str) {
    sandbox.write(
        "tightship/.git/HEAD",
        &format!("ref: refs/heads/{branch}\n"),
    );
}

fn render(sandbox: &Sandbox, fixture: &str, columns: Option<&str>) -> Output {
    render_with_prefix(sandbox, fixture, columns, Some("Claude "))
}

fn render_with_prefix(
    sandbox: &Sandbox,
    fixture: &str,
    columns: Option<&str>,
    prefix: Option<&str>,
) -> Output {
    let input = read_fixture(sandbox, fixture);
    let mut command = sandbox.tightship(&["status-line", "render"]);
    command.env("TZ", "UTC");
    match columns {
        Some(columns) => command.env("COLUMNS", columns),
        None => command.env_remove("COLUMNS"),
    };
    match prefix {
        Some(prefix) => command.env("TIGHTSHIP_MODEL_PREFIX", prefix),
        None => command.env_remove("TIGHTSHIP_MODEL_PREFIX"),
    };
    with_stdin(&mut command, &input)
}

fn read_fixture(sandbox: &Sandbox, fixture: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/status")
        .join(format!("{fixture}.json"));
    fs::read_to_string(file).expect("the fixture").replace(
        "/home/adopter/tightship",
        &sandbox
            .path("tightship")
            .display()
            .to_string()
            .replace('\\', "\\\\"),
    )
}

fn status_line(sandbox: &Sandbox, arguments: &[&str], root: &str, option: Option<&str>) -> Output {
    let mut command = sandbox.tightship(arguments);
    command
        .env("TIGHTSHIP_SETTINGS", sandbox.path("config/settings.json"))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_ROOT", root)
        .env("TIGHTSHIP_STATUS_LINE_REFRESH", "60");
    if let Some(option) = option {
        command.env("TIGHTSHIP_STATUS_LINE", option);
    }
    run(&mut command)
}

fn hook(sandbox: &Sandbox, option: Option<&str>) -> Output {
    status_line(sandbox, &["hook", "status-line"], ROOT, option)
}

fn settings(sandbox: &Sandbox) -> String {
    fs::read_to_string(sandbox.path("config/settings.json")).expect("the settings file")
}

fn pretty(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).expect("JSON") + "\n"
}

fn record(sandbox: &Sandbox) -> Option<serde_json::Value> {
    fs::read_to_string(sandbox.path("data/status-line.json"))
        .ok()
        .map(|text| serde_json::from_str(&text).expect("a JSON record"))
}
