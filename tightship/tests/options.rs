#[allow(dead_code)]
mod common;

use std::fs;

use common::{Sandbox, assert_fails_with, assert_succeeds, run, stderr_of, stdout_of};

fn cursor_options_file(sandbox: &Sandbox, home: &str) -> std::path::PathBuf {
    sandbox.path(home).join(".cursor/tightship/options.json")
}

fn with_home(sandbox: &Sandbox, home: &str) -> std::process::Command {
    let home = sandbox.path(home);
    fs::create_dir_all(&home).expect("home");
    let mut command = sandbox.tightship(&[]);
    command.arg("options");
    command.env("HOME", &home).env("USERPROFILE", &home);
    command
}

#[test]
fn scenario_9_set_memory_creates_options_json() {
    let sandbox = Sandbox::new();
    let home = "adopter";
    let path = cursor_options_file(&sandbox, home);
    assert!(!path.is_file());

    let mut command = with_home(&sandbox, home);
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
    let mut command = with_home(&sandbox, "adopter");
    command.args(["set", "not_an_option", "true"]);
    let output = run(&mut command);
    assert_fails_with(&output, "error: unknown option \"not_an_option\"");
}

#[test]
fn list_prints_empty_object_when_missing() {
    let sandbox = Sandbox::new();
    let mut command = with_home(&sandbox, "adopter");
    command.arg("list");
    let output = run(&mut command);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "{}\n");
}

#[test]
fn set_rejects_non_boolean_for_boolean_options() {
    let sandbox = Sandbox::new();
    let mut command = with_home(&sandbox, "adopter");
    command.args(["set", "memory", "maybe"]);
    let output = run(&mut command);
    assert_fails_with(&output, "error: memory must be true or false");
}

#[test]
fn reset_removes_options_json() {
    let sandbox = Sandbox::new();
    let path = cursor_options_file(&sandbox, "adopter");
    fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    fs::write(&path, "{\"memory\":false}\n").expect("options");
    let mut command = with_home(&sandbox, "adopter");
    command.arg("reset");
    let output = run(&mut command);
    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "Removed ~/.cursor/tightship/options.json.\n"
    );
    assert!(!path.is_file());
}

#[test]
fn disabled_exits_zero_when_option_is_off() {
    let sandbox = Sandbox::new();
    let path = cursor_options_file(&sandbox, "adopter");
    fs::create_dir_all(path.parent().expect("parent")).expect("folder");
    fs::write(&path, "{\"contract\":false}\n").expect("options");
    let mut command = with_home(&sandbox, "adopter");
    command.args(["disabled", "contract"]);
    let output = run(&mut command);
    assert_succeeds(&output);
    assert!(stderr_of(&output).is_empty());
}

#[test]
fn disabled_exits_one_when_option_is_on() {
    let sandbox = Sandbox::new();
    let mut command = with_home(&sandbox, "adopter");
    command.args(["disabled", "contract"]);
    let output = run(&mut command);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr_of(&output).is_empty());
}

#[test]
fn vault_folder_expands_tilde_on_set() {
    let sandbox = Sandbox::new();
    let home = sandbox.path("adopter");
    let mut command = sandbox.tightship(&["options", "set", "vault_folder", "~/vault"]);
    command.env("HOME", &home).env("USERPROFILE", &home);
    let output = run(&mut command);
    assert_succeeds(&output);
    let path = cursor_options_file(&sandbox, "adopter");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("JSON");
    assert_eq!(
        value["vault_folder"],
        serde_json::json!(home.join("vault").to_string_lossy())
    );
}
