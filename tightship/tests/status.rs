#[allow(dead_code)]
mod common;

use std::fs;
use std::path::Path;
use std::process::Output;

use serde_json::json;
use unicode_width::UnicodeWidthStr;

use common::{Sandbox, assert_fails_with, assert_succeeds, run, stderr_of, stdout_of, with_stdin};

const ROOT: &str = "/plugins/0.1.0";
const OURS: &str = "sh \"/plugins/0.1.0/bin/tightship\" status-line render";
const MINE: &str = r#"{"statusLine": {"type": "command", "command": "mine"}}"#;
const ON: &str = "tightship's status line is on. Turn the plugin's status_line option off to bring back the one you had.";

fn checkout(sandbox: &Sandbox, branch: &str) {
    sandbox.write(
        "tightship/.git/HEAD",
        &format!("ref: refs/heads/{branch}\n"),
    );
}

fn render(sandbox: &Sandbox, fixture: &str, columns: Option<&str>) -> Output {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/status")
        .join(format!("{fixture}.json"));
    let input = fs::read_to_string(file).expect("the fixture").replace(
        "/home/adopter/tightship",
        &sandbox.path("tightship").display().to_string(),
    );
    let mut command = sandbox.tightship(&["status-line", "render"]);
    command.env("TZ", "UTC");
    match columns {
        Some(columns) => command.env("COLUMNS", columns),
        None => command.env_remove("COLUMNS"),
    };
    with_stdin(&mut command, &input)
}

fn plain(output: &Output) -> String {
    let mut plain = String::new();
    let mut in_code = false;
    for character in stdout_of(output).chars() {
        match character {
            '\x1b' => in_code = true,
            'm' if in_code => in_code = false,
            _ if in_code => {}
            _ => plain.push(character),
        }
    }
    plain
}

#[test]
fn render_prints_both_rows_in_their_colors() {
    let sandbox = Sandbox::new();
    checkout(&sandbox, "feat/7-status-line");

    let output = render(&sandbox, "request", Some("200"));

    assert_succeeds(&output);
    let separator = " \x1b[2m·\x1b[0m ";
    let identity = [
        "\x1b[36mtightship\x1b[0m",
        "\x1b[2;37mATNexusLab/tightship\x1b[0m",
        "\x1b[33mfeat/7-status-line\x1b[0m",
        "\x1b[32mPR #31 (approved)\x1b[0m",
        "\x1b[1;37mOpus 5.5\x1b[0m",
        "\x1b[33mmedium\x1b[0m",
    ]
    .join(separator);
    let usage = [
        "\x1b[32mctx █░░░░░ 11%\x1b[0m \x1b[2m109k/1M\x1b[0m",
        "\x1b[32m5h ░░░░░░ 4%\x1b[0m \x1b[2m17:50\x1b[0m",
        "\x1b[33m7d ████░░ 66%\x1b[0m \x1b[2m17/10 15:00\x1b[0m",
    ]
    .join(separator);
    assert_eq!(stdout_of(&output), format!("{identity}\n{usage}\n"));
}

#[test]
fn a_narrow_terminal_drops_by_priority() {
    let sandbox = Sandbox::new();
    checkout(&sandbox, "feat/7-status-line");

    let output = render(&sandbox, "request", Some("60"));

    assert_succeeds(&output);
    let plain = plain(&output);
    let rows: Vec<&str> = plain.lines().collect();
    assert_eq!(rows[0], "tightship · feat/7-status-line · Opus 5.5");
    assert!(rows.iter().all(|row| row.width() <= 56), "{plain}");
}

#[test]
fn a_long_branch_is_cut_before_anything_drops() {
    let sandbox = Sandbox::new();
    checkout(&sandbox, "feat/7-a-very-long-branch-name");

    let output = render(&sandbox, "request", Some("60"));

    assert_succeeds(&output);
    let plain = plain(&output);
    let identity = plain.lines().next().expect("the identity row");
    assert!(
        identity
            .split(" · ")
            .any(|segment| segment == "feat/7-a-very-long-…"),
        "{identity}"
    );
}

#[test]
fn no_field_prints_nothing_and_no_columns_prints_whole_rows() {
    let sandbox = Sandbox::new();
    checkout(&sandbox, "feat/7-status-line");

    let empty = render(&sandbox, "empty", None);
    let whole = render(&sandbox, "request", None);

    assert_succeeds(&empty);
    assert_eq!(stdout_of(&empty), "");
    assert_succeeds(&whole);
    assert_eq!(
        plain(&whole),
        "tightship · ATNexusLab/tightship · feat/7-status-line · PR #31 (approved) · Opus 5.5 · medium\n\
         ctx █░░░░░ 11% 109k/1M · 5h ░░░░░░ 4% 17:50 · 7d ████░░ 66% 17/10 15:00\n"
    );
}

#[test]
fn input_that_is_not_json_fails() {
    let sandbox = Sandbox::new();

    let output = with_stdin(&mut sandbox.tightship(&["status-line", "render"]), "nope");

    let stderr = stderr_of(&output);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stderr.starts_with("error: the status line input is not a JSON object: "),
        "{stderr}"
    );
}

fn status_line(sandbox: &Sandbox, arguments: &[&str], root: &str, option: Option<&str>) -> Output {
    let mut command = sandbox.tightship(arguments);
    command
        .env("TIGHTSHIP_SETTINGS", sandbox.path("config/settings.json"))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_ROOT", root);
    if let Some(option) = option {
        command.env("TIGHTSHIP_STATUS_LINE", option);
    }
    run(&mut command)
}

fn hook(sandbox: &Sandbox, option: Option<&str>) -> Output {
    status_line(sandbox, &["hook", "status-line"], ROOT, option)
}

