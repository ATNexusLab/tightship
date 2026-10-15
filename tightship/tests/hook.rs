#[allow(dead_code)]
mod common;

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use common::{Sandbox, assert_fails_with, assert_succeeds, note, run, stdout_of, with_stdin};

fn project(sandbox: &Sandbox, name: &str) -> PathBuf {
    let folder = sandbox.path(name);
    fs::create_dir_all(folder.join(".git")).expect("the project's .git");
    folder
}

fn hook(sandbox: &Sandbox, subcommand: &str, working_directory: &Path) -> Command {
    let mut command = sandbox.tightship(&["hook", subcommand]);
    command
        .current_dir(working_directory)
        .env("TIGHTSHIP_VAULT", sandbox.path("vault"))
        .env("TIGHTSHIP_ENV_FILE", sandbox.path("env-file"))
        .env("TIGHTSHIP_DATA", sandbox.path("data"));
    command
}

fn export_line(sandbox: &Sandbox) -> String {
    format!(
        "export TIGHTSHIP_VAULT='{}'\n",
        sandbox.path("vault").display()
    )
}

#[test]
fn start_lists_the_pinned_notes_of_the_project_then_all() {
    let sandbox = Sandbox::new();
    note(
        &sandbox,
        "tightship",
        "mem-0002",
        "pinned: true\n",
        "Pinned here.",
    );
    note(&sandbox, "tightship", "mem-0003", "", "Not pinned.");
    note(
        &sandbox,
        "all",
        "mem-0001",
        "pinned: true\n",
        "Pinned everywhere.",
    );
    note(
        &sandbox,
        "other",
        "mem-0004",
        "pinned: true\n",
        "Pinned elsewhere.",
    );
    let working_directory = project(&sandbox, "tightship");
    fs::create_dir_all(working_directory.join("src")).expect("a subfolder");

    let output = run(&mut hook(&sandbox, "start", &working_directory.join("src")));

    assert_succeeds(&output);
    let vault = sandbox.path("vault");
    let vault = vault.display();
    assert_eq!(
        stdout_of(&output),
        format!(
            "# Vault — curated cross-harness memory\n\n\
             Scope `tightship` and `all`. Before non-trivial work, search them for the subsystem and the task:\n\
             `tightship vault search <term> <term>` — one line per match, its `why` stating\n\
             what not knowing it costs. Open one at `{vault}/<path>`. Rules: `{vault}/AGENTS.md`.\n\
             Listed below, only the pinned notes. A hint to verify against the repo, never truth.\n\
             Write with `tightship vault new --help`.\n\n\
             ## Scope — tightship\n\n\
             - [[mem-0002-a-note]] (2026-10-01) — the mem-0002 why costs real time\n\n\
             ## Scope — all\n\n\
             - [[mem-0001-a-note]] (2026-10-01) — the mem-0001 why costs real time\n\n\
             [1 more notes in these scopes — `tightship vault search <terms>`]\n\
             [other scopes: other(1) — `tightship vault search --everywhere <terms>`]\n"
        )
    );
    let environment = fs::read_to_string(sandbox.path("env-file")).expect("the environment file");
    assert!(
        environment.ends_with(&export_line(&sandbox)),
        "{environment}"
    );
}

#[test]
fn start_shows_the_handoff_apart_from_the_pinned_notes() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "vault/notes/tightship/mem-0005-handoff.md",
        "---\nid: mem-0005\ntype: handoff\nproject: tightship\nwhy: the next session picks up here\npinned: true\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-14\n---\n\n# Handoff\n\nStep 3 of 5.\n",
    );
    let working_directory = project(&sandbox, "tightship");

    let output = run(&mut hook(&sandbox, "start", &working_directory));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains(
            "\n\n## Handoff — tightship · mem-0005 · 2026-10-01 → 2026-10-14\n\nClaim it with the handoff command once you pick the work up.\n\n# Handoff\n\nStep 3 of 5.\n"
        ),
        "{stdout}"
    );
    assert!(!stdout.contains("## Scope —"), "{stdout}");
}

