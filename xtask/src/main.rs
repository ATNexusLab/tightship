mod build;
mod diagnostics;
mod git;
mod package;
mod scan;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use diagnostics::Diagnostic;

enum Command<'arguments> {
    Build(Option<&'arguments Path>),
    Package(&'arguments str),
    Scan,
}

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
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let command = match words.as_slice() {
        ["build"] => Command::Build(None),
        ["build", "--binaries", folder] => Command::Build(Some(Path::new(folder))),
        ["package", tag] => Command::Package(tag),
        ["scan"] => Command::Scan,
        _ => {
            eprintln!(
                "error: usage: cargo xtask build [--binaries <folder>] | package <tag> | scan"
            );
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
    let outcome = match command {
        Command::Build(prebuilt) => build::run(&root, prebuilt),
        Command::Package(tag) => package::run(&root, tag),
        Command::Scan => scan::run(&root),
    };
    match outcome {
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
