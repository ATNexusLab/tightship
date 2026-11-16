use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;

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
        relative
            .split('/')
            .fold(self.root.path().to_path_buf(), |path, part| path.join(part))
    }

    pub fn write(&self, relative: &str, text: &str) -> PathBuf {
        let file = self.path(relative);
        fs::create_dir_all(file.parent().expect("a parent folder")).expect("the parent folder");
        fs::write(&file, text).expect("the fixture file");
        file
    }

    pub fn tightship(&self, arguments: &[&str]) -> Command {
        self.tightship_at(Path::new(env!("CARGO_BIN_EXE_tightship")), arguments)
    }

    pub fn tightship_at(&self, binary: &Path, arguments: &[&str]) -> Command {
        let mut command = Command::new(binary);
        command
            .args(arguments)
            .current_dir(self.root.path())
            .env_remove("TIGHTSHIP_VAULT")
            .env_remove("TIGHTSHIP_ENV_FILE")
            .env_remove("TIGHTSHIP_DATA")
            .env_remove("TIGHTSHIP_STATUS_LINE")
            .env_remove("TIGHTSHIP_STATUS_LINE_REFRESH")
            .env_remove("TIGHTSHIP_SETTINGS")
            .env_remove("TIGHTSHIP_ROOT")
            .env_remove("TIGHTSHIP_MANIFEST")
            .env_remove("TIGHTSHIP_HARNESS")
            .env_remove("TIGHTSHIP_AUTO_UPDATE")
            .env_remove("TIGHTSHIP_CLEAN")
            .env_remove("TIGHTSHIP_CONFIG")
            .env_remove("TIGHTSHIP_CONTRACT")
            .env_remove("TIGHTSHIP_MEMORY")
            .env_remove("TIGHTSHIP_HOOK_OUTPUT")
            .env_remove("TIGHTSHIP_CHECKPOINT_OUTPUT")
            .env_remove("TIGHTSHIP_MODEL_PREFIX")
            .env_remove("TIGHTSHIP_PROJECT_POINTER")
            .env_remove("TIGHTSHIP_STATUS_LINE_COMMAND")
            .env_remove("TIGHTSHIP_UPDATE_FILE")
            .env_remove("TIGHTSHIP_UPDATE_FOLDER")
            .env_remove("TIGHTSHIP_UPDATE_URL")
            .env("GIT_CONFIG_GLOBAL", self.write(".gitconfig", ""))
            .env("GIT_CONFIG_NOSYSTEM", "1");
        command
    }
}

pub const BLOCK: &str = concat!(
    "<!-- tightship:rules:start -->\n",
    include_str!("../../../content/vault-template/AGENTS.md"),
    "<!-- tightship:rules:end -->\n"
);

pub fn project(sandbox: &Sandbox, name: &str) -> PathBuf {
    let folder = sandbox.path(name);
    fs::create_dir_all(folder.join(".git")).expect("the project's .git");
    folder
}

pub fn run(command: &mut Command) -> Output {
    // On Linux, a process another test thread forks keeps a just-written binary open for writing
    // until it execs, and running the binary in that window fails with ETXTBSY (rust-lang/rust#114554).
    for attempt in 1..=5 {
        match command.output() {
            Err(error) if error.kind() == io::ErrorKind::ExecutableFileBusy && attempt < 5 => {
                thread::sleep(Duration::from_millis(50 * attempt));
            }
            output => return output.expect("the command runs"),
        }
    }
    unreachable!("the last attempt returns")
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
    let written = child
        .stdin
        .take()
        .expect("a stdin")
        .write_all(input.as_bytes());
    match written {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
        written => written.expect("the input"),
    }
    child.wait_with_output().expect("the command ends")
}

pub const IDENTITY: [(&str, &str); 4] = [
    ("GIT_AUTHOR_NAME", "Ada Vault"),
    ("GIT_AUTHOR_EMAIL", "ada@example.test"),
    ("GIT_COMMITTER_NAME", "Ada Vault"),
    ("GIT_COMMITTER_EMAIL", "ada@example.test"),
];

pub fn git(sandbox: &Sandbox, folder: &Path, arguments: &[&str]) -> String {
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

pub fn commit(sandbox: &Sandbox, clone: &Path, message: &str) {
    git(sandbox, clone, &["add", "-A"]);
    git(sandbox, clone, &["commit", "--quiet", "-m", message]);
}
