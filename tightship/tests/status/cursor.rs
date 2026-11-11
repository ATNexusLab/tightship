//! The status line in Cursor's CLI settings.

use super::*;

const CURSOR_OURS: &str = "\"${CURSOR_PLUGIN_ROOT}/bin/tightship\" status-line render";

fn cursor_status_line(sandbox: &Sandbox, arguments: &[&str], root: &str) -> Output {
    let mut command = sandbox.tightship(arguments);
    command
        .env("TIGHTSHIP_SETTINGS", sandbox.path("config/cli-config.json"))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_ROOT", root)
        .env("TIGHTSHIP_STATUS_LINE_COMMAND", CURSOR_OURS);
    run(&mut command)
}

fn cursor_settings(sandbox: &Sandbox) -> String {
    fs::read_to_string(sandbox.path("config/cli-config.json")).expect("the CLI config")
}

#[test]
fn cursor_session_start_writes_status_line_when_the_option_is_on() {
    let sandbox = Sandbox::new();
    sandbox.write("config/cli-config.json", MINE);
    fs::create_dir_all(sandbox.path("data")).expect("the data folder");

    let output = cursor_status_line(&sandbox, &["hook", "status-line"], ROOT);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert_eq!(
        cursor_settings(&sandbox),
        pretty(&json!({ "statusLine": { "type": "command", "command": CURSOR_OURS } }))
    );
    assert_eq!(
        record(&sandbox),
        Some(json!({ "state": "on", "previous": { "type": "command", "command": "mine" } }))
    );
}

#[test]
fn cursor_turning_the_option_off_restores_the_previous_cli_status_line() {
    let sandbox = Sandbox::new();
    let mine = json!({ "type": "command", "command": "mine" });
    sandbox.write(
        "config/cli-config.json",
        &pretty(&json!({ "statusLine": mine })),
    );
    sandbox.write(
        "data/options.json",
        &pretty(&json!({ "status_line": false })),
    );

    let output = cursor_status_line(&sandbox, &["hook", "status-line"], ROOT);

    assert_succeeds(&output);
    assert_eq!(
        cursor_settings(&sandbox),
        pretty(&json!({ "statusLine": mine }))
    );
    assert_eq!(record(&sandbox), None);
}

#[test]
fn cursor_status_line_keeps_the_plugin_root_token_when_the_plugin_folder_changes() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "config/cli-config.json",
        &pretty(&json!({ "statusLine": { "type": "command", "command": CURSOR_OURS } })),
    );
    sandbox.write("data/status-line.json", r#"{"state": "on"}"#);

    let output = cursor_status_line(&sandbox, &["hook", "status-line"], "/plugins/0.2.0");

    assert_succeeds(&output);
    assert_eq!(
        cursor_settings(&sandbox),
        pretty(&json!({ "statusLine": { "type": "command", "command": CURSOR_OURS } }))
    );
}
