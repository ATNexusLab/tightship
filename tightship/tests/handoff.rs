#[allow(dead_code)]
mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use common::{
    Sandbox, assert_fails_with, assert_succeeds, commit, git, note, run, stderr_of, stdout_of,
    with_stdin,
};

const BODY: &str = "\
**Goal:** close the spec of epic #6, issue #6
**Done:** the assumptions are confirmed and the draft is written
**Next step:** confirm the table of proposed values, then run the closed-spec gate
**Open decisions:** none
**Repo state:** branch `feat/6-handoff` · clean · no unpushed commit";

fn project(sandbox: &Sandbox) -> PathBuf {
    let folder = sandbox.path("tightship");
    fs::create_dir_all(&folder).expect("the project folder");
    git(sandbox, &folder, &["init", "--quiet", "-b", "main"]);
    folder
}

fn handoff_command(sandbox: &Sandbox, project: &Path, arguments: &[&str]) -> Command {
    let mut command = sandbox.tightship(&[&["vault", "handoff"], arguments].concat());
    command
        .current_dir(project)
        .env("TIGHTSHIP_VAULT", sandbox.path("vault"));
    command
}

fn release(sandbox: &Sandbox, project: &Path, title: &str, body: &str) -> Output {
    with_stdin(
        &mut handoff_command(sandbox, project, &["release", "--title", title]),
        body,
    )
}

fn claim(sandbox: &Sandbox, project: &Path) -> Output {
    run(&mut handoff_command(sandbox, project, &["claim"]))
}

fn handoff(sandbox: &Sandbox, scope: &str, stem: &str, created: &str, updated: &str) -> PathBuf {
    let id = &stem[..8];
    sandbox.write(
        &format!("vault/notes/{scope}/{stem}.md"),
        &format!(
            "---\nid: {id}\ntype: handoff\nproject: {scope}\nwhy: without it the next session starts blind\nsource: session\ncreated: {created}\nupdated: {updated}\n---\n\n# Work on {id}\n\n**Next step:** {id}\n"
        ),
    )
}

fn files_under(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(folder).expect("a readable folder") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files.sort();
    files
}

#[test]
fn release_replaces_the_scope_handoff() {
    let sandbox = Sandbox::new();
    let project = project(&sandbox);
    note(&sandbox, "all", "mem-0640", "", "A fact.");
    handoff(
        &sandbox,
        "tightship",
        "mem-0630-old",
        "2026-10-01",
        "2026-10-01",
    );
    let other = handoff(&sandbox, "other", "mem-0620-x", "2026-10-01", "2026-10-01");
    let other_text = fs::read_to_string(&other).expect("the other scope's handoff");

    let output = release(
        &sandbox,
        &project,
        "Handoff spec drafted, waiting for the values table",
        BODY,
    );

    assert_succeeds(&output);
    let written = "notes/tightship/mem-0641-handoff-spec-drafted-waiting-for-the-values-table.md";
    assert_eq!(
        stdout_of(&output),
        format!("{written}\nreplaced notes/tightship/mem-0630-old.md\n")
    );
    assert_eq!(
        files_under(&sandbox.path("vault/notes/tightship")),
        [sandbox.path("vault").join(written)]
    );
    let text = fs::read_to_string(sandbox.path("vault").join(written)).expect("the handoff");
    let front_matter: Vec<&str> = text.lines().take(9).collect();
    let created = front_matter[6].strip_prefix("created: ").expect("created");
    assert_eq!(
        front_matter,
        [
            "---",
            "id: mem-0641",
            "type: handoff",
            "project: tightship",
            "why: without it the next session, on another machine or harness, starts blind on work already in flight",
            "source: session",
            &format!("created: {created}"),
            &format!("updated: {created}"),
            "---",
        ]
    );
    assert!(
        text.ends_with(&format!(
            "\n\n# Handoff spec drafted, waiting for the values table\n\n{BODY}\n"
        )),
        "{text}"
    );
    assert_eq!(
        fs::read_to_string(&other).expect("the other scope's handoff"),
        other_text
    );
    let lint = run(sandbox
        .tightship(&["vault", "lint"])
        .env("TIGHTSHIP_VAULT", sandbox.path("vault")));
    assert_succeeds(&lint);
}

#[test]
fn release_leaves_the_project_repository_as_it_was() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.path("vault")).expect("the vault");
    git(
        &sandbox,
        &sandbox.path("."),
        &["init", "--quiet", "--bare", "-b", "main", "remote.git"],
    );
    git(
        &sandbox,
        &sandbox.path("."),
        &["clone", "--quiet", "remote.git", "tightship"],
    );
    let project = sandbox.path("tightship");
    git(
        &sandbox,
        &project,
        &["symbolic-ref", "HEAD", "refs/heads/main"],
    );
    sandbox.write("tightship/README.md", "first\n");
    commit(&sandbox, &project, "start");
    git(
        &sandbox,
        &project,
        &["push", "--quiet", "-u", "origin", "main"],
    );
    sandbox.write("tightship/README.md", "stashed\n");
    git(&sandbox, &project, &["stash", "--quiet"]);
    sandbox.write("tightship/README.md", "second\n");
    commit(&sandbox, &project, "not pushed");
    sandbox.write("tightship/README.md", "dirty\n");
    let state = || {
        [
            &["rev-parse", "HEAD"][..],
            &["for-each-ref"],
            &["stash", "list"],
            &["status", "--porcelain"],
        ]
        .map(|arguments| git(&sandbox, &project, arguments))
    };
    let before = state();

    assert_succeeds(&release(&sandbox, &project, "Work in flight", BODY));
    assert_eq!(state(), before);
}

