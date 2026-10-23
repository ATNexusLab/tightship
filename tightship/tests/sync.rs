#[allow(dead_code)]
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use jiff::ToSpan;
use jiff::civil::Date;

use common::{
    IDENTITY, Sandbox, assert_fails_with, assert_succeeds, commit, git, run, stderr_of, stdout_of,
};

fn sync(sandbox: &Sandbox, vault: &Path) -> Output {
    run(sandbox
        .tightship(&["vault", "sync"])
        .env("TIGHTSHIP_VAULT", vault)
        .envs(IDENTITY))
}

fn note_text(scope: &str, id: &str, body: &str) -> String {
    format!(
        "---\nid: {id}\ntype: decision\nproject: {scope}\nwhy: the {id} why costs real time\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-01\n---\n\n# A note\n\n{body}\n"
    )
}

fn write_dated(sandbox: &Sandbox, clone: &str, stem: &str, front: &str, body: &str) -> PathBuf {
    let id = &stem[..8];
    sandbox.write(
        &format!("{clone}/notes/all/{stem}.md"),
        &format!(
            "---\nid: {id}\nproject: all\nwhy: the {id} why costs real time\n{front}---\n\n{body}\n"
        ),
    )
}

fn write_note(sandbox: &Sandbox, clone: &str, stem: &str, body: &str) -> PathBuf {
    write_scoped(sandbox, clone, "all", stem, body)
}