fn agent(sandbox: &Sandbox, answer: &str) -> Output {
    status_line(sandbox, &["status-line", answer], ROOT, None)
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

fn offer(sandbox: &Sandbox) -> String {
    let quoted = |relative: &str| format!("'{}'", sandbox.path(relative).display());
    format!(
        "tightship can show its status line in this session. In your first reply, ask the adopter, word for word:\n\
         \"tightship can show its status line under the prompt. It replaces the status line in your user settings, and turning the plugin's status_line option off brings yours back. Turn it on?\"\n\
         On yes, run: TIGHTSHIP_SETTINGS={} TIGHTSHIP_DATA={} TIGHTSHIP_ROOT='{ROOT}' tightship status-line enable\n\
         On no, run: TIGHTSHIP_DATA={} tightship status-line decline\n",
        quoted("config/settings.json"),
        quoted("data"),
        quoted("data"),
    )
}

#[test]
fn the_session_start_offers_the_status_line() {
    let sandbox = Sandbox::new();
    sandbox.write("config/settings.json", MINE);
    fs::create_dir_all(sandbox.path("data")).expect("the data folder");

    let output = hook(&sandbox, None);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), offer(&sandbox));
    assert_eq!(settings(&sandbox), MINE);
    assert_eq!(record(&sandbox), None);
}

#[test]
fn yes_saves_the_previous_status_line_and_off_puts_it_back_in_place() {
    let sandbox = Sandbox::new();
    let mine = json!({ "type": "command", "command": "mine" });
    sandbox.write(
        "config/settings.json",
        r#"{"model": "opus", "statusLine": {"type": "command", "command": "mine"}, "theme": "dark"}"#,
    );

    let enabled = agent(&sandbox, "enable");

    assert_succeeds(&enabled);
    assert_eq!(stdout_of(&enabled), format!("{ON}\n"));
    assert_eq!(
        settings(&sandbox),
        pretty(&json!({
            "model": "opus",
            "statusLine": { "type": "command", "command": OURS },
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
fn yes_with_no_status_line_before_removes_the_key_when_off() {
    let sandbox = Sandbox::new();
    let ours = pretty(&json!({ "statusLine": { "type": "command", "command": OURS } }));

    assert_succeeds(&agent(&sandbox, "enable"));
    assert_eq!(settings(&sandbox), ours);
    assert_eq!(record(&sandbox), Some(json!({ "state": "on" })));

    let again = agent(&sandbox, "enable");
    let declined = agent(&sandbox, "decline");

    assert_eq!(
        stdout_of(&again),
        "tightship's status line is already on.\n"
    );
    assert_eq!(
        stdout_of(&declined),
        "tightship's status line is on; turn the plugin's status_line option off to bring back the one you had.\n"
    );
    assert_eq!(settings(&sandbox), ours);
    assert_eq!(record(&sandbox), Some(json!({ "state": "on" })));

    sandbox.write(
        "config/settings.json",
        &pretty(&json!({
            "statusLine": { "type": "command", "command": OURS },
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
fn no_is_kept_until_the_option_is_turned_off_and_on() {
    let sandbox = Sandbox::new();
    sandbox.write("config/settings.json", MINE);

    let declined = agent(&sandbox, "decline");

    assert_succeeds(&declined);
    assert_eq!(
        stdout_of(&declined),
        "The status line stays as it is. tightship will not ask again until its status_line option is turned off and on.\n"
    );
    assert_eq!(settings(&sandbox), MINE);
    assert_eq!(stdout_of(&hook(&sandbox, None)), "");

    let off = hook(&sandbox, Some("false"));
    let on = hook(&sandbox, None);

    assert_eq!(stdout_of(&off), "");
    assert_eq!(record(&sandbox), None);
    assert_eq!(stdout_of(&on), offer(&sandbox));
    assert_eq!(settings(&sandbox), MINE);
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
        } }))
    );
}

#[test]
fn a_status_line_replaced_by_hand_counts_as_declined() {
    let sandbox = Sandbox::new();
    let theirs = r#"{"statusLine": {"type": "command", "command": "theirs"}}"#;
    sandbox.write("config/settings.json", theirs);
    sandbox.write("data/status-line.json", r#"{"state": "on"}"#);

    let output = hook(&sandbox, None);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    assert_eq!(settings(&sandbox), theirs);
    assert_eq!(record(&sandbox), Some(json!({ "state": "declined" })));
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

    assert_succeeds(&agent(&sandbox, "enable"));

    assert_eq!(
        fs::read_link(sandbox.path("config/settings.json")).expect("still a link"),
        target
    );
    assert_eq!(
        fs::read_to_string(&target).expect("the target"),
        pretty(&json!({ "statusLine": { "type": "command", "command": OURS } }))
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
        &agent(&sandbox, "enable"),
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

    let mut no_root = sandbox.tightship(&["status-line", "enable"]);
    no_root
        .env("TIGHTSHIP_SETTINGS", &file)
        .env("TIGHTSHIP_DATA", sandbox.path("data"));
    assert_fails_with(
        &run(&mut no_root),
        "error: TIGHTSHIP_ROOT is not set; run the exact command the session start printed",
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

    let enabled = agent(&sandbox, "enable");

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

    let declined = agent(&sandbox, "decline");

    lock("data", 0o755);
    assert_eq!(declined.status.code(), Some(1));
    let stderr = stderr_of(&declined);
    assert!(
        stderr.starts_with(&format!(
            "error: cannot write the status line record {}: ",
            sandbox.path("data/status-line.json").display()
        )),
        "{stderr}"
    );
}
