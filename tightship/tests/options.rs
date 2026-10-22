#[allow(dead_code)]
mod common;

use std::fs;

use common::{Sandbox, assert_fails_with, assert_succeeds, run, stderr_of, stdout_of};

fn options_file(sandbox: &Sandbox, data: &str) -> std::path::PathBuf {
    sandbox.path(data).join("options.json")
}

fn with_data(sandbox: &Sandbox, data: &str) -> std::process::Command {
    let mut command = sandbox.tightship(&[]);
    command.arg("options");
    command.env("TIGHTSHIP_DATA", sandbox.path(data));
    command
}

#[test]
fn scenario_9_set_memory_creates_options_json() {
    let sandbox = Sandbox::new();
    let path = options_file(&sandbox, "data");
    assert!(!path.is_file());

    let mut command = with_data(&sandbox, "data");
    command.args(["set", "memory", "false"]);
    let output = run(&mut command);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "Set memory to false.\n");

    let text = fs::read_to_string(&path).expect("options.json");
    let value: serde_json::Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(value["memory"], serde_json::json!(false));
}

#[test]
fn scenario_10_unknown_option_fails() {
    let sandbox = Sandbox::new();
    let mut command = with_data(&sandbox, "data");
    command.args(["set", "not_an_option", "true"]);
    let output = run(&mut command);
    assert_fails_with(&output, "error: unknown option \"not_an_option\"");
}

#[test]
fn list_prints_empty_object_when_missing() {
    let sandbox = Sandbox::new();
    let mut command = with_data(&sandbox, "data");
    command.arg("list");
    let output = run(&mut command);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "{}\n");
}

#[test]
fn set_rejects_non_boolean_for_boolean_options() {
    let sandbox = Sandbox::new();
    let mut command = with_data(&sandbox, "data");
    command.args(["set", "memory", "maybe"]);
    let output = run(&mut command);
    assert_fails_with(&output, "error: memory must be true or false");
}

#[test]
fn reset_removes_options_json() {
    let sandbox = Sandbox::new();
    let path = options_file(&sandbox, "data");
    fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    fs::write(&path, "{\"memory\":false}\n").expect("options");
    let mut command = with_data(&sandbox, "data");
    command.arg("reset");
    let output = run(&mut command);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), format!("Removed {}.\n", path.display()));
    assert!(!path.is_file());
}

#[test]
fn disabled_exits_zero_when_option_is_off() {
    let sandbox = Sandbox::new();
    let path = options_file(&sandbox, "data");
    fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    fs::write(&path, "\u{feff}{\"contract\":false}\n").expect("options");
    let mut command = with_data(&sandbox, "data");
    command.args(["disabled", "contract"]);
    let output = run(&mut command);
    assert_succeeds(&output);
    assert!(stderr_of(&output).is_empty());
}

#[test]
fn disabled_exits_one_when_option_is_on() {
    let sandbox = Sandbox::new();
    let mut command = with_data(&sandbox, "data");
    command.args(["disabled", "contract"]);
    let output = run(&mut command);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr_of(&output).is_empty());
}

#[test]
fn disabled_rejects_vault_folder_as_a_usage_error() {
    let sandbox = Sandbox::new();
    let path = options_file(&sandbox, "data");
    fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    fs::write(&path, "{\"vault_folder\":\"/notes\"}\n").expect("options");
    let mut command = with_data(&sandbox, "data");
    command.args(["disabled", "vault_folder"]);
    let output = run(&mut command);
    assert_fails_with(
        &output,
        "error: vault_folder is a folder, not an on/off option",
    );
    assert!(stderr_of(&output).contains("Usage:"));
}

#[test]
fn vault_folder_expands_tilde_on_set() {
    let sandbox = Sandbox::new();
    let home = sandbox.path("home");
    let path = options_file(&sandbox, "data");
    fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    fs::write(&path, "\u{feff}{\"memory\":false}\n").expect("options");
    let mut command = with_data(&sandbox, "data");
    command
        .args(["set", "vault_folder", "~/vault"])
        .env("HOME", &home)
        .env("USERPROFILE", &home);
    let output = run(&mut command);
    assert_succeeds(&output);
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("JSON");
    assert_eq!(
        value["vault_folder"],
        serde_json::json!(home.join("vault").to_string_lossy())
    );
    assert_eq!(value["memory"], serde_json::json!(false));
}