#[test]
fn start_cuts_the_pinned_notes_at_the_cap() {
    let sandbox = Sandbox::new();
    let body = "x".repeat(10);
    for number in 1..=40 {
        sandbox.write(
            &format!("vault/notes/tightship/mem-{number:04}-long.md"),
            &format!(
                "---\nid: mem-{number:04}\ntype: gotcha\nproject: tightship\nwhy: {}\npinned: true\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-01\n---\n\n{body}\n",
                "é".repeat(260)
            ),
        );
    }
    let working_directory = project(&sandbox, "tightship");

    let output = run(&mut hook(&sandbox, "start", &working_directory));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(stdout.encode_utf16().count() <= 9000, "{}", stdout.len());
    let listed = stdout.matches("- [[mem-").count();
    assert!(listed > 0 && listed < 40, "{listed}");
    assert!(
        stdout.ends_with(&format!(
            "[{} pinned notes cut at the 9000-character cap — `tightship vault lint` names the scope]\n",
            40 - listed
        )),
        "{stdout}"
    );
}

#[test]
fn start_takes_the_scope_whose_name_prefixes_the_project_folder() {
    let sandbox = Sandbox::new();
    note(
        &sandbox,
        "tightship",
        "mem-0001",
        "pinned: true\n",
        "Pinned.",
    );
    fs::create_dir_all(sandbox.path("vault/notes/all")).expect("the all scope");
    let working_directory = project(&sandbox, "tightship-docs");

    let output = run(&mut hook(&sandbox, "start", &working_directory));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(stdout.contains("Scope `tightship` and `all`."), "{stdout}");
    assert!(stdout.contains("- [[mem-0001-a-note]]"), "{stdout}");
}

#[test]
fn start_without_a_usable_vault_says_so_and_exports_nothing() {
    let sandbox = Sandbox::new();
    let gone = sandbox.path("gone");
    let gone = gone.to_str().expect("a UTF-8 path");
    let cases = [
        (
            None,
            "Memory is on and no vault folder is set, so this session has no vault.".to_owned(),
        ),
        (
            Some("vault"),
            "Memory is on and the vault folder `vault` is not an absolute path, so this session has no".to_owned(),
        ),
        (
            Some(gone),
            format!("Memory is on and the vault folder `{gone}` does not exist, so this session has no vault."),
        ),
    ];

    for (vault, opening) in cases {
        let mut command = hook(&sandbox, "start", &sandbox.path(""));
        command.env_remove("TIGHTSHIP_VAULT");
        if let Some(vault) = vault {
            command.env("TIGHTSHIP_VAULT", vault);
        }
        let output = run(&mut command);
        assert_succeeds(&output);
        let stdout = stdout_of(&output);
        assert!(
            stdout.starts_with(&format!("# Vault\n\n{opening}")),
            "{stdout}"
        );
    }
    assert!(!sandbox.path("env-file").exists());
}

#[test]
fn start_leaves_out_files_that_are_not_notes() {
    let sandbox = Sandbox::new();
    note(
        &sandbox,
        "tightship",
        "mem-0001",
        "pinned: true\n",
        "Pinned.",
    );
    sandbox.write("vault/notes/tightship/index.md", "# Index\n\n- a link\n");
    sandbox.write(
        "vault/notes/tightship/mem-0002-open.md",
        "---\nid: mem-0002\npinned: true\n\n# No closing line\n",
    );
    let working_directory = project(&sandbox, "tightship");

    let output = run(&mut hook(&sandbox, "start", &working_directory));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(stdout.contains("- [[mem-0001-a-note]]"), "{stdout}");
    assert!(!stdout.contains("mem-0002"), "{stdout}");
    assert!(!stdout.contains("more notes"), "{stdout}");
}

