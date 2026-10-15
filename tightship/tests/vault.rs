mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use common::{
    Sandbox, assert_fails_with, assert_succeeds, note, run, stderr_of, stdout_of, with_stdin,
};

const TEMPLATE: &str = include_str!("../../content/vault-template/AGENTS.md");

#[test]
fn init_creates_a_vault_from_the_template() {
    let sandbox = Sandbox::new();
    let vault = sandbox.path("vault");
    let folder = vault.to_str().expect("a UTF-8 path");

    let output = run(&mut sandbox.tightship(&["vault", "init", folder]));

    assert_succeeds(&output);
    assert!(stdout_of(&output).starts_with(&format!("Created the vault at {folder}. Commit it,")));
    assert_eq!(
        fs::read_to_string(vault.join("AGENTS.md")).expect("AGENTS.md"),
        TEMPLATE
    );
    let notes = fs::read_dir(vault.join("notes/all")).expect("notes/all/");
    assert_eq!(notes.count(), 0);
    assert!(vault.join(".git").is_dir());
}

#[test]
fn init_into_a_folder_that_holds_files_changes_nothing() {
    let sandbox = Sandbox::new();
    let rules = sandbox.write("vault/AGENTS.md", "my own rules\n");
    let folder = sandbox.path("vault");
    let folder = folder.to_str().expect("a UTF-8 path");

    assert_fails_with(
        &run(&mut sandbox.tightship(&["vault", "init", folder])),
        &format!("error: {folder} already exists and is not an empty folder; nothing changed"),
    );
    assert_eq!(
        fs::read_to_string(rules).expect("AGENTS.md"),
        "my own rules\n"
    );
    assert!(!sandbox.path("vault/.git").exists());
}

#[test]
fn init_without_git_creates_nothing() {
    let sandbox = Sandbox::new();
    fs::create_dir(sandbox.path("empty-path")).expect("a folder with no git");
    let folder = sandbox.path("vault");

    let mut command = sandbox.tightship(&["vault", "init", folder.to_str().expect("a UTF-8 path")]);
    command.env("PATH", sandbox.path("empty-path"));

    assert_fails_with(
        &run(&mut command),
        "error: git is not installed; a vault is a git repository, so nothing was created",
    );
    assert!(!folder.exists());
}

#[test]
fn init_reports_a_failing_git() {
    let sandbox = Sandbox::new();
    let git = sandbox.write(
        "failing-git/git",
        "#!/bin/sh\necho 'fatal: no space left' >&2\nexit 1\n",
    );
    executable(&git);
    let folder = sandbox.path("vault");
    let folder = folder.to_str().expect("a UTF-8 path");

    let mut command = sandbox.tightship(&["vault", "init", folder]);
    command.env("PATH", sandbox.path("failing-git"));

    assert_fails_with(
        &run(&mut command),
        &format!("error: git init failed in {folder}: fatal: no space left"),
    );
}

#[test]
fn path_prints_the_folder_the_adopter_set() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.path("elsewhere")).expect("the adopter's vault");
    fs::create_dir_all(sandbox.path("home/ai-memory")).expect("the baseline's default vault");
    let elsewhere = sandbox.path("elsewhere");

    let mut command = sandbox.tightship(&["vault", "path"]);
    command
        .env("TIGHTSHIP_VAULT", &elsewhere)
        .env("HOME", sandbox.path("home"));
    let output = run(&mut command);

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), format!("{}\n", elsewhere.display()));
}

#[test]
fn an_unusable_vault_folder_fails() {
    let sandbox = Sandbox::new();
    let gone = sandbox.path("gone");
    let gone = gone.to_str().expect("a UTF-8 path");
    let cases = [
        (
            None,
            "error: no vault is set for this session; turn memory on, set the plugin's vault folder, and start a new session".to_owned(),
        ),
        (
            Some("vault"),
            "error: the vault folder vault is not an absolute path".to_owned(),
        ),
        (
            Some(gone),
            format!("error: the vault folder {gone} does not exist; create it with `tightship vault init {gone}`"),
        ),
    ];

    for (vault, line) in cases {
        let mut command = sandbox.tightship(&["vault", "path"]);
        if let Some(vault) = vault {
            command.env("TIGHTSHIP_VAULT", vault);
        }
        assert_fails_with(&run(&mut command), &line);
    }
}

