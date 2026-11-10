#[allow(dead_code)]
mod common;

use std::fs;
use std::process::Output;

use common::{Sandbox, assert_succeeds, run, stdout_of};

const REMOVED: &str =
    "tightship removed the plugins it replaces, and everything they left: ponytail, humanizer.\n";
const PONYTAIL_STATUS_LINE: &str = "sh \"/home/.claude/ponytail-statusline.sh\"";

fn clean(sandbox: &Sandbox, data: &serde_json::Value) -> Output {
    sandbox.write("plugin/clean.json", &data.to_string());
    let mut command = sandbox.tightship(&["hook", "clean"]);
    command
        .env("HOME", sandbox.path("home"))
        .env("USERPROFILE", sandbox.path("home"))
        .env("TIGHTSHIP_CONFIG", sandbox.path("home/.claude"))
        .env("TIGHTSHIP_CLEAN", sandbox.path("plugin/clean.json"))
        .env(
            "TIGHTSHIP_SETTINGS",
            sandbox.path("home/.claude/settings.json"),
        )
        .env("TIGHTSHIP_DATA", sandbox.path("data"));
    run(&mut command)
}

fn leftovers(extra: serde_json::Value) -> serde_json::Value {
    let mut data = serde_json::json!({
        "names": ["ponytail", "humanizer"],
        "commands": [{ "when": "{config}/plugins/marketplaces/ponytail", "run": "echo ran > uninstalled" }],
        "paths": ["{config}/plugins/cache/ponytail", "{config}/.ponytail-active", "~/.config/ponytail"],
        "json": [
            { "file": "{config}/settings.json", "remove": "/enabledPlugins/ponytail@ponytail" },
            { "file": "~/manifest.json", "remove": "/imports[name=ponytail]" },
        ],
        "status-line": ["ponytail-statusline"],
    });
    if let (Some(data), serde_json::Value::Object(extra)) = (data.as_object_mut(), extra) {
        data.extend(extra);
    }
    data
}

fn json(sandbox: &Sandbox, relative: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(sandbox.path(relative)).expect("the JSON file"))
        .expect("valid JSON")
}

#[test]
fn removes_every_listed_leftover_and_says_so_once() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "home/.claude/plugins/marketplaces/ponytail/marketplace.json",
        "{}",
    );
    sandbox.write(
        "home/.claude/plugins/cache/ponytail/1.0.0/plugin.json",
        "{}",
    );
    sandbox.write("home/.claude/.ponytail-active", "full");
    sandbox.write("home/.config/ponytail/band/ledger.jsonl", "{}");
    sandbox.write("home/.config/ponytail-not-listed", "stays");
    sandbox.write(
        "home/.claude/settings.json",
        r#"{"enabledPlugins": {"ponytail@ponytail": true, "frontend-design@x": true},
            "statusLine": {"type": "command", "command": "sh \"/home/.claude/ponytail-statusline.sh\""}}"#,
    );
    sandbox.write(
        "home/manifest.json",
        r#"{"imports": [{"name": "ponytail"}, {"name": "other"}]}"#,
    );

    let output = clean(&sandbox, &leftovers(serde_json::json!({})));

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), REMOVED);
    assert!(sandbox.path("uninstalled").is_file(), "the command ran");
    for gone in [
        "home/.claude/plugins/cache/ponytail",
        "home/.claude/.ponytail-active",
        "home/.config/ponytail",
    ] {
        assert!(!sandbox.path(gone).exists(), "{gone} is left");
    }
    assert!(sandbox.path("home/.config/ponytail-not-listed").exists());
    assert_eq!(
        json(&sandbox, "home/.claude/settings.json"),
        serde_json::json!({ "enabledPlugins": { "frontend-design@x": true } })
    );
    assert_eq!(
        json(&sandbox, "home/manifest.json"),
        serde_json::json!({ "imports": [{ "name": "other" }] })
    );

    fs::remove_dir_all(sandbox.path("home/.claude/plugins/marketplaces/ponytail"))
        .expect("the uninstall removes the catalog");
    fs::remove_file(sandbox.path("uninstalled")).expect("the marker");
    let again = clean(&sandbox, &leftovers(serde_json::json!({})));
    assert_succeeds(&again);
    assert_eq!(stdout_of(&again), "");
    assert!(
        !sandbox.path("uninstalled").exists(),
        "no command runs twice"
    );
}

#[test]
fn nothing_left_prints_nothing_and_runs_no_command() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.path("home")).expect("the home folder");

    let output = clean(&sandbox, &leftovers(serde_json::json!({})));

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert!(!sandbox.path("uninstalled").exists());
    assert!(!sandbox.path("home/.claude/settings.json").exists());
}

#[test]
fn no_clean_data_does_nothing() {
    let sandbox = Sandbox::new();
    let mut unset = sandbox.tightship(&["hook", "clean"]);
    let output = run(&mut unset);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");

    sandbox.write("home/.ponytail-active", "full");
    let output = clean(
        &sandbox,
        &serde_json::json!({ "names": "not a list", "paths": ["~/.ponytail-active"] }),
    );
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert!(sandbox.path("home/.ponytail-active").exists());
}

