#[allow(dead_code)]
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{Sandbox, assert_fails_with, assert_succeeds, run, stderr_of, stdout_of};

const IDENTITY: [(&str, &str); 4] = [
    ("GIT_AUTHOR_NAME", "Ada Vault"),
    ("GIT_AUTHOR_EMAIL", "ada@example.test"),
    ("GIT_COMMITTER_NAME", "Ada Vault"),
    ("GIT_COMMITTER_EMAIL", "ada@example.test"),
];

fn git(sandbox: &Sandbox, folder: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(folder)
        .args(arguments)
        .env("GIT_CONFIG_GLOBAL", sandbox.write(".gitconfig", ""))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .envs(IDENTITY)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        stderr_of(&output)
    );
    stdout_of(&output).trim().to_owned()
}

fn sync(sandbox: &Sandbox, vault: &Path) -> Output {
    run(sandbox
        .tightship(&["vault", "sync"])
        .env("TIGHTSHIP_VAULT", vault)
        .envs(IDENTITY))
}

fn note_text(id: &str, body: &str) -> String {
    format!(
        "---\nid: {id}\ntype: gotcha\nproject: all\nwhy: the {id} why costs real time\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-01\n---\n\n# A note\n\n{body}\n"
    )
}

fn write_note(sandbox: &Sandbox, clone: &str, stem: &str, body: &str) -> PathBuf {
    let id = &stem[..8];
    sandbox.write(
        &format!("{clone}/notes/all/{stem}.md"),
        &note_text(id, body),
    )
}

fn commit(sandbox: &Sandbox, clone: &Path, message: &str) {
    git(sandbox, clone, &["add", "-A"]);
    git(sandbox, clone, &["commit", "--quiet", "-m", message]);
}

fn machines(sandbox: &Sandbox, first: &str) -> (PathBuf, PathBuf, PathBuf) {
    let remote = sandbox.path("remote.git");
    let (a, b) = (sandbox.path("a"), sandbox.path("b"));
    git(
        sandbox,
        &sandbox.path("."),
        &["init", "--quiet", "--bare", "-b", "main", "remote.git"],
    );
    git(
        sandbox,
        &sandbox.path("."),
        &["clone", "--quiet", "remote.git", "a"],
    );
    git(sandbox, &a, &["symbolic-ref", "HEAD", "refs/heads/main"]);
    write_note(sandbox, "a", first, "One fact.");
    commit(sandbox, &a, "vault: start");
    git(sandbox, &a, &["push", "--quiet", "-u", "origin", "main"]);
    git(
        sandbox,
        &sandbox.path("."),
        &["clone", "--quiet", "remote.git", "b"],
    );
    (remote, a, b)
}

fn remote_files(sandbox: &Sandbox, remote: &Path) -> String {
    git(sandbox, remote, &["ls-tree", "-r", "--name-only", "main"])
}

#[test]
fn sync_takes_the_remote_notes_and_pushes_the_local_ones() {
    let sandbox = Sandbox::new();
    let (remote, a, b) = machines(&sandbox, "mem-0001-x");
    write_note(&sandbox, "b", "mem-0002-b", "From B.");
    commit(&sandbox, &b, "vault: b");
    git(&sandbox, &b, &["push", "--quiet"]);
    write_note(&sandbox, "a", "mem-0003-a", "From A.");

    let synced = sync(&sandbox, &a);

    assert_succeeds(&synced);
    assert_eq!(
        stdout_of(&synced),
        "> Recording local changes\n> Fetching the remote\n> Rebasing local commits on the remote\n> Pushing\nsynced: 1 pushed, 1 pulled\n"
    );
    assert!(a.join("notes/all/mem-0002-b.md").is_file());
    assert_eq!(
        remote_files(&sandbox, &remote),
        "notes/all/mem-0001-x.md\nnotes/all/mem-0002-b.md\nnotes/all/mem-0003-a.md"
    );
    assert_eq!(
        git(&sandbox, &a, &["log", "-1", "--format=%s"]),
        "vault: sync"
    );
}

#[test]
fn a_vault_with_no_remote_is_committed_and_kept_local() {
    let sandbox = Sandbox::new();
    let vault = sandbox.path("vault");
    git(
        &sandbox,
        &sandbox.path("."),
        &["init", "--quiet", "-b", "main", "vault"],
    );
    write_note(&sandbox, "vault", "mem-0001-x", "One fact.");

    let synced = sync(&sandbox, &vault);

    assert_succeeds(&synced);
    assert_eq!(
        stdout_of(&synced),
        "> Recording local changes\nkept local — the vault has no git remote\n"
    );
    assert_eq!(
        git(&sandbox, &vault, &["log", "-1", "--format=%s"]),
        "vault: sync"
    );
    assert_eq!(git(&sandbox, &vault, &["status", "--porcelain"]), "");
}