#[test]
fn export_writes_the_vault_to_the_environment_and_prints_nothing() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.path("vault")).expect("the vault");

    let output = run(&mut hook(&sandbox, "export", &sandbox.path("")));

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    let environment = fs::read_to_string(sandbox.path("env-file")).expect("the environment file");
    assert_eq!(environment, export_line(&sandbox));
}

#[test]
fn prompt_prints_the_checkpoint_every_10_prompts_and_sweeps_stale_counters() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.path("vault")).expect("the vault");
    let stale = sandbox.write("data/checkpoints/old-session", "3");
    File::options()
        .write(true)
        .open(&stale)
        .expect("the stale counter")
        .set_modified(SystemTime::now() - Duration::from_secs(8 * 24 * 60 * 60))
        .expect("an old modification time");

    let printed: Vec<u32> = (1..=20)
        .filter(|_| {
            let output = with_stdin(
                &mut hook(&sandbox, "prompt", &sandbox.path("")),
                r#"{"session_id":"abc"}"#,
            );
            assert_succeeds(&output);
            !stdout_of(&output).is_empty()
        })
        .collect();

    assert_eq!(printed, [10, 20]);
    let counter = fs::read_to_string(sandbox.path("data/checkpoints/abc")).expect("the counter");
    assert_eq!(counter, "20");
    assert!(!stale.exists());
    assert_succeeds(&with_stdin(
        &mut hook(&sandbox, "prompt", &sandbox.path("")),
        r#"{"session_id":"x/y"}"#,
    ));
    assert!(sandbox.path("data/checkpoints/x-y").exists());
}

#[test]
fn checkpoint_prints_the_checkpoint_only_with_a_vault() {
    let sandbox = Sandbox::new();

    let output = run(&mut hook(&sandbox, "checkpoint", &sandbox.path("")));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(
        stdout
            .starts_with("MEMORY CHECKPOINT — did anything this session become a durable fact?\n")
    );
    assert!(stdout.ends_with("Nothing durable? Ignore this and carry on.\n"));

    let mut without_vault = hook(&sandbox, "checkpoint", &sandbox.path(""));
    without_vault.env_remove("TIGHTSHIP_VAULT");
    assert_eq!(stdout_of(&run(&mut without_vault)), "");
}

#[cfg(unix)]
#[test]
fn an_unreadable_vault_or_note_or_data_folder_fails_with_exit_1() {
    use std::os::unix::fs::PermissionsExt;

    let sandbox = Sandbox::new();
    let unreadable = fs::Permissions::from_mode(0o000);
    let readable = fs::Permissions::from_mode(0o755);
    let working_directory = project(&sandbox, "tightship");
    let vault = sandbox.path("vault");
    let secret = note(&sandbox, "tightship", "mem-0001", "", "Hidden.");

    fs::set_permissions(&vault, unreadable.clone()).expect("an unreadable vault");
    let output = run(&mut hook(&sandbox, "start", &working_directory));
    fs::set_permissions(&vault, readable).expect("a readable vault");
    assert_fails_with(
        &output,
        &format!(
            "error: cannot read the vault folder {}: Permission denied (os error 13)",
            vault.display()
        ),
    );

    fs::set_permissions(&secret, unreadable).expect("an unreadable note");
    assert_fails_with(
        &run(&mut hook(&sandbox, "start", &working_directory)),
        "error: cannot read notes/tightship/mem-0001-a-note.md: Permission denied (os error 13)",
    );

    sandbox.write("data", "a file, not a folder");
    assert_fails_with(
        &with_stdin(
            &mut hook(&sandbox, "prompt", &sandbox.path("")),
            r#"{"session_id":"abc"}"#,
        ),
        &format!(
            "error: cannot write the checkpoint counter {}: Not a directory (os error 20)",
            sandbox.path("data/checkpoints/abc").display()
        ),
    );
}