#[test]
fn a_failed_command_is_reported_and_tried_again() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "home/.claude/plugins/marketplaces/ponytail/marketplace.json",
        "{}",
    );
    let data = leftovers(serde_json::json!({
        "commands": [{ "when": "{config}/plugins/marketplaces/ponytail", "run": "exit 3" }],
    }));

    for _ in 0..2 {
        let output = clean(&sandbox, &data);
        assert_succeeds(&output);
        let stdout = stdout_of(&output);
        assert!(
            stdout.starts_with(
                "tightship could not remove {config}/plugins/marketplaces/ponytail: `exit 3` exited with"
            ),
            "{stdout}"
        );
        assert!(
            stdout.ends_with(". It tries again at the next session.\n"),
            "{stdout}"
        );
    }
}

#[cfg(unix)]
#[test]
fn a_leftover_that_cannot_be_removed_is_reported_and_the_rest_still_goes() {
    use std::os::unix::fs::PermissionsExt;

    let sandbox = Sandbox::new();
    sandbox.write("home/.config/ponytail/band/ledger.jsonl", "{}");
    sandbox.write("home/.claude/.ponytail-active", "full");
    let locked = sandbox.path("home/.config");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).expect("lock the folder");

    let output = clean(&sandbox, &leftovers(serde_json::json!({})));

    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("unlock the folder");
    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], REMOVED.trim_end());
    assert!(
        lines[1].starts_with("tightship could not remove ~/.config/ponytail: ")
            && lines[1].ends_with(". It tries again at the next session."),
        "{stdout}"
    );
    assert!(!sandbox.path("home/.claude/.ponytail-active").exists());
    assert!(sandbox.path("home/.config/ponytail").exists());
}

#[test]
fn the_adopters_own_status_line_stays() {
    let sandbox = Sandbox::new();
    let settings = r#"{"statusLine": {"type": "command", "command": "my-status"}}"#;
    sandbox.write("home/.claude/settings.json", settings);

    let output = clean(&sandbox, &leftovers(serde_json::json!({})));

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert_eq!(
        fs::read_to_string(sandbox.path("home/.claude/settings.json")).expect("settings"),
        settings
    );
}

#[test]
fn ponytails_status_line_is_never_restored() {
    let sandbox = Sandbox::new();
    let ours = "sh \"/plugins/2.0.0/bin/tightship\" status-line render";
    sandbox.write(
        "home/.claude/settings.json",
        &serde_json::json!({ "statusLine": { "type": "command", "command": ours } }).to_string(),
    );
    sandbox.write(
        "data/status-line.json",
        &serde_json::json!({
            "state": "on",
            "previous": { "type": "command", "command": PONYTAIL_STATUS_LINE },
        })
        .to_string(),
    );

    let output = clean(&sandbox, &leftovers(serde_json::json!({})));
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), REMOVED);
    assert_eq!(
        json(&sandbox, "data/status-line.json"),
        serde_json::json!({ "state": "on" })
    );

    let mut off = sandbox.tightship(&["hook", "status-line"]);
    off.env(
        "TIGHTSHIP_SETTINGS",
        sandbox.path("home/.claude/settings.json"),
    )
    .env("TIGHTSHIP_DATA", sandbox.path("data"))
    .env("TIGHTSHIP_ROOT", "/plugins/2.0.0")
    .env("TIGHTSHIP_STATUS_LINE", "false");
    assert_succeeds(&run(&mut off));
    assert_eq!(
        json(&sandbox, "home/.claude/settings.json"),
        serde_json::json!({})
    );
}

#[test]
fn an_older_version_no_session_uses_is_removed_silently() {
    let sandbox = Sandbox::new();
    let versions = "home/.claude/plugins/cache/tightship/tightship";
    sandbox.write(&format!("{versions}/2.0.0/.orphaned_at"), "1");
    sandbox.write(&format!("{versions}/1.2.0/.orphaned_at"), "1");
    fs::create_dir_all(sandbox.path(&format!("{versions}/1.2.0/.in_use"))).expect(".in_use");
    sandbox.write(&format!("{versions}/1.1.0/.orphaned_at"), "1");
    sandbox.write(&format!("{versions}/1.1.0/.in_use/4242"), "");
    sandbox.write(&format!("{versions}/1.0.0/plugin.json"), "{}");
    let data = serde_json::json!({ "names": ["ponytail"], "previous-versions": true });

    sandbox.write("plugin/clean.json", &data.to_string());
    let mut command = sandbox.tightship(&["hook", "clean"]);
    command
        .env("HOME", sandbox.path("home"))
        .env("TIGHTSHIP_CLEAN", sandbox.path("plugin/clean.json"))
        .env("TIGHTSHIP_ROOT", sandbox.path(&format!("{versions}/2.0.0")));
    let output = run(&mut command);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert!(!sandbox.path(&format!("{versions}/1.2.0")).exists());
    assert!(
        sandbox
            .path(&format!("{versions}/2.0.0/.orphaned_at"))
            .exists()
    );
    assert!(
        sandbox.path(&format!("{versions}/1.1.0")).exists(),
        "in use"
    );
    assert!(
        sandbox.path(&format!("{versions}/1.0.0")).exists(),
        "not orphaned"
    );
}