#[test]
fn a_conflict_stops_in_the_rebase_and_the_resolution_is_pushed() {
    let sandbox = Sandbox::new();
    let (remote, a, b) = machines(&sandbox, "mem-0001-x");
    write_note(&sandbox, "b", "mem-0001-x", "B's fact.");
    commit(&sandbox, &b, "vault: b");
    git(&sandbox, &b, &["push", "--quiet"]);
    let conflicting = write_note(&sandbox, "a", "mem-0001-x", "A's fact.");
    let shown = a.display();

    let stopped = sync(&sandbox, &a);

    assert_fails_with(
        &stopped,
        &format!(
            "error: these notes conflict with the remote: notes/all/mem-0001-x.md; resolve each, run `git -C {shown} add` on it and `git -C {shown} rebase --continue`, then run `tightship vault sync` again"
        ),
    );
    assert_eq!(
        git(&sandbox, &remote, &["rev-parse", "main"]),
        git(&sandbox, &b, &["rev-parse", "HEAD"])
    );
    let held = fs::read_to_string(&conflicting).expect("the conflicting note");
    assert!(
        held.contains("A's fact.") && held.contains("B's fact."),
        "{held}"
    );

    assert_fails_with(
        &sync(&sandbox, &a),
        &format!(
            "error: a rebase or merge is under way in {shown}; finish it with `git -C {shown} rebase --continue` or `git -C {shown} merge --continue`, then run `tightship vault sync` again"
        ),
    );

    let resolution = note_text("mem-0001", "A's fact. B's fact.");
    fs::write(&conflicting, &resolution).expect("the resolution");
    git(&sandbox, &a, &["add", "notes/all/mem-0001-x.md"]);
    git(
        &sandbox,
        &a,
        &["-c", "core.editor=true", "rebase", "--continue"],
    );

    let resumed = sync(&sandbox, &a);

    assert_succeeds(&resumed);
    assert!(stdout_of(&resumed).ends_with("synced: 1 pushed, 0 pulled\n"));
    assert_eq!(
        git(&sandbox, &remote, &["show", "main:notes/all/mem-0001-x.md"]),
        resolution.trim()
    );
}

#[test]
fn an_id_taken_on_both_machines_is_renumbered_with_its_links() {
    let sandbox = Sandbox::new();
    let (remote, a, b) = machines(&sandbox, "mem-0631-base");
    write_note(&sandbox, "b", "mem-0632-from-b", "From B.");
    commit(&sandbox, &b, "vault: b");
    git(&sandbox, &b, &["push", "--quiet"]);
    write_note(&sandbox, "a", "mem-0632-from-a", "From A.");
    write_note(&sandbox, "a", "mem-0633-links", "See [[mem-0632-from-a]].");
    commit(&sandbox, &a, "vault: a");

    let synced = sync(&sandbox, &a);

    assert_succeeds(&synced);
    assert_eq!(
        stdout_of(&synced),
        "> Recording local changes\n> Fetching the remote\n> Rebasing local commits on the remote\n> Renumbering colliding ids\nmem-0632 → mem-0634 notes/all/mem-0634-from-a.md\n> Pushing\nsynced: 2 pushed, 1 pulled\n"
    );
    let renamed =
        fs::read_to_string(a.join("notes/all/mem-0634-from-a.md")).expect("the renamed note");
    assert!(renamed.lines().any(|line| line == "id: mem-0634"));
    let links =
        fs::read_to_string(a.join("notes/all/mem-0633-links.md")).expect("the linking note");
    assert!(links.contains("[[mem-0634-from-a]]"));
    assert_succeeds(&run(sandbox
        .tightship(&["vault", "lint"])
        .env("TIGHTSHIP_VAULT", &a)));
    assert_eq!(
        remote_files(&sandbox, &remote),
        "notes/all/mem-0631-base.md\nnotes/all/mem-0632-from-b.md\nnotes/all/mem-0633-links.md\nnotes/all/mem-0634-from-a.md"
    );
}

#[test]
fn no_upstream_and_a_remote_out_of_reach_fail_with_the_next_step() {
    let sandbox = Sandbox::new();
    let vault = sandbox.path("vault");
    git(
        &sandbox,
        &sandbox.path("."),
        &["init", "--quiet", "--bare", "-b", "main", "lone.git"],
    );
    git(
        &sandbox,
        &sandbox.path("."),
        &["init", "--quiet", "-b", "main", "vault"],
    );
    git(
        &sandbox,
        &vault,
        &["remote", "add", "origin", "../lone.git"],
    );
    write_note(&sandbox, "vault", "mem-0001-x", "One fact.");
    let shown = vault.display();

    assert_fails_with(
        &sync(&sandbox, &vault),
        &format!(
            "error: the vault's branch main tracks no remote branch; run `git -C {shown} push -u origin main` once, then run `tightship vault sync` again"
        ),
    );

    let (remote, a, _) = machines(&sandbox, "mem-0001-x");
    fs::remove_dir_all(&remote).expect("the remote removed");

    let unreachable = sync(&sandbox, &a);

    assert_eq!(unreachable.status.code(), Some(1));
    let prefix = format!("error: git fetch failed in {}: ", a.display());
    assert!(
        stderr_of(&unreachable).starts_with(&prefix),
        "{}",
        stderr_of(&unreachable)
    );
}