#[test]
fn a_usage_error_exits_1_with_the_usage_and_help_exits_0() {
    let sandbox = Sandbox::new();
    let usage = "Usage: tightship vault path";

    let output = run(&mut sandbox.tightship(&["vault", "fly"]));
    assert_fails_with(&output, "error: unknown subcommand fly");
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains(&format!("error: unknown subcommand fly\n\n{usage}\n")),
        "{stderr}"
    );

    for arguments in [
        &["vault", "path", "--fast"][..],
        &["vault", "init"],
        &["vault", "new", "--title"],
        &["vault"],
        &[],
    ] {
        let output = run(&mut sandbox.tightship(arguments));
        assert_eq!(
            output.status.code(),
            Some(1),
            "{arguments:?}: {}",
            stderr_of(&output)
        );
        assert!(stderr_of(&output).contains(usage), "{arguments:?}");
    }

    assert_fails_with(
        &run(&mut sandbox.tightship(&["vault", "new", "--title"])),
        "error: --title needs a value",
    );

    let help = run(&mut sandbox.tightship(&["--help"]));
    assert_succeeds(&help);
    assert!(stdout_of(&help).starts_with(usage));
}

#[test]
fn the_launcher_without_a_binary_for_the_machine_fails() {
    let sandbox = Sandbox::new();
    let launcher = sandbox.write("bin/tightship", include_str!("../launcher.sh"));
    executable(&launcher);
    let uname = |flag| {
        let output = Command::new("uname")
            .arg(flag)
            .output()
            .expect("uname runs");
        stdout_of(&output).trim().to_owned()
    };

    let output = run(Command::new("sh").arg(&launcher));

    assert_fails_with(
        &output,
        &format!(
            "error: tightship has no binary for {} {}",
            uname("-s"),
            uname("-m")
        ),
    );
}

const NOTE_BODY: &str = "A plugin's SessionStart hook can export variables for the Bash tool.\n";

fn in_vault(sandbox: &Sandbox, arguments: &[&str]) -> Command {
    let project = sandbox.path("tightship");
    fs::create_dir_all(&project).expect("the project folder");
    let mut command = sandbox.tightship(arguments);
    command
        .current_dir(project)
        .env("TIGHTSHIP_VAULT", sandbox.path("vault"));
    command
}

fn new_note(sandbox: &Sandbox, replaced: (&str, &str)) -> Command {
    let mut arguments = vec![
        "vault",
        "new",
        "--scope",
        "tightship",
        "--type",
        "gotcha",
        "--title",
        "Hooks get CLAUDE_ENV_FILE",
        "--why",
        "without it a plugin's bin cannot find a value only hooks receive",
        "--tags",
        "hooks,env",
        "--source",
        "session",
    ];
    let (flag, value) = replaced;
    match arguments.iter().position(|argument| *argument == flag) {
        Some(index) => arguments[index + 1] = value,
        None => arguments.extend([flag, value]),
    }
    in_vault(sandbox, &arguments)
}

fn files_of(folder: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    for entry in fs::read_dir(folder).expect("the folder") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            files.extend(files_of(&path));
        } else {
            let text = fs::read_to_string(&path).expect("the file");
            files.push((path, text));
        }
    }
    files.sort();
    files
}

#[test]
fn search_lists_the_project_scope_first() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "vault/notes/all/mem-0001-the-launcher.md",
        "---\nid: mem-0001\ntype: gotcha\nproject: all\nwhy: a machine without uname gets no binary at all\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-01\n---\n\n# The launcher picks the launcher's binary\n",
    );
    note(&sandbox, "tightship", "mem-0002", "", "The launcher is sh.");
    note(&sandbox, "other", "mem-0003", "", "Another launcher.");

    let output = run(&mut in_vault(
        &sandbox,
        &["vault", "search", "launcher", "uname"],
    ));

    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        format!(
            "2 of 2 matching notes, paths under `{}/`:\n\
             - notes/tightship/mem-0002-a-note.md — the mem-0002 why costs real time\n\
             - notes/all/mem-0001-the-launcher.md — a machine without uname gets no binary at all\n",
            sandbox.path("vault").display()
        )
    );
}

#[test]
fn search_with_no_match_says_so_and_exits_0() {
    let sandbox = Sandbox::new();
    note(&sandbox, "tightship", "mem-0001", "", "The launcher is sh.");

    let output = run(&mut in_vault(&sandbox, &["vault", "search", "zeppelin"]));

    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "no note matches zeppelin — try other terms, or add --everywhere for the other scopes\n"
    );
}

