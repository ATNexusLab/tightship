mod claude_code;
mod content;
mod diagnostics;
mod git;
mod harness;
mod identity;
mod neutrality;
mod references;
mod secrets;
mod tokens;
mod vault_notes;
mod words;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use content::{Content, Kind};
use diagnostics::Diagnostic;

const HARNESS: &str = "claude-code";

enum Failure {
    Checks(Vec<Diagnostic>),
    Io(io::Error),
}

impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let command: fn(&Path) -> Result<String, Failure> = match arguments.as_slice() {
        [name] if name == "build" => build,
        [name] if name == "scan" => scan,
        _ => {
            eprintln!("error: usage: cargo xtask build|scan");
            return ExitCode::FAILURE;
        }
    };
    let root = match workspace_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    match command(&root) {
        Ok(result) => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        Err(Failure::Checks(diagnostics)) => {
            for diagnostic in diagnostics {
                eprintln!("error: {diagnostic}");
            }
            ExitCode::FAILURE
        }
        Err(Failure::Io(error)) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn workspace_root() -> io::Result<PathBuf> {
    let current = env::current_dir()?;
    for folder in current.ancestors() {
        let manifest = folder.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let declares_workspace = fs::read_to_string(&manifest)?
            .parse::<toml::Table>()
            .is_ok_and(|table| table.contains_key("workspace"));
        if declares_workspace {
            return Ok(folder.to_path_buf());
        }
    }
    Err(io::Error::other(format!(
        "{} is not inside a Cargo workspace",
        current.display()
    )))
}

fn build(root: &Path) -> Result<String, Failure> {
    let built = build_plugin(root);
    if built.is_err()
        && let Err(error) = claude_code::discard(root, HARNESS)
    {
        eprintln!("error: {error}");
    }
    built
}

fn build_plugin(root: &Path) -> Result<String, Failure> {
    println!("> Reading content/");
    let mut diagnostics = Vec::new();
    let content = content::read(root, &mut diagnostics)?;
    let harnesses = harness::read_all(root, &mut diagnostics)?;
    let harness_path = format!("harnesses/{HARNESS}/harness.toml");
    if !root.join("harnesses").join(HARNESS).is_dir() {
        diagnostics.push(Diagnostic::general(format!("{harness_path} is missing")));
    }

    println!("> Checking references and harness neutrality");
    references::check(&content, &harnesses, &mut diagnostics);
    neutrality::check(&content, &harnesses, &mut diagnostics);
    stop_on_errors(&mut diagnostics)?;

    println!("> Rendering {HARNESS} into dist/{HARNESS}/");
    let harness = harnesses
        .iter()
        .find(|harness| harness.name == HARNESS)
        .ok_or_else(|| io::Error::other(format!("{harness_path} could not be read")))?;
    let (staging, files) = claude_code::render(root, harness, &content)?;

    println!("> Scanning dist/{HARNESS}/ for identity terms");
    identity::check(root, &staging, &format!("dist/{HARNESS}"), &mut diagnostics)?;
    stop_on_errors(&mut diagnostics)?;
    claude_code::publish(root, &staging, HARNESS)?;
    Ok(format!(
        "{}\nTry it: claude --plugin-dir dist/{HARNESS}",
        summary(&content, files)
    ))
}

fn scan(root: &Path) -> Result<String, Failure> {
    println!(
        "> Scanning the git history for secrets (gitleaks {})",
        secrets::GITLEAKS_VERSION
    );
    let mut diagnostics = Vec::new();
    let commits = secrets::check(root, &mut diagnostics)?;
    println!("> Looking for vault notes in the working tree");
    vault_notes::check(root, &mut diagnostics)?;
    stop_on_errors(&mut diagnostics)?;
    Ok(format!("No secret in {commits} commits. No vault note."))
}

fn stop_on_errors(diagnostics: &mut Vec<Diagnostic>) -> Result<(), Failure> {
    if diagnostics.is_empty() {
        return Ok(());
    }
    diagnostics.sort();
    Err(Failure::Checks(std::mem::take(diagnostics)))
}

fn summary(content: &Content, files: usize) -> String {
    let plural = |count: usize, noun: &str| {
        if count == 1 {
            format!("{count} {noun}")
        } else {
            format!("{count} {noun}s")
        }
    };
    let count = |kind: Kind| {
        let pieces = content.pieces.iter().filter(|piece| piece.kind == kind);
        plural(pieces.count(), kind.noun())
    };
    format!(
        "Built dist/{HARNESS}/: {} ({}, {}, {}), {}.",
        plural(content.pieces.len(), "piece"),
        count(Kind::Skill),
        count(Kind::Command),
        count(Kind::Agent),
        plural(files, "file"),
    )
}
