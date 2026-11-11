//! The `cargo xtask` entry point: parses the command, finds the workspace, and reports the outcome.

mod build;
mod diagnostics;
mod files;
mod git;
mod package;
mod scan;
mod workspace;

use std::env;
use std::path::Path;
use std::process::ExitCode;

use diagnostics::Failure;

enum Command<'arguments> {
    Build(Option<&'arguments Path>),
    Package(&'arguments str),
    Scan,
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
    let root = match workspace::root() {
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
