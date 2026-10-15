use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::diagnostics::{Diagnostic, naming};
use crate::git;

pub const GITLEAKS_VERSION: &str = "8.30.1";

pub fn check(root: &Path, diagnostics: &mut Vec<Diagnostic>) -> io::Result<usize> {
    let commits = git::run(root, &["rev-list", "--all", "--count"])?
        .trim()
        .parse()
        .map_err(io::Error::other)?;
    let version = match Command::new("gitleaks").arg("version").output() {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            diagnostics.push(Diagnostic::general(format!(
                "gitleaks is not on the PATH; install {GITLEAKS_VERSION} from https://github.com/gitleaks/gitleaks/releases/tag/v{GITLEAKS_VERSION}"
            )));
            return Ok(commits);
        }
        result => result.map_err(naming(Path::new("gitleaks")))?,
    };
    let found = String::from_utf8_lossy(&version.stdout);
    let found = found.trim().trim_start_matches('v');
    if found != GITLEAKS_VERSION {
        diagnostics.push(Diagnostic::general(format!(
            "gitleaks {found} found; {GITLEAKS_VERSION} is required"
        )));
        return Ok(commits);
    }
    let report = tempfile::Builder::new()
        .prefix("tightship-gitleaks-")
        .suffix(".json")
        .tempfile()?;
    let status = Command::new("gitleaks")
        .args([
            "git",
            "--log-opts=--all",
            "--redact",
            "--verbose",
            "--no-banner",
        ])
        .args(["--report-format", "json", "--report-path"])
        .arg(report.path())
        .arg(root)
        .status()
        .map_err(naming(Path::new("gitleaks")))?;
    let text = fs::read_to_string(report.path()).map_err(naming(report.path()))?;
    let leaks: Vec<serde_json::Value> = serde_json::from_str(&text).map_err(|error| {
        io::Error::other(format!(
            "gitleaks exited with {status} and left no valid report: {error}"
        ))
    })?;
    if !status.success() && leaks.is_empty() {
        return Err(io::Error::other(format!("gitleaks failed: {status}")));
    }
    if !leaks.is_empty() {
        diagnostics.push(Diagnostic::general(format!(
            "gitleaks found {} secrets in the history; its redacted report is above",
            leaks.len()
        )));
    }
    Ok(commits)
}
