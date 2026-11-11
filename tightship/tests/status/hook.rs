//! The status-line hook: turning it on and off, following the plugin, and failing without a half change.

use super::*;

#[test]
fn the_session_start_activates_the_status_line() {
    let sandbox = Sandbox::new();
    sandbox.write("config/settings.json", MINE);
    fs::create_dir_all(sandbox.path("data")).expect("the data folder");

    let output = hook(&sandbox, None);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert_eq!(
        settings(&sandbox),
        pretty(
            &json!({ "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 } })
        )
    );
    assert_eq!(
        record(&sandbox),
        Some(json!({ "state": "on", "previous": { "type": "command", "command": "mine" } }))
    );
}

#[test]
fn on_saves_the_previous_status_line_and_off_puts_it_back_in_place() {
    let sandbox = Sandbox::new();
    let mine = json!({ "type": "command", "command": "mine" });
    sandbox.write(
        "config/settings.json",
        r#"{"model": "opus", "statusLine": {"type": "command", "command": "mine"}, "theme": "dark"}"#,
    );

    assert_succeeds(&hook(&sandbox, None));
    assert_eq!(
        settings(&sandbox),
        pretty(&json!({
            "model": "opus",
            "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 },
            "theme": "dark",
        }))
    );
    assert_eq!(
        record(&sandbox),
        Some(json!({ "state": "on", "previous": mine }))
    );

    let off = hook(&sandbox, Some("false"));

    assert_succeeds(&off);
    assert_eq!(stdout_of(&off), "");
    assert_eq!(
        settings(&sandbox),
        pretty(&json!({ "model": "opus", "statusLine": mine, "theme": "dark" }))
    );
    assert_eq!(record(&sandbox), None);
}

#[test]
fn on_with_no_status_line_before_removes_the_key_when_off() {
    let sandbox = Sandbox::new();
    let ours = pretty(
        &json!({ "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 } }),
    );

    assert_succeeds(&hook(&sandbox, None));
    assert_eq!(settings(&sandbox), ours);
    assert_eq!(record(&sandbox), Some(json!({ "state": "on" })));

    assert_succeeds(&hook(&sandbox, None));
    assert_eq!(settings(&sandbox), ours);
    assert_eq!(record(&sandbox), Some(json!({ "state": "on" })));

    sandbox.write(
        "config/settings.json",
        &pretty(&json!({
            "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 },
            "model": "opus",
            "theme": "dark",
        })),
    );
    assert_succeeds(&hook(&sandbox, Some("false")));
    assert_eq!(
        settings(&sandbox),
        pretty(&json!({ "model": "opus", "theme": "dark" }))
    );
}

#[test]
fn an_old_declined_record_does_not_keep_the_status_line_off() {
    let sandbox = Sandbox::new();
    sandbox.write("config/settings.json", MINE);
    sandbox.write("data/status-line.json", r#"{"state": "declined"}"#);

    let on = hook(&sandbox, None);

    assert_succeeds(&on);
    assert_eq!(stdout_of(&on), "");
    assert_eq!(
        settings(&sandbox),
        pretty(
            &json!({ "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 } })
        )
    );
    assert_eq!(
        record(&sandbox),
        Some(json!({ "state": "on", "previous": { "type": "command", "command": "mine" } }))
    );
}

#[test]
fn the_status_line_follows_the_plugin_folder() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "config/settings.json",
        &pretty(&json!({ "statusLine": { "type": "command", "command": OURS, "padding": 1 } })),
    );
    sandbox.write("data/status-line.json", r#"{"state": "on"}"#);

    let output = status_line(&sandbox, &["hook", "status-line"], "/plugins/0.2.0", None);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert_eq!(
        settings(&sandbox),
        pretty(&json!({ "statusLine": {
            "type": "command",
            "command": "sh \"/plugins/0.2.0/bin/tightship\" status-line render",
            "padding": 1,
            "refreshInterval": 60,
        } }))
    );
}

#[test]
fn a_refresh_interval_set_by_hand_is_kept() {
    let sandbox = Sandbox::new();
    let paced = pretty(&json!({ "statusLine": {
        "type": "command",
        "command": OURS,
        "refreshInterval": 5,
    } }));
    sandbox.write("config/settings.json", &paced);
    sandbox.write("data/status-line.json", r#"{"state": "on"}"#);

    assert_succeeds(&hook(&sandbox, None));

    assert_eq!(settings(&sandbox), paced);
}

#[test]
fn a_status_line_replaced_by_hand_is_saved_and_replaced() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "config/settings.json",
        r#"{"statusLine": {"type": "command", "command": "theirs"}}"#,
    );
    sandbox.write(
        "data/status-line.json",
        r#"{"state": "on", "previous": {"type": "command", "command": "mine"}}"#,
    );

    let output = hook(&sandbox, None);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert_eq!(
        settings(&sandbox),
        pretty(
            &json!({ "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 } })
        )
    );
    assert_eq!(
        record(&sandbox),
        Some(json!({ "state": "on", "previous": { "type": "command", "command": "theirs" } }))
    );
}

#[cfg(unix)]
#[test]
fn a_linked_settings_file_stays_a_link_with_its_permissions() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let sandbox = Sandbox::new();
    let target = sandbox.write("elsewhere/settings.json", "{}");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).expect("the mode");
    fs::create_dir_all(sandbox.path("config")).expect("the config folder");
    symlink(&target, sandbox.path("config/settings.json")).expect("the link");

    assert_succeeds(&hook(&sandbox, None));

    assert_eq!(
        fs::read_link(sandbox.path("config/settings.json")).expect("still a link"),
        target
    );
    assert_eq!(
        fs::read_to_string(&target).expect("the target"),
        pretty(
            &json!({ "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 } })
        )
    );
    let mode = fs::metadata(&target)
        .expect("the target")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn damaged_input_fails_and_changes_nothing() {
    let sandbox = Sandbox::new();
    let file = sandbox.write("config/settings.json", "[1, 2");

    assert_fails_with(
        &hook(&sandbox, None),
        &format!(
            "error: the settings file {} is not a JSON object; nothing changed",
            file.display()
        ),
    );
    assert_eq!(settings(&sandbox), "[1, 2");
    assert!(!sandbox.path("data").exists());

    sandbox.write("config/settings.json", MINE);
    let damaged = sandbox.write("data/status-line.json", r#"{"state": "paused"}"#);
    assert_fails_with(
        &hook(&sandbox, None),
        &format!(
            "error: the status line record {} is damaged; delete it and start a new session",
            damaged.display()
        ),
    );
    assert_eq!(settings(&sandbox), MINE);

    let mut no_root = sandbox.tightship(&["hook", "status-line"]);
    no_root
        .env("TIGHTSHIP_SETTINGS", &file)
        .env("TIGHTSHIP_DATA", sandbox.path("data"));
    assert_fails_with(
        &run(&mut no_root),
        "error: TIGHTSHIP_ROOT is not set; run tightship through the plugin's bin/tightship",
    );
}

#[cfg(unix)]
#[test]
fn files_that_cannot_be_written_fail_without_a_half_change() {
    use std::os::unix::fs::PermissionsExt;

    let sandbox = Sandbox::new();
    let file = sandbox.write("config/settings.json", MINE);
    let lock = |relative: &str, mode| {
        fs::set_permissions(sandbox.path(relative), fs::Permissions::from_mode(mode))
            .expect("the mode");
    };
    lock("config", 0o555);

    let enabled = hook(&sandbox, None);

    lock("config", 0o755);
    assert_eq!(enabled.status.code(), Some(1));
    let stderr = stderr_of(&enabled);
    assert!(
        stderr.starts_with(&format!(
            "error: cannot write the settings file {}: ",
            file.display()
        )),
        "{stderr}"
    );
    assert_eq!(settings(&sandbox), MINE);
    assert_eq!(record(&sandbox), None);

    fs::create_dir_all(sandbox.path("data")).expect("the data folder");
    lock("data", 0o555);

    let unrecorded = hook(&sandbox, None);

    lock("data", 0o755);
    assert_eq!(unrecorded.status.code(), Some(1));
    let stderr = stderr_of(&unrecorded);
    assert!(
        stderr.starts_with(&format!(
            "error: cannot write the status line record {}: ",
            sandbox.path("data/status-line.json").display()
        )),
        "{stderr}"
    );
}

#[test]
fn scenario_9_status_line_enables_renders_and_restores_via_options_toml() {
    let sandbox = Sandbox::new();
    let previous_status_line = json!({ "type": "command", "command": "previous-status-cmd" });
    sandbox.write(
        "config/settings.json",
        &pretty(&json!({
            "statusLine": previous_status_line,
            "theme": "dark"
        })),
    );
    sandbox.write("data/options.toml", "status_line = true\n");

    let output = hook(&sandbox, None);
    assert_succeeds(&output);

    assert_eq!(
        settings(&sandbox),
        pretty(&json!({
            "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 },
            "theme": "dark"
        }))
    );
    assert_eq!(
        record(&sandbox),
        Some(json!({
            "state": "on",
            "previous": previous_status_line
        }))
    );

    checkout(&sandbox, "feat/59-antigravity-cli-plugin");
    let render_output = render(&sandbox, "request", Some("200"));
    assert_succeeds(&render_output);
    let rendered_text = stdout_of(&render_output);
    assert!(rendered_text.lines().count() >= 2, "{rendered_text}");

    sandbox.write("data/options.toml", "status_line = false\n");
    let output = hook(&sandbox, None);
    assert_succeeds(&output);

    assert_eq!(
        settings(&sandbox),
        pretty(&json!({
            "statusLine": previous_status_line,
            "theme": "dark"
        }))
    );
    assert_eq!(record(&sandbox), None);
}

#[test]
fn the_status_line_recognizes_its_launchers_and_binaries_on_every_platform() {
    let sandbox = Sandbox::new();
    sandbox.write("data/status-line.json", r#"{"state": "on"}"#);

    for command in [
        r#""C:\path\bin\tightship.cmd" status-line render"#,
        r#""C:/path/bin/tightship.cmd" status-line render"#,
        r#""C:\path\1.0.2\bin\windows-x64\tightship.exe" status-line render"#,
        r#""/path/1.0.2/bin/linux-x64/tightship" status-line render"#,
        r#"C:\path\bin\tightship.cmd status-line render"#,
        r#"C:\PROGRA~1\bin\tightship.cmd status-line render"#,
        r#"C:/PROGRA~1/bin/tightship.cmd status-line render"#,
    ] {
        sandbox.write(
            "config/settings.json",
            &pretty(&json!({ "statusLine": { "type": "command", "command": command } })),
        );

        assert_succeeds(&hook(&sandbox, None));
        assert_eq!(
            settings(&sandbox),
            pretty(
                &json!({ "statusLine": { "type": "command", "command": OURS, "refreshInterval": 60 } })
            ),
            "{command}"
        );
        assert_eq!(
            record(&sandbox),
            Some(json!({ "state": "on" })),
            "{command}"
        );
    }
}

#[cfg(windows)]
#[test]
fn on_windows_the_status_line_command_runs_through_every_shell_from_a_folder_with_a_space() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "plugin with space/bin/tightship.cmd",
        "@echo off\r\necho %*> \"%~dp0..\\ran.txt\"\r\n",
    );
    let root = sandbox.path("plugin with space");

    assert_succeeds(&status_line(
        &sandbox,
        &["hook", "status-line"],
        &root.display().to_string(),
        None,
    ));

    let written: serde_json::Value = serde_json::from_str(&settings(&sandbox)).expect("JSON");
    let command = written["statusLine"]["command"]
        .as_str()
        .expect("a command")
        .to_owned();
    assert!(!command.contains(['"', '\\']), "{command}");
    for (shell, flag) in [("cmd", "/c"), ("sh", "-c"), ("powershell", "-Command")] {
        let ran = root.join("ran.txt");
        let _ = fs::remove_file(&ran);
        let output = std::process::Command::new(shell)
            .args([flag, &command])
            .output()
            .expect("the shell runs");
        assert_succeeds(&output);
        assert_eq!(
            fs::read_to_string(&ran).expect("the stub ran"),
            "status-line render\r\n",
            "{shell}"
        );
    }
}
