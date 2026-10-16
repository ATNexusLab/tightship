mod antigravity_cli;
pub mod binary;
mod claude_code;
mod content;
mod contract;
mod cursor;
pub mod harness;
mod identity;
mod installer;
mod neutrality;
mod options;
mod references;
mod render;
pub mod scripts;
mod staging;
mod tokens;
mod words;

use std::fs;
use std::io;
use std::path::Path;

use crate::diagnostics::{Diagnostic, naming};
use crate::{Failure, stop_on_errors};
use content::{Content, Kind};

pub fn run(root: &Path, prebuilt: Option<&Path>) -> Result<String, Failure> {
    let built = build_plugin(root, prebuilt);
    if built.is_err()
        && let Ok(entries) = fs::read_dir(root.join("harnesses"))
    {
        for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Err(error) = staging::discard(root, &name) {
                eprintln!("error: {error}");
            }
        }
    }
    built
}

fn build_plugin(root: &Path, prebuilt: Option<&Path>) -> Result<String, Failure> {
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
        diagnostics.push(Diagnostic::general(
            "no harnesses found in harnesses/".to_owned(),
        ));
    }

    println!("> Checking references and harness neutrality");
    references::check(&content, &harnesses, &mut diagnostics);
    neutrality::check(&content, &harnesses, &mut diagnostics);
    neutrality::check_binary(root, &harnesses, &mut diagnostics)?;

    let mut contract_parts = Vec::new();
    for harness in &harnesses {
        let parts = match &content.contract {
            Some(contract) => {
                println!("> Packing the contract for {}", harness.name);
                contract::pack(contract, harness, &mut diagnostics)
            }
            None => Vec::new(),
        };
        contract_parts.push(parts);
    }
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

    let mut rendered = Vec::new();
    for (harness, parts) in harnesses.iter().zip(&contract_parts) {
        println!("> Rendering {} into dist/{}/", harness.name, harness.name);
        let (staging, files) = match harness.name.as_str() {
            "claude-code" => claude_code::render(root, harness, &content, parts, &binaries)?,
            "antigravity-cli" => {
                antigravity_cli::render(root, harness, &content, parts, &binaries)?
            }
            "cursor" => cursor::render(root, harness, &content, parts, &binaries)?,
            unknown => {
                return Err(Failure::Io(io::Error::other(format!(
                    "unknown harness \"{unknown}\""
                ))));
            }
        };
        println!("> Scanning dist/{}/ for identity terms", harness.name);
        identity::check(
            root,
            &staging,
            &format!("dist/{}", harness.name),
            &mut diagnostics,
        )?;
        rendered.push((harness, staging, files));
    }
    stop_on_errors(&mut diagnostics)?;
    println!("> Assembling the installers");
    let installers = installer::assemble(root, &harnesses)?;

    let mut summaries = Vec::new();
    for (harness, staging, files) in &rendered {
        staging::publish(root, staging, &harness.name)?;
        summaries.push(format!(
            "{}\nTry it: {}",
            summary(&content, &harness.name, *files),
            harness.dist.try_it
        ));
    }
    let mut names = Vec::new();
    for (file, text) in installers {
        let target = root.join("dist").join(file);
        fs::write(&target, text).map_err(naming(&target))?;
        names.push(format!("dist/{file}"));
    }
    summaries.push(format!("Built {}.", names.join(" and ")));
    Ok(summaries.join("\n"))
}

fn summary(content: &Content, harness_name: &str, files: usize) -> String {
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
        "Built dist/{harness_name}/: {} ({}, {}, {}), {}.",
        plural(content.pieces.len(), "piece"),
        count(Kind::Skill),
        count(Kind::Command),
        count(Kind::Agent),
        plural(files, "file"),
    )
}
