mod build;
mod diagnostics;
mod git;
mod scan;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use diagnostics::Diagnostic;

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
        [name] if name == "build" => build::run,
        [name] if name == "scan" => scan::run,
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

fn stop_on_errors(diagnostics: &mut Vec<Diagnostic>) -> Result<(), Failure> {
    if diagnostics.is_empty() {
        return Ok(());
    }
    diagnostics.sort();
    Err(Failure::Checks(std::mem::take(diagnostics)))
}
