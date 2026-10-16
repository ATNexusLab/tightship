use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use tempfile::{NamedTempFile, TempDir};

pub struct Repository {
    root: TempDir,
    global_git_config: NamedTempFile,
}

impl Repository {
    pub fn with(files: &[(&str, String)]) -> Self {
        let repository = Self {
            root: tempfile::tempdir().expect("a temporary directory"),
            global_git_config: NamedTempFile::new().expect("an empty git config"),
        };
        let harness = include_str!("../../../harnesses/claude-code/harness.toml");
        let base = [
            (
                "Cargo.toml",
                "[workspace.package]\nversion = \"0.1.0\"\n".to_owned(),
            ),
            ("harnesses/claude-code/harness.toml", harness.to_owned()),
            (".identity-terms", "fixture-maintainer-handle\n".to_owned()),
        ];
        for (path, text) in base.iter().chain(files) {
            repository.write(path, text);
        }
        repository.git(&["init", "--quiet"]);
        repository.git(&["config", "user.name", "Fixture Maintainer"]);
        repository.git(&["config", "user.email", "maintainer@fixture.invalid"]);
        repository
    }

    pub fn write(&self, path: &str, text: &str) {
        let file = self.path(path);
        fs::create_dir_all(file.parent().expect("a parent folder")).expect("the parent folder");
        fs::write(file, text).expect("the fixture file");
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.path().join(relative)
    }

    pub fn git(&self, arguments: &[&str]) {
        let output = self
            .isolated(Command::new("git"))
            .args(arguments)
            .output()
            .expect("git runs");
        assert!(
            output.status.success(),
            "git {arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    pub fn xtask(&self, subcommand: &str) -> Command {
        let mut command = self.isolated(Command::new(env!("CARGO_BIN_EXE_xtask")));
        command.arg(subcommand);
        command
    }

    pub fn run(&self, subcommand: &str) -> Output {
        self.xtask(subcommand).output().expect("xtask runs")
    }

    fn isolated(&self, mut command: Command) -> Command {
        command
            .current_dir(self.root.path())
            .env("GIT_CONFIG_GLOBAL", self.global_git_config.path())
            .env("GIT_CONFIG_NOSYSTEM", "1");
        command
    }
}

pub fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[allow(dead_code)]
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