#[test]
fn new_writes_the_note_and_rejects_a_short_why_a_bad_scope_and_no_body() {
    let sandbox = Sandbox::new();
    note(&sandbox, "all", "mem-0631", "", "The highest id.");
    let today = jiff::Zoned::now().date();

    let output = with_stdin(&mut new_note(&sandbox, ("--source", "session")), NOTE_BODY);

    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "notes/tightship/mem-0632-hooks-get-claude-env-file.md\n"
    );
    let written = sandbox.path("vault/notes/tightship/mem-0632-hooks-get-claude-env-file.md");
    assert_eq!(
        fs::read_to_string(written).expect("the note"),
        format!(
            "---\nid: mem-0632\ntype: gotcha\nproject: tightship\nwhy: without it a plugin's bin cannot find a value only hooks receive\ntags: [hooks, env]\nsource: session\ncreated: {today}\nupdated: {today}\n---\n\n# Hooks get CLAUDE_ENV_FILE\n\n{NOTE_BODY}"
        )
    );

    let folded = with_stdin(
        &mut new_note(&sandbox, ("--why", "a line that forges\npinned: true")),
        NOTE_BODY,
    );
    assert_succeeds(&folded);
    let folded_note = sandbox.path("vault/notes/tightship/mem-0633-hooks-get-claude-env-file.md");
    let folded_text = fs::read_to_string(folded_note).expect("the folded note");
    assert!(
        folded_text.contains("\nwhy: a line that forges pinned: true\n"),
        "{folded_text}"
    );
    assert!(!folded_text.contains("\npinned: true\n"), "{folded_text}");

    let before = files_of(&sandbox.path("vault"));
    assert_fails_with(
        &with_stdin(&mut new_note(&sandbox, ("--why", "about hooks")), NOTE_BODY),
        "error: --why must state in a full line what not knowing this would cost later, not name a topic (got 2 words)",
    );
    assert_fails_with(
        &with_stdin(&mut new_note(&sandbox, ("--scope", "../x")), NOTE_BODY),
        "error: --scope must be 1 to 100 characters, with no / or \\ and no leading dot",
    );
    assert_fails_with(
        &run(new_note(&sandbox, ("--source", "session")).stdin(Stdio::null())),
        "error: the note's body must arrive on stdin",
    );
    assert_eq!(files_of(&sandbox.path("vault")), before);
}

#[test]
fn lint_passes_a_clean_vault_and_names_each_problem() {
    let sandbox = Sandbox::new();
    note(&sandbox, "all", "mem-0001", "", "See [[mem-0002-a-note]].");
    note(
        &sandbox,
        "tightship",
        "mem-0002",
        "",
        "Write `[[a-link]]` like this.",
    );

    let clean = run(&mut in_vault(&sandbox, &["vault", "lint"]));

    assert_succeeds(&clean);
    assert_eq!(
        stdout_of(&clean),
        "2 notes, 0 with problems, 0 scopes over the session start\n"
    );

    sandbox.write(
        "vault/notes/tightship/mem-0003-no-why.md",
        "---\nid: mem-0003\ntype: gotcha\nproject: tightship\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-01\n---\n\n# No why\n",
    );
    note(&sandbox, "all", "mem-0007", "", "First.");
    note(&sandbox, "tightship", "mem-0007", "", "Second.");

    let broken = run(&mut in_vault(&sandbox, &["vault", "lint"]));

    assert_eq!(broken.status.code(), Some(1));
    assert_eq!(stderr_of(&broken), "");
    assert_eq!(
        stdout_of(&broken),
        "notes/tightship/mem-0003-no-why.md\n  - missing `why`\n\
         notes/tightship/mem-0007-a-note.md\n  - id `mem-0007` already used by mem-0007-a-note.md\n\
         5 notes, 2 with problems, 0 scopes over the session start\n"
    );
}

#[test]
fn a_bad_argument_of_new_or_search_writes_nothing() {
    let sandbox = Sandbox::new();
    note(&sandbox, "all", "mem-0631", "", "The highest id.");
    sandbox.write(
        "vault/notes/tightship/mem-0632-hooks-get-claude-env-file.md",
        "Not a note yet.\n",
    );
    let before = files_of(&sandbox.path("vault"));
    let long_title = "t".repeat(121);

    for (mut command, message) in [
        (
            in_vault(&sandbox, &["vault", "search", "  "]),
            "search needs at least one term",
        ),
        (
            new_note(&sandbox, ("--type", "handoff")),
            "--type must be one of preference, decision, gotcha, project",
        ),
        (
            new_note(&sandbox, ("--title", &long_title)),
            "--title must be 1 to 120 characters",
        ),
        (
            new_note(&sandbox, ("--source", "chat")),
            "--source must be one of user, session, repo",
        ),
        (
            new_note(&sandbox, ("--created", "15/10/2026")),
            "--created must be a date as YYYY-MM-DD",
        ),
        (
            new_note(&sandbox, ("--source", "session")),
            "notes/tightship/mem-0632-hooks-get-claude-env-file.md already exists",
        ),
    ] {
        assert_fails_with(
            &with_stdin(&mut command, NOTE_BODY),
            &format!("error: {message}"),
        );
    }
    assert_eq!(files_of(&sandbox.path("vault")), before);
}

fn executable(file: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(file, fs::Permissions::from_mode(0o755)).expect("an executable");
    }
}
