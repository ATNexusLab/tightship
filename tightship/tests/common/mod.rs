use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

pub struct Sandbox {
    root: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            root: tempfile::tempdir().expect("a temporary directory"),
        }
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.path().join(relative)
    }

    pub fn write(&self, relative: &str, text: &str) -> PathBuf {
        let file = self.path(relative);
        fs::create_dir_all(file.parent().expect("a parent folder")).expect("the parent folder");
        fs::write(&file, text).expect("the fixture file");
        file
    }

    pub fn tightship(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_tightship"));
        command
            .args(arguments)
            .current_dir(self.root.path())
            .env_remove("TIGHTSHIP_VAULT")
            .env_remove("TIGHTSHIP_ENV_FILE")
            .env_remove("TIGHTSHIP_DATA")
            .env("GIT_CONFIG_GLOBAL", self.write(".gitconfig", ""))
            .env("GIT_CONFIG_NOSYSTEM", "1");
        command
    }
}

pub fn run(command: &mut Command) -> Output {
    command.output().expect("the command runs")
}

pub fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub fn assert_fails_with(output: &Output, line: &str) {
    let stderr = stderr_of(output);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stderr.lines().any(|found| found == line),
        "missing {line:?} in:\n{stderr}"
    );
}

pub fn assert_succeeds(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_of(output)
    );
}

pub fn note(sandbox: &Sandbox, scope: &str, id: &str, extra: &str, body: &str) -> PathBuf {
    sandbox.write(
        &format!("vault/notes/{scope}/{id}-a-note.md"),
        &format!(
            "---\nid: {id}\ntype: gotcha\nproject: {scope}\nwhy: the {id} why costs real time\n{extra}source: session\ncreated: 2026-10-01\nupdated: 2026-10-01\n---\n\n# A note\n\n{body}\n"
        ),
    )
}

pub fn with_stdin(command: &mut Command, input: &str) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the command starts");
    child
        .stdin
        .take()
        .expect("a stdin")
        .write_all(input.as_bytes())
        .expect("the input");
    child.wait_with_output().expect("the command ends")
}
