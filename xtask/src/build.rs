pub mod binary;
mod claude_code;
mod content;
mod contract;
mod cursor;
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
use harness::Harness;

pub fn run(root: &Path, prebuilt: Option<&Path>) -> Result<String, Failure> {
    match build_plugins(root, prebuilt) {
        Ok(message) => Ok(message),
        Err(failure) => {
            for name in ["claude-code", "cursor"] {
                let _ = claude_code::discard(root, name);
            }
            Err(failure)
        }
    }
}

fn build_plugins(root: &Path, prebuilt: Option<&Path>) -> Result<String, Failure> {
    let mut diagnostics = Vec::new();
    if let Some(folder) = prebuilt {
        println!("> Checking the binaries in {}", folder.display());
        binary::check_prebuilt(folder, &mut diagnostics);
        stop_on_errors(&mut diagnostics)?;
    }
    println!("> Reading content/");
    let content = content::read(root, &mut diagnostics)?;
    let harnesses = harness::read_all(root, &mut diagnostics)?;
    if harnesses.is_empty() {
        diagnostics.push(Diagnostic::general("harnesses/ holds no harness.toml".to_owned()));
    }

    println!("> Checking references and harness neutrality");
    references::check(&content, &harnesses, &mut diagnostics);
    neutrality::check(&content, &harnesses, &mut diagnostics);
    stop_on_errors(&mut diagnostics)?;

    let binaries = match prebuilt {
        Some(folder) => {
            println!("> Copying the four binaries from {}", folder.display());
            binary::prebuilt(folder)
        }
        None => {
            let platform = binary::platform()?;
            println!("> Compiling tightship for {platform}");
            vec![binary::compile(platform)?]
        }
    };

    let mut messages = Vec::new();
    for harness in &harnesses {
        build_one(root, harness, &content, &binaries, &mut messages)?;
    }
    Ok(messages.join("\n"))
}

fn build_one(
    root: &Path,
    harness: &Harness,
    content: &Content,
    binaries: &[binary::Binary],
    messages: &mut Vec<String>,
) -> Result<(), Failure> {
    let parts = if let Some(contract) = &content.contract
        && harness.contract.is_some()
    {
        println!("> Packing the contract for {}", harness.name);
        let mut diagnostics = Vec::new();
        let parts = contract::pack(contract, harness, &mut diagnostics);
        stop_on_errors(&mut diagnostics)?;
        parts
    } else {
        Vec::new()
    };

    println!("> Rendering {} into dist/{}/", harness.name, harness.name);
    let (staging, files) = match harness.name.as_str() {
        "claude-code" => claude_code::render(root, harness, content, &parts, binaries)?,
        "cursor" => cursor::render(root, harness, content, &parts, binaries)?,
        _ => {
            return Err(Failure::Io(io::Error::other(format!(
                "build failed for {}: no renderer for this harness",
                harness.name
            ))));
        }
    };

    let dist_label = format!("dist/{}", harness.name);
    println!("> Scanning {dist_label} for identity terms");
    let mut diagnostics = Vec::new();
    identity::check(root, &staging, &dist_label, &mut diagnostics)?;
    stop_on_errors(&mut diagnostics)?;
    claude_code::publish(root, &staging, &harness.name)?;
    messages.push(format!(
        "{}\n{}",
        summary(harness, content, files),
        try_it(harness),
    ));
    Ok(())
}

fn summary(harness: &Harness, content: &Content, files: usize) -> String {
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
        "Built dist/{}/: {} ({}, {}, {}), {}.",
        harness.name,
        plural(content.pieces.len(), "piece"),
        count(Kind::Skill),
        count(Kind::Command),
        count(Kind::Agent),
        plural(files, "file"),
    )
}

fn try_it(harness: &Harness) -> String {
    match harness.name.as_str() {
        "claude-code" => "Try it: claude --plugin-dir dist/claude-code".to_owned(),
        "cursor" => {
            "Try it: copy dist/cursor to ~/.cursor/plugins/local/tightship and reload the window, or agent --plugin-dir dist/cursor".to_owned()
        }
        _ => format!("Try it: dist/{}/", harness.name),
    }
}