fn write_scoped(sandbox: &Sandbox, clone: &str, scope: &str, stem: &str, body: &str) -> PathBuf {
    let id = &stem[..8];
    sandbox.write(
        &format!("{clone}/notes/{scope}/{stem}.md"),
        &note_text(scope, id, body),
    )
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
    git(
        sandbox,
        remote,
        &[
            "-c",
            "core.quotePath=false",
            "ls-tree",
            "-r",
            "--name-only",
            "main",
        ],
    )
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
        "> Recording local changes\n> Fetching the remote\n> Rebasing local commits on the remote\n> Pushing\nsynced: 1 pushed, 1 pulled, 0 expired\n"
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
fn a_vault_with_no_remote_is_committed_kept_local_and_expired() {
    let sandbox = Sandbox::new();
    let vault = sandbox.path("vault");
    git(
        &sandbox,
        &sandbox.path("."),
        &["init", "--quiet", "-b", "main", "vault"],
    );
    let old = "created: 2025-01-01\nupdated: 2025-01-01\n";
    let kept = [
        (
            "mem-0001-user",
            format!("type: gotcha\nsource: user\n{old}"),
        ),
        (
            "mem-0002-pinned",
            format!("type: project\npinned: true\nsource: session\n{old}"),
        ),
        (
            "mem-0003-decision",
            format!("type: decision\nsource: session\n{old}"),
        ),
        (
            "mem-0004-preference",
            format!("type: preference\nsource: session\n{old}"),
        ),
        (
            "mem-0005-handoff",
            format!("type: handoff\nsource: session\n{old}"),
        ),
        (
            "mem-0006-soon",
            "type: gotcha\nsource: session\ncreated: 2025-01-01\nupdated: soon\n".to_owned(),
        ),
    ];
    for (stem, front) in &kept {
        write_dated(&sandbox, "vault", stem, front, "# A note\n\nOne fact.");
    }
    write_dated(
        &sandbox,
        "vault",
        "mem-0007-old",
        &format!("type: gotcha\nsource: session\n{old}"),
        "# A note\n\nOne fact.",
    );

    let synced = sync(&sandbox, &vault);

    assert_succeeds(&synced);
    assert_eq!(
        stdout_of(&synced),
        "> Recording local changes\n> Removing expired notes\nexpired notes/all/mem-0007-old.md (updated 2025-01-01)\n\
         kept local — the vault has no git remote; 1 expired\n"
    );
    assert_eq!(kept.len(), 6);
    for (stem, _) in &kept {
        assert!(
            vault.join(format!("notes/all/{stem}.md")).is_file(),
            "{stem}"
        );
    }
    assert!(!vault.join("notes/all/mem-0007-old.md").exists());
    assert_eq!(
        git(&sandbox, &vault, &["log", "--format=%s"]),
        "vault: expire notes\nvault: sync"
    );
    assert_eq!(git(&sandbox, &vault, &["status", "--porcelain"]), "");
}

#[test]
fn an_expired_note_and_its_links_leave_the_vault_and_the_remote() {
    let sandbox = Sandbox::new();
    let (remote, a, _) = machines(&sandbox, "mem-0001-x");
    let today = jiff::Zoned::now().date();
    let expired_on = today - 31.days();
    let session = |updated: Date| {
        format!("type: gotcha\nsource: session\ncreated: {updated}\nupdated: {updated}\n")
    };
    write_dated(
        &sandbox,
        "a",
        "mem-0002-a",
        &session(expired_on),
        "# Cargo test flakes on Windows\n\nRetry once.",
    );
    let linker = write_dated(
        &sandbox,
        "a",
        "mem-0003-links",
        &session(today),
        "# Links\n\nSee [[mem-0002-a]], [[mem-0002-a#fix]], and [[mem-0002-a|the flake]].",
    );
    write_dated(
        &sandbox,
        "a",
        "mem-0004-thirty",
        &session(today - 30.days()),
        "# A note\n\nStill live.",
    );
    write_dated(
        &sandbox,
        "a",
        "mem-0005-revised",
        &format!(
            "type: project\nsource: repo\ncreated: {}\nupdated: {}\n",
            today - 83.days(),
            today - 12.days()
        ),
        "# A note\n\nRevised in place.",
    );
    commit(&sandbox, &a, "vault: a");
    git(&sandbox, &a, &["push", "--quiet"]);
    let linked = fs::read_to_string(&linker).expect("the linking note");

    let synced = sync(&sandbox, &a);

    assert_succeeds(&synced);
    assert_eq!(
        stdout_of(&synced),
        format!(
            "> Recording local changes\n> Fetching the remote\n> Rebasing local commits on the remote\n\
             > Removing expired notes\nexpired notes/all/mem-0002-a.md (updated {expired_on})\n\
             > Pushing\nsynced: 1 pushed, 0 pulled, 1 expired\n"
        )
    );
    assert_eq!(
        remote_files(&sandbox, &remote),
        "notes/all/mem-0001-x.md\nnotes/all/mem-0003-links.md\nnotes/all/mem-0004-thirty.md\nnotes/all/mem-0005-revised.md"
    );
    assert_eq!(
        git(
            &sandbox,
            &remote,
            &["log", "-1", "--format=%s", "--name-status", "main"]
        ),
        "vault: expire notes\n\nD\tnotes/all/mem-0002-a.md\nM\tnotes/all/mem-0003-links.md"
    );
    assert_eq!(
        fs::read_to_string(&linker).expect("the linking note"),
        linked.replace(
            "[[mem-0002-a]], [[mem-0002-a#fix]], and [[mem-0002-a|the flake]]",
            "Cargo test flakes on Windows, Cargo test flakes on Windows, and the flake"
        )
    );
    assert_succeeds(&run(sandbox
        .tightship(&["vault", "lint"])
        .env("TIGHTSHIP_VAULT", &a)));
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

    let resolution = note_text("all", "mem-0001", "A's fact. B's fact.");
    fs::write(&conflicting, &resolution).expect("the resolution");
    git(&sandbox, &a, &["add", "notes/all/mem-0001-x.md"]);
    git(
        &sandbox,
        &a,
        &["-c", "core.editor=true", "rebase", "--continue"],
    );

    let resumed = sync(&sandbox, &a);

    assert_succeeds(&resumed);
    assert!(stdout_of(&resumed).ends_with("synced: 1 pushed, 0 pulled, 0 expired\n"));
    assert_eq!(
        git(&sandbox, &remote, &["show", "main:notes/all/mem-0001-x.md"]),
        resolution.trim()
    );
}

#[test]
fn an_id_taken_on_both_machines_or_twice_locally_is_renumbered_with_its_links() {
    let sandbox = Sandbox::new();
    let (remote, a, b) = machines(&sandbox, "mem-0631-base");
    write_scoped(&sandbox, "b", "orçamento", "mem-0632-from-b", "From B.");
    commit(&sandbox, &b, "vault: b");
    git(&sandbox, &b, &["push", "--quiet"]);
    write_scoped(&sandbox, "a", "orçamento", "mem-0632-from-a", "From A.");
    write_note(&sandbox, "a", "mem-0633-links", "See [[mem-0632-from-a]].");
    write_note(&sandbox, "a", "mem-0635-first", "Written first.");
    write_scoped(
        &sandbox,
        "a",
        "orçamento",
        "mem-0635-second",
        "Written at once.",
    );
    commit(&sandbox, &a, "vault: a");

    let synced = sync(&sandbox, &a);

    assert_succeeds(&synced);
    assert_eq!(
        stdout_of(&synced),
        "> Recording local changes\n> Fetching the remote\n> Rebasing local commits on the remote\n> Renumbering colliding ids\n\
         mem-0632 → mem-0636 notes/orçamento/mem-0636-from-a.md\n\
         mem-0635 → mem-0637 notes/orçamento/mem-0637-second.md\n\
         > Pushing\nsynced: 2 pushed, 1 pulled, 0 expired\n"
    );
    let renamed =
        fs::read_to_string(a.join("notes/orçamento/mem-0636-from-a.md")).expect("the renamed note");
    assert!(renamed.lines().any(|line| line == "id: mem-0636"));
    let links =
        fs::read_to_string(a.join("notes/all/mem-0633-links.md")).expect("the linking note");
    assert!(links.contains("[[mem-0636-from-a]]"));
    assert_succeeds(&run(sandbox
        .tightship(&["vault", "lint"])
        .env("TIGHTSHIP_VAULT", &a)));
    assert_eq!(
        remote_files(&sandbox, &remote),
        "notes/all/mem-0631-base.md\nnotes/all/mem-0633-links.md\nnotes/all/mem-0635-first.md\n\
         notes/orçamento/mem-0632-from-b.md\nnotes/orçamento/mem-0636-from-a.md\nnotes/orçamento/mem-0637-second.md"
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
