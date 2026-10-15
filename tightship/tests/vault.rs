mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use common::{Sandbox, assert_fails_with, assert_succeeds, run, stderr_of, stdout_of};

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

fn executable(file: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(file, fs::Permissions::from_mode(0o755)).expect("an executable");
    }
}
