mod binary;
mod claude_code;
mod content;
mod contract;
mod harness;
mod identity;
mod neutrality;
mod options;
mod references;
mod tokens;
mod words;

use std::io;
use std::path::Path;

use crate::diagnostics::Diagnostic;
use crate::{Failure, stop_on_errors};
use content::{Content, Kind};

const HARNESS: &str = "claude-code";

pub fn run(root: &Path) -> Result<String, Failure> {
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
    let harness = harnesses.iter().find(|harness| harness.name == HARNESS);
    let parts = match (&content.contract, harness) {
        (Some(contract), Some(harness)) => {
            println!("> Packing the contract");
            contract::pack(contract, harness, &mut diagnostics)
        }
        _ => Vec::new(),
    };
    stop_on_errors(&mut diagnostics)?;

    let platform = binary::platform()?;
    println!("> Compiling tightship for {platform}");
    let binary = binary::compile(platform)?;

    println!("> Rendering {HARNESS} into dist/{HARNESS}/");
    let harness =
        harness.ok_or_else(|| io::Error::other(format!("{harness_path} could not be read")))?;
    let (staging, files) = claude_code::render(root, harness, &content, &parts, &binary)?;

    println!("> Scanning dist/{HARNESS}/ for identity terms");
    identity::check(root, &staging, &format!("dist/{HARNESS}"), &mut diagnostics)?;
    stop_on_errors(&mut diagnostics)?;
    claude_code::publish(root, &staging, HARNESS)?;
    Ok(format!(
        "{}\nTry it: claude --plugin-dir dist/{HARNESS}",
        summary(&content, files)
    ))
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
