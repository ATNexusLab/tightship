mod common;

use common::{Repository, assert_fails_with, assert_succeeds, stderr_of};

fn committed(files: &[(&str, String)]) -> Repository {
    let repository = Repository::with(files);
    repository.git(&["add", "--all"]);
    repository.git(&["commit", "--quiet", "--message", "Start"]);
    repository
}

#[cfg(unix)]
fn tools_with_git_only() -> tempfile::TempDir {
    use std::env;

    let path = env::var_os("PATH").expect("a PATH");
    let git = env::split_paths(&path)
        .map(|folder| folder.join("git"))
        .find(|candidate| candidate.is_file())
        .expect("git on the PATH");
    let tools = tempfile::tempdir().expect("a tools folder");
    std::os::unix::fs::symlink(git, tools.path().join("git")).expect("a link to git");
    tools
}

#[test]
fn a_clean_repository_passes() {
    let repository = committed(&[]);

    let output = repository.run("scan");
    assert_succeeds(&output);
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "No secret in 1 commits. No vault note."),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn a_secret_on_another_branch_fails_without_printing_it() {
    let repository = committed(&[]);
    let alphabet: Vec<char> = ('a'..='z').chain('0'..='9').collect();
    let shuffled: String = (0..alphabet.len())
        .map(|index| alphabet[(index * 7 + 3) % alphabet.len()])
        .collect();
    let leaked = format!("ghp_{shuffled}");
    repository.git(&["switch", "--quiet", "--create", "side"]);
    repository.write("deploy.env", &format!("GITHUB_TOKEN={leaked}\n"));
    repository.git(&["add", "--all"]);
    repository.git(&["commit", "--quiet", "--message", "Add the deploy settings"]);
    repository.git(&["switch", "--quiet", "-"]);

    let output = repository.run("scan");
    assert_fails_with(
        &output,
        "error: gitleaks found 1 secrets in the history; its redacted report is above",
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(&leaked));
    assert!(!stderr_of(&output).contains(&leaked));
}

#[cfg(unix)]
#[test]
fn a_missing_gitleaks_fails() {
    let repository = committed(&[]);
    let tools = tools_with_git_only();

    let output = repository
        .xtask("scan")
        .env("PATH", tools.path())
        .output()
        .expect("xtask runs");
    assert_fails_with(
        &output,
        "error: gitleaks is not on the PATH; install 8.30.1 from https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1",
    );
}

#[cfg(unix)]
#[test]
fn another_gitleaks_version_fails() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let repository = committed(&[]);
    let tools = tools_with_git_only();
    let fake = tools.path().join("gitleaks");
    fs::write(&fake, "#!/bin/sh\necho 8.29.1\n").expect("the fake gitleaks");
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).expect("an executable");

    let output = repository
        .xtask("scan")
        .env("PATH", tools.path())
        .output()
        .expect("xtask runs");
    assert_fails_with(&output, "error: gitleaks 8.29.1 found; 8.30.1 is required");
}

#[test]
fn a_tracked_vault_note_fails() {
    let repository = committed(&[(
        "docs/note.md",
        "---\nid: mem-0001\ntype: gotcha\n---\n\nA note.\n".to_owned(),
    )]);

    assert_fails_with(
        &repository.run("scan"),
        "error: docs/note.md is a vault note; vault notes never enter this repository",
    );
}
