#[allow(dead_code)]
mod common;

use std::collections::BTreeMap;
use std::env::consts::EXE_SUFFIX;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use common::{Sandbox, assert_fails_with, assert_succeeds, run, stdout_of};

const SHA256: &str = "3e549264345effb6cbaf8ff0ca795d2d862a721a8144c760ba243e404e15ea89";
const OPTIONS: &str = "{\n  \"contract\": false\n}\n";

fn file_url(path: &Path) -> String {
    let path = path.display().to_string().replace('\\', "/");
    format!("file:///{}", path.trim_start_matches('/'))
}

fn plugin_at(sandbox: &Sandbox, version: &str, sha256: &str) {
    let latest = json!({
        "version": "1.2.0",
        "tag": "v1.2.0",
        "assets": { "cursor": { "archive": "tightship-cursor.zip", "sha256": sha256 } },
    });
    sandbox.write(
        "repo/releases/latest/download/latest.json",
        &latest.to_string(),
    );
    let archive = sandbox.write("repo/releases/download/v1.2.0/tightship-cursor.zip", "");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update/tightship-cursor.zip"),
        archive,
    )
    .expect("the release archive");
    let manifest = json!({
        "name": "tightship",
        "version": version,
        "repository": file_url(&sandbox.path("repo")),
    });
    sandbox.write("plugin/.cursor-plugin/plugin.json", &manifest.to_string());
    sandbox.write("plugin/.cursor-plugin/marketplace.json", "{}\n");
    sandbox.write("plugin/skills/old/SKILL.md", "old\n");
    sandbox.write("plugin/hooks.json", "{\"os\": \"old\"}\n");
    sandbox.write("data/options.json", OPTIONS);
}

fn updater(sandbox: &Sandbox, binary: &Path, arguments: &[&str]) -> Command {
    let mut command = sandbox.tightship_at(binary, arguments);
    command
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_ROOT", sandbox.path("plugin"))
        .env(
            "TIGHTSHIP_MANIFEST",
            sandbox.path("plugin/.cursor-plugin/plugin.json"),
        )
        .env("TIGHTSHIP_HARNESS", "cursor");
    command
}

fn update(sandbox: &Sandbox, arguments: &[&str]) -> Command {
    updater(
        sandbox,
        Path::new(env!("CARGO_BIN_EXE_tightship")),
        arguments,
    )
}

fn snapshot(sandbox: &Sandbox) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut folders = vec![sandbox.path("plugin")];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).expect("the plugin folder") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                folders.push(path);
            } else {
                files.insert(
                    path.display().to_string(),
                    fs::read(&path).expect("a plugin file"),
                );
            }
        }
    }
    files
}

fn read(sandbox: &Sandbox, relative: &str) -> String {
    fs::read_to_string(sandbox.path(relative)).expect("the file")
}

fn notice(from: &str, to: &str) -> String {
    format!(
        "# tightship update\n\ntightship was updated from {from} to {to} since the last session. Tell the user, word for word: \"tightship was updated from {from} to {to}.\"\n"
    )
}

#[test]
fn a_newer_release_replaces_the_plugin_and_keeps_the_options() {
    let sandbox = Sandbox::new();
    plugin_at(&sandbox, "1.1.1", SHA256);
    let binary = sandbox.path(&format!("plugin/bin/tightship{EXE_SUFFIX}"));
    fs::create_dir_all(binary.parent().expect("the bin folder")).expect("the bin folder");
    fs::copy(env!("CARGO_BIN_EXE_tightship"), &binary).expect("the running binary");

    let output = run(&mut updater(&sandbox, &binary, &["update"]));

    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "Updated tightship from 1.1.1 to 1.2.0. The next session loads it.\n"
    );
    assert!(read(&sandbox, "plugin/.cursor-plugin/plugin.json").contains("\"1.2.0\""));
    assert_eq!(read(&sandbox, "plugin/skills/new/SKILL.md"), "new\n");
    let hooks = if cfg!(windows) { "windows" } else { "unix" };
    assert_eq!(
        read(&sandbox, "plugin/hooks.json"),
        format!("{{\"os\": \"{hooks}\"}}\n")
    );
    assert_eq!(
        read(&sandbox, "plugin/.cursor-plugin/marketplace.json"),
        "{}\n"
    );
    for gone in [
        "plugin/skills/old",
        "plugin/hooks.windows.json",
        "plugin/.update",
        "data/update",
    ] {
        assert!(!sandbox.path(gone).exists(), "{gone} is still there");
    }
    assert!(!binary.exists());
    assert_eq!(read(&sandbox, "data/options.json"), OPTIONS);
    let state: Value =
        serde_json::from_str(&read(&sandbox, "data/update.json")).expect("the update state");
    assert_eq!(
        state["notice"],
        json!({ "kind": "updated", "from": "1.1.1", "to": "1.2.0" })
    );
}

