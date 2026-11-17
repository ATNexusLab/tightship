//! The repository's docs: their files, the code each one covers, and the `tightship docs` commands.

pub mod affected;
pub mod check;
mod front_matter;
mod glob;
mod markdown;

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;

use crate::failure::Failure;
use crate::project;
use crate::system::git;

/// The folders, from the repository's root, that hold docs.
const FOLDERS: [&str; 2] = ["docs/", "client-docs/"];

/// The repository a docs command runs in: its root and every file git does not ignore.
pub struct Repository {
    pub root: PathBuf,
    pub files: Vec<String>,
}

/// A doc: its path from the root, its text, and the patterns of the code it covers.
pub struct Doc {
    pub path: String,
    pub text: String,
    pub covers: Vec<front_matter::Cover>,
    pub body_start: usize,
}

impl Repository {
    /// The repository around the working folder, with every file it tracks or holds untracked.
    ///
    /// # Errors
    ///
    /// When the working folder is in no git repository, or git cannot list its files.
    pub fn open() -> Result<Self, Failure> {
        let folder = env::current_dir().expect("the command's working directory");
        let root = project::repository_root(&folder)
            .ok_or(Failure::DocsNoRepository)?
            .to_path_buf();
        let listed = git::run(
            &root,
            &["ls-files", "--cached", "--others", "--exclude-standard"],
        )
        .map_err(Failure::DocsGitFailed)?;
        let mut files: Vec<String> = listed
            .lines()
            .filter(|file| root.join(file).is_file())
            .map(str::to_owned)
            .collect();
        files.sort();
        files.dedup();
        Ok(Self { root, files })
    }

    /// Every doc under `docs/` and `client-docs/`, with its `covers`.
    ///
    /// # Errors
    ///
    /// When a doc cannot be read.
    pub fn docs(&self) -> Result<Vec<Doc>, Failure> {
        self.files
            .iter()
            .filter(|file| {
                file.ends_with(".md") && FOLDERS.iter().any(|folder| file.starts_with(folder))
            })
            .map(|file| {
                let text = self.read(file)?;
                let (covers, body_start) = front_matter::covers(&text);
                Ok(Doc {
                    path: file.clone(),
                    text,
                    covers,
                    body_start,
                })
            })
            .collect()
    }

    /// The text of a file, from its path under the root.
    ///
    /// # Errors
    ///
    /// When the file cannot be read.
    pub fn read(&self, file: &str) -> Result<String, Failure> {
        let path = self.root.join(file);
        fs::read(&path)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .map_err(|error| Failure::DocsUnreadable(path, error))
    }

    /// The files of the repository the patterns match.
    pub fn covered<'a>(&'a self, covers: &'a [front_matter::Cover]) -> Vec<&'a str> {
        self.files
            .iter()
            .map(String::as_str)
            .filter(|file| {
                covers
                    .iter()
                    .any(|cover| glob::matches(&cover.pattern, file))
            })
            .collect()
    }

    /// The paths among `paths` that the repository's ignore rules exclude.
    ///
    /// # Errors
    ///
    /// When git cannot run.
    pub fn ignored(&self, paths: &[String]) -> Result<Vec<String>, Failure> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        let mut child = git::command(&self.root)
            .args(["check-ignore", "--no-index", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| Failure::DocsGitFailed(error.to_string()))?;
        let input = paths.join("\n") + "\n";
        child
            .stdin
            .take()
            .expect("a piped stdin")
            .write_all(input.as_bytes())
            .map_err(|error| Failure::DocsGitFailed(error.to_string()))?;
        let output = child
            .wait_with_output()
            .map_err(|error| Failure::DocsGitFailed(error.to_string()))?;
        // check-ignore exits 1 when no path is ignored, which is no failure.
        if !matches!(output.status.code(), Some(0 | 1)) {
            return Err(Failure::DocsGitFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_owned)
            .collect())
    }
}
