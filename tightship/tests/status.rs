#[allow(dead_code)]
mod common;

use std::fs;
use std::path::Path;
use std::process::Output;

use unicode_width::UnicodeWidthStr;

use common::{Sandbox, assert_succeeds, stderr_of, stdout_of, with_stdin};

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