#[test]
fn a_body_without_its_five_parts_writes_nothing() {
    let sandbox = Sandbox::new();
    let project = project(&sandbox);
    let old = handoff(
        &sandbox,
        "tightship",
        "mem-0630-old",
        "2026-10-01",
        "2026-10-01",
    );
    let body =
        "**Goal:** a goal\n**Next step:** a step\n**Open decisions:** none\nNo **Done:** here";

    assert_fails_with(
        &release(&sandbox, &project, "Work in flight", body),
        "error: the handoff's body has no **Done:**, **Repo state:**; each of **Goal:**, **Done:**, **Next step:**, **Open decisions:**, and **Repo state:** starts a line of it",
    );
    assert_eq!(files_under(&sandbox.path("vault/notes")), [old]);
}

#[test]
fn a_bad_title_or_no_body_writes_nothing() {
    let sandbox = Sandbox::new();
    let project = project(&sandbox);
    let old = handoff(
        &sandbox,
        "tightship",
        "mem-0630-old",
        "2026-10-01",
        "2026-10-01",
    );
    let long_title = "x".repeat(121);
    let no_stdin = run(
        handoff_command(&sandbox, &project, &["release", "--title", "Work"]).stdin(Stdio::null()),
    );

    for (output, message) in [
        (
            release(&sandbox, &project, &long_title, BODY),
            "error: --title must be 1 to 120 characters",
        ),
        (
            release(&sandbox, &project, "   ", BODY),
            "error: --title must be 1 to 120 characters",
        ),
        (no_stdin, "error: the note's body must arrive on stdin"),
    ] {
        assert_fails_with(&output, message);
        assert_eq!(
            files_under(&sandbox.path("vault/notes")),
            std::slice::from_ref(&old)
        );
    }
}

#[test]
fn claim_prints_the_handoff_and_removes_it() {
    let sandbox = Sandbox::new();
    let project = project(&sandbox);
    let path = handoff(
        &sandbox,
        "tightship",
        "mem-0641-a",
        "2026-10-14",
        "2026-10-15",
    );

    let output = claim(&sandbox, &project);

    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "# Handoff — tightship · mem-0641 · 2026-10-14 → 2026-10-15\n\n# Work on mem-0641\n\n**Next step:** mem-0641\n"
    );
    assert!(!path.exists());
    assert_eq!(
        stdout_of(&claim(&sandbox, &project)),
        "no handoff for tightship\n"
    );
}

#[test]
fn claim_with_no_handoff_changes_nothing() {
    let sandbox = Sandbox::new();
    let project = project(&sandbox);
    let vault = sandbox.path("vault");
    note(&sandbox, "tightship", "mem-0001", "", "A fact.");
    git(&sandbox, &vault, &["init", "--quiet", "-b", "main"]);
    commit(&sandbox, &vault, "vault: start");

    let output = claim(&sandbox, &project);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "no handoff for tightship\n");
    assert_eq!(git(&sandbox, &vault, &["status", "--porcelain"]), "");
}

#[test]
fn claim_prints_every_handoff_latest_first_and_removes_them() {
    let sandbox = Sandbox::new();
    let project = project(&sandbox);
    let older = handoff(
        &sandbox,
        "tightship",
        "mem-0641-a",
        "2026-10-14",
        "2026-10-14",
    );
    let newer = handoff(
        &sandbox,
        "tightship",
        "mem-0642-b",
        "2026-10-15",
        "2026-10-15",
    );

    let output = claim(&sandbox, &project);

    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "# Handoff — tightship · mem-0642 · 2026-10-15\n\n# Work on mem-0642\n\n**Next step:** mem-0642\n\n\
         # Handoff — tightship · mem-0641 · 2026-10-14\n\n# Work on mem-0641\n\n**Next step:** mem-0641\n"
    );
    assert!(!older.exists());
    assert!(!newer.exists());
}

#[test]
fn claim_reports_a_handoff_it_cannot_remove_after_printing_it() {
    let sandbox = Sandbox::new();
    let project = project(&sandbox);
    handoff(
        &sandbox,
        "tightship",
        "mem-0641-a",
        "2026-10-14",
        "2026-10-14",
    );
    let scope = sandbox.path("vault/notes/tightship");
    fs::set_permissions(&scope, fs::Permissions::from_mode(0o555)).expect("a read-only scope");

    let output = claim(&sandbox, &project);

    fs::set_permissions(&scope, fs::Permissions::from_mode(0o755)).expect("a writable scope");
    assert!(
        stdout_of(&output).starts_with("# Handoff — tightship · mem-0641 · 2026-10-14\n"),
        "{}",
        stdout_of(&output)
    );
    assert_fails_with(
        &output,
        "error: cannot remove notes/tightship/mem-0641-a.md: Permission denied (os error 13)",
    );
}

#[test]
fn no_vault_and_a_wrong_subcommand_fail_with_exit_1() {
    let sandbox = Sandbox::new();
    let project = project(&sandbox);

    let mut no_vault = handoff_command(&sandbox, &project, &["claim"]);
    no_vault.env_remove("TIGHTSHIP_VAULT");
    assert_fails_with(
        &run(&mut no_vault),
        "error: no vault is set for this session; turn memory on, set the plugin's vault folder, and start a new session",
    );

    let wrong = run(&mut handoff_command(&sandbox, &project, &["fly"]));
    assert_fails_with(&wrong, "error: unknown subcommand fly");
    assert!(
        stderr_of(&wrong).contains("fly\n\nUsage: tightship vault path\n"),
        "{}",
        stderr_of(&wrong)
    );
}
