//! Rendering the rows, fitting them to the width, and the cached rate limits.

use super::*;

fn render_remembering(sandbox: &Sandbox, fixture_name: &str) -> Output {
    let mut command = sandbox.tightship(&["status-line", "render"]);
    command
        .env("TZ", "UTC")
        .env_remove("COLUMNS")
        .env("TIGHTSHIP_DATA", sandbox.path("data"));
    with_stdin(&mut command, &read_fixture(sandbox, fixture_name))
}

fn now() -> i64 {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("a clock after 1970");
    i64::try_from(elapsed.as_secs()).expect("seconds that fit")
}

fn cache(sandbox: &Sandbox) -> serde_json::Value {
    let text = fs::read_to_string(sandbox.path("data/rate-limits.json")).expect("the cache");
    serde_json::from_str(&text).expect("a JSON cache")
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
fn a_harness_without_a_model_prefix_keeps_the_whole_model_name() {
    let sandbox = Sandbox::new();
    checkout(&sandbox, "feat/7-status-line");

    let output = render_with_prefix(&sandbox, "request", Some("200"), None);

    assert_succeeds(&output);
    assert!(
        plain(&output).contains(" · Claude Opus 5.5 · "),
        "{}",
        plain(&output)
    );
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
    assert_eq!(
        rows[1],
        "ctx █░░░░░ 11% 109k/1M · 5h ░░░░░░ 4% · 7d ████░░ 66%"
    );
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
fn a_window_missing_from_the_input_shows_its_last_reading() {
    let sandbox = Sandbox::new();
    let reset = now() + 3600;
    let seven_day = json!({ "used_percentage": 66, "resets_at": reset });
    sandbox.write(
        "data/rate-limits.json",
        &json!({ "seven_day": seven_day }).to_string(),
    );

    let output = render_remembering(&sandbox, "five-hour-only");

    assert_succeeds(&output);
    assert!(
        plain(&output).contains("5h ░░░░░░ 4% 17:50 · 7d ████░░ 66%"),
        "{}",
        plain(&output)
    );
    assert_eq!(
        cache(&sandbox),
        json!({
            "five_hour": { "used_percentage": 4, "resets_at": 1792086600 },
            "seven_day": seven_day,
        })
    );
}

#[test]
fn a_reading_past_its_reset_is_forgotten() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "data/rate-limits.json",
        &json!({ "seven_day": { "used_percentage": 66, "resets_at": now() - 60 } }).to_string(),
    );

    let output = render_remembering(&sandbox, "five-hour-only");

    assert_succeeds(&output);
    assert!(!plain(&output).contains("7d"), "{}", plain(&output));
    assert_eq!(
        cache(&sandbox),
        json!({ "five_hour": { "used_percentage": 4, "resets_at": 1792086600 } })
    );
}

#[test]
fn a_damaged_cache_is_replaced_and_a_missing_data_folder_is_not_created() {
    let sandbox = Sandbox::new();
    sandbox.write("data/rate-limits.json", "nope");

    let damaged = render_remembering(&sandbox, "request");
    fs::remove_dir_all(sandbox.path("data")).expect("the data folder");
    let without_folder = render_remembering(&sandbox, "request");

    assert_succeeds(&damaged);
    assert!(
        plain(&damaged).contains("7d ████░░ 66%"),
        "{}",
        plain(&damaged)
    );
    assert_succeeds(&without_folder);
    assert_eq!(plain(&without_folder), plain(&damaged));
    assert!(!sandbox.path("data").exists());
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
