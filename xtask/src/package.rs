use std::fs;
use std::io::{self, Write};
use std::path::Path;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipWriter};

use crate::build::binary::PLATFORMS;
use crate::diagnostics::{Diagnostic, naming};
use crate::{Failure, stop_on_errors};

const ARCHIVE_CLAUDE: &str = "tightship-claude-code.zip";
const ARCHIVE_AGY: &str = "tightship-antigravity-cli.zip";
const CATALOG: &str = "marketplace.json";

const HARNESSES: [(&str, &str, &str); 2] = [
    ("claude-code", ARCHIVE_CLAUDE, ".claude-plugin/plugin.json"),
    ("antigravity-cli", ARCHIVE_AGY, "plugin.json"),
];

pub fn run(root: &Path, tag: &str) -> Result<String, Failure> {
    let dist = root.join("dist");
    let staging = dist.join(".package.tmp");
    let packed = pack(&dist, &staging, tag);
    let published = packed.and_then(|summary| {
        for file in [ARCHIVE_CLAUDE, ARCHIVE_AGY, CATALOG] {
            let target = dist.join(file);
            fs::rename(staging.join(file), &target).map_err(naming(&target))?;
        }
        Ok(summary)
    });
    match fs::remove_dir_all(&staging) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            eprintln!("error: {}", naming(&staging)(error));
        }
        _ => {}
    }
    published
}

fn pack(dist: &Path, staging: &Path, tag: &str) -> Result<String, Failure> {
    #[derive(Deserialize)]
    struct Manifest {
        version: String,
        repository: String,
    }

    let mut diagnostics = Vec::new();
    let mut manifests = Vec::new();

    for (harness, _, manifest_rel) in HARNESSES {
        println!("> Reading dist/{harness}/");
        let plugin = dist.join(harness);
        let manifest_path = plugin.join(manifest_rel);
        if !manifest_path.is_file() {
            diagnostics.push(Diagnostic::general(format!(
                "dist/{harness}/plugin.json is missing; run cargo xtask build --binaries <folder> first"
            )));
            continue;
        }
        let manifest_text = fs::read_to_string(&manifest_path).map_err(naming(&manifest_path))?;
        let manifest: Manifest = serde_json::from_str(&manifest_text)
            .map_err(|error| io::Error::other(format!("{}: {error}", manifest_path.display())))?;
        let version = &manifest.version;
        if tag != format!("v{version}") {
            diagnostics.push(Diagnostic::general(format!(
                "the tag {tag} does not match the version {version}; tag v{version}"
            )));
        }
        for (platform, binary) in PLATFORMS {
            let binary_path = plugin.join("bin").join(platform).join(binary);
            if !binary_path.is_file() {
                diagnostics.push(Diagnostic::general(format!(
                    "dist/{harness}/bin/{platform}/{binary} is missing; run cargo xtask build --binaries <folder> first"
                )));
            }
        }
        manifests.push((harness, manifest));
    }
    stop_on_errors(&mut diagnostics)?;

    fs::create_dir_all(staging).map_err(naming(staging))?;

    let mut archive_summaries = Vec::new();
    let mut claude_info = None;

    for (harness, archive_name, _) in HARNESSES {
        println!("> Packing {archive_name}");
        let plugin = dist.join(harness);
        let mut files = Vec::new();
        list_files(&plugin, "", &mut files)?;
        if harness == "claude-code" {
            files.retain(|file| file != ".claude-plugin/marketplace.json");
        }
        files.sort();

        let archive_path = staging.join(archive_name);
        let mut archive = ZipWriter::new(io::Cursor::new(Vec::new()));
        for file in &files {
            let mode = if file.starts_with("bin/") {
                0o755
            } else {
                0o644
            };
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .system(System::Unix)
                .last_modified_time(DateTime::default())
                .unix_permissions(mode);
            let source = plugin.join(file);
            let bytes = fs::read(&source).map_err(naming(&source))?;
            archive
                .start_file(file, options)
                .map_err(io::Error::other)?;
            archive.write_all(&bytes)?;
        }
        let bytes = archive.finish().map_err(io::Error::other)?.into_inner();
        fs::write(&archive_path, &bytes).map_err(naming(&archive_path))?;
        let sha256: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();

        if harness == "claude-code" {
            let manifest = manifests
                .iter()
                .find(|(h, _)| *h == harness)
                .map(|(_, m)| m)
                .expect("claude-code manifest");
            claude_info = Some((manifest.repository.clone(), sha256.clone()));
        }
        archive_summaries.push((harness, archive_name, files.len(), sha256));
    }

    println!("> Writing {CATALOG}");
    let claude_plugin = dist.join("claude-code");
    let local_path = claude_plugin.join(".claude-plugin").join(CATALOG);
    let local_text = fs::read_to_string(&local_path).map_err(naming(&local_path))?;
    let mut catalog: serde_json::Value = serde_json::from_str(&local_text)
        .map_err(|error| io::Error::other(format!("{}: {error}", local_path.display())))?;
    let entry = catalog
        .pointer_mut("/plugins/0/source")
        .ok_or_else(|| io::Error::other(format!("{} has no plugin", local_path.display())))?;
    let (repo, claude_sha256) = claude_info.expect("claude-code info");
    *entry = serde_json::json!({
        "source": "archive",
        "url": format!("{repo}/releases/download/{tag}/{ARCHIVE_CLAUDE}"),
        "sha256": claude_sha256,
    });
    let catalog_path = staging.join(CATALOG);
    let catalog_text = serde_json::to_string_pretty(&catalog).map_err(io::Error::other)? + "\n";
    fs::write(&catalog_path, catalog_text).map_err(naming(&catalog_path))?;

    let parts: Vec<String> = archive_summaries
        .iter()
        .map(|(_, name, count, _)| format!("dist/{name} ({count} files)"))
        .collect();
    let shas: Vec<String> = archive_summaries
        .iter()
        .map(|(harness, _, _, sha)| format!("sha256 ({harness}): {sha}"))
        .collect();

    Ok(format!(
        "Packed {}, and dist/{CATALOG} for {tag}.\n{}",
        parts.join(", "),
        shas.join("\n")
    ))
}

fn list_files(folder: &Path, prefix: &str, files: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(folder).map_err(naming(folder))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let relative = format!("{prefix}{name}");
        if entry.file_type()?.is_dir() {
            list_files(&entry.path(), &format!("{relative}/"), files)?;
        } else {
            files.push(relative);
        }
    }
    Ok(())
}