#[test]
fn the_notice_is_printed_once() {
    let sandbox = Sandbox::new();
    let checked_at = jiff::Timestamp::now().to_string();
    sandbox.write(
        "data/update.json",
        &json!({
            "checked_at": checked_at,
            "notice": { "kind": "updated", "from": "1.1.1", "to": "1.2.0" },
        })
        .to_string(),
    );

    let first = run(&mut update(&sandbox, &["hook", "update"]));
    let second = run(&mut update(&sandbox, &["hook", "update"]));

    assert_succeeds(&first);
    assert_eq!(stdout_of(&first), notice("1.1.1", "1.2.0"));
    assert_succeeds(&second);
    assert_eq!(stdout_of(&second), "");
}

#[test]
fn the_session_start_starts_the_update_when_a_check_is_due() {
    let sandbox = Sandbox::new();
    plugin_at(&sandbox, "1.1.1", SHA256);

    let output = run(&mut update(&sandbox, &["hook", "update"]));

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    let deadline = Instant::now() + Duration::from_secs(60);
    while !read(&sandbox, "plugin/.cursor-plugin/plugin.json").contains("\"1.2.0\"")
        || sandbox.path("data/update.lock").exists()
    {
        assert!(Instant::now() < deadline, "the update never ran");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn the_option_off_leaves_the_plugin_as_it_is() {
    let sandbox = Sandbox::new();
    plugin_at(&sandbox, "1.1.1", SHA256);
    let before = snapshot(&sandbox);

    let output = run(update(&sandbox, &["update"]).env("TIGHTSHIP_AUTO_UPDATE", "false"));

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert_eq!(snapshot(&sandbox), before);
    assert!(!sandbox.path("data/update.json").exists());
}

#[test]
fn a_mismatched_checksum_changes_nothing_and_is_announced() {
    let sandbox = Sandbox::new();
    let published = "0".repeat(64);
    plugin_at(&sandbox, "1.1.1", &published);
    let before = snapshot(&sandbox);

    let output = run(&mut update(&sandbox, &["update"]));
    let first = run(&mut update(&sandbox, &["hook", "update"]));
    let second = run(&mut update(&sandbox, &["hook", "update"]));

    assert_fails_with(
        &output,
        &format!(
            "error: tightship-cursor.zip has sha256 {SHA256}, and the release v1.2.0 published {published}; nothing changed"
        ),
    );
    assert_eq!(snapshot(&sandbox), before);
    assert_succeeds(&first);
    assert_eq!(
        stdout_of(&first),
        "# tightship update\n\nThe tightship 1.2.0 release did not match the sha256 it published, so it was not installed, and 1.1.1 stays. Tell the user, word for word: \"tightship refused the 1.2.0 update: its download did not match the sha256 the release published.\"\n"
    );
    assert_eq!(stdout_of(&second), "");
}

#[test]
fn a_release_that_is_not_newer_changes_nothing() {
    let sandbox = Sandbox::new();
    plugin_at(&sandbox, "1.2.0", SHA256);
    let before = snapshot(&sandbox);

    let output = run(&mut update(&sandbox, &["update"]));

    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "tightship 1.2.0 is the latest release.\n"
    );
    assert_eq!(snapshot(&sandbox), before);
}

#[test]
fn a_running_update_holds_the_lock() {
    let sandbox = Sandbox::new();
    plugin_at(&sandbox, "1.1.1", SHA256);
    sandbox.write("data/update.lock", "");
    let before = snapshot(&sandbox);

    let output = run(&mut update(&sandbox, &["update"]));

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "Another tightship update is running.\n");
    assert_eq!(snapshot(&sandbox), before);
}

#[test]
fn the_marketplace_entry_follows_the_option() {
    let sandbox = Sandbox::new();
    let manifest = sandbox.write(
        "claude/.claude-plugin/plugin.json",
        r#"{"name": "tightship", "version": "1.1.1", "repository": "https://github.com/example/tightship"}"#,
    );
    let settings = sandbox.write(
        "settings.json",
        r#"{"model": "opus", "extraKnownMarketplaces": {"tightship": {"source": {"source": "github", "repo": "example/tightship"}}}, "theme": "dark"}"#,
    );
    let empty = sandbox.write("empty/settings.json", "{}");
    let hook = |file: &Path, auto_update: &str| {
        let mut command = sandbox.tightship(&["hook", "marketplace-auto-update"]);
        command
            .env("TIGHTSHIP_MANIFEST", &manifest)
            .env("TIGHTSHIP_SETTINGS", file)
            .env("TIGHTSHIP_AUTO_UPDATE", auto_update);
        assert_succeeds(&run(&mut command));
        let written: Value = serde_json::from_str(&fs::read_to_string(file).expect("the settings"))
            .expect("JSON settings");
        written.to_string()
    };
    let entry = |auto_update: bool| {
        format!(
            r#"{{"model":"opus","extraKnownMarketplaces":{{"tightship":{{"source":{{"source":"github","repo":"example/tightship"}},"autoUpdate":{auto_update}}}}},"theme":"dark"}}"#
        )
    };

    assert_eq!(hook(&settings, "true"), entry(true));
    assert_eq!(hook(&settings, "false"), entry(false));
    assert_eq!(
        hook(&empty, "true"),
        r#"{"extraKnownMarketplaces":{"tightship":{"source":{"source":"github","repo":"example/tightship"},"autoUpdate":true}}}"#
    );
}
