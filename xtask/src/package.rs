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

const CATALOG: &str = "marketplace.json";
const INSTALLERS: [(&str, [&str; 2]); 2] = [
    ("install.sh", ["RELEASE_TAG=", "ANTIGRAVITY_CLI_SHA256="]),
    (
        "install.ps1",
        ["$ReleaseTag = ", "$AntigravityCliSha256 = "],
    ),
];

struct Archive {
    harness: &'static str,
    file: &'static str,
    manifest: &'static str,
}

const CLAUDE_CODE: Archive = Archive {
    harness: "claude-code",
    file: "tightship-claude-code.zip",
    manifest: ".claude-plugin/plugin.json",
};

const ANTIGRAVITY_CLI: Archive = Archive {
    harness: "antigravity-cli",
    file: "tightship-antigravity-cli.zip",
    manifest: "plugin.json",
};

const ARCHIVES: [Archive; 2] = [CLAUDE_CODE, ANTIGRAVITY_CLI];

#[derive(Deserialize)]
struct Manifest {
    version: String,
    repository: String,
}

pub fn run(root: &Path, tag: &str) -> Result<String, Failure> {
    let dist = root.join("dist");
    let staging = dist.join(".package.tmp");
    let packed = pack(root, &staging, tag);
    let published = packed.and_then(|summary| {
        let installers = INSTALLERS.iter().map(|(installer, _)| *installer);
        let files = ARCHIVES
            .iter()
            .map(|archive| archive.file)
            .chain([CATALOG])
            .chain(installers);
        for file in files {
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

fn pack(root: &Path, staging: &Path, tag: &str) -> Result<String, Failure> {
    let dist = root.join("dist");
    let mut diagnostics = Vec::new();
    let mut manifests = Vec::new();

    for archive in &ARCHIVES {
        let harness = archive.harness;
        println!("> Reading dist/{harness}/");
        let plugin = dist.join(harness);
        let manifest_path = plugin.join(archive.manifest);
        if !manifest_path.is_file() {
            diagnostics.push(Diagnostic::general(format!(
                "dist/{harness}/plugin.json is missing; run cargo xtask build --binaries <folder> first"
            )));
            continue;
        }
        let manifest_text = fs::read_to_string(&manifest_path).map_err(naming(&manifest_path))?;
        let manifest: Manifest = serde_json::from_str(&manifest_text)
            .map_err(|error| io::Error::other(format!("{}: {error}", manifest_path.display())))?;
        for (platform, binary) in PLATFORMS {
            let binary_path = plugin.join("bin").join(platform).join(binary);
            if !binary_path.is_file() {
                diagnostics.push(Diagnostic::general(format!(
                    "dist/{harness}/bin/{platform}/{binary} is missing; run cargo xtask build --binaries <folder> first"
                )));
            }
        }
        manifests.push(manifest);
    }
    let mut versions: Vec<&str> = manifests
        .iter()
        .map(|manifest| manifest.version.as_str())
        .collect();
    versions.sort_unstable();
    versions.dedup();
    for version in versions {
        if tag != format!("v{version}") {
            diagnostics.push(Diagnostic::general(format!(
                "the tag {tag} does not match the version {version}; tag v{version}"
            )));
        }
    }
    stop_on_errors(&mut diagnostics)?;

    fs::create_dir_all(staging).map_err(naming(staging))?;

    let mut parts = Vec::new();
    let mut shas = Vec::new();
    for (archive, manifest) in ARCHIVES.iter().zip(&manifests) {
        let harness = archive.harness;
        println!("> Packing {}", archive.file);
        let plugin = dist.join(harness);
        let mut files = Vec::new();
        list_files(&plugin, "", &mut files)?;
        if harness == CLAUDE_CODE.harness {
            files.retain(|file| file != ".claude-plugin/marketplace.json");
        }
        files.sort();

        let archive_path = staging.join(archive.file);
        let mut writer = ZipWriter::new(io::Cursor::new(Vec::new()));
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
            writer.start_file(file, options).map_err(io::Error::other)?;
            writer.write_all(&bytes)?;
        }
        let bytes = writer.finish().map_err(io::Error::other)?.into_inner();
        fs::write(&archive_path, &bytes).map_err(naming(&archive_path))?;
        let sha256: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();

        if harness == CLAUDE_CODE.harness {
            write_catalog(&dist, staging, &manifest.repository, &sha256, tag)?;
        }
        if harness == ANTIGRAVITY_CLI.harness {
            write_installers(root, staging, &sha256, tag)?;
        }
        parts.push(format!("dist/{} ({} files)", archive.file, files.len()));
        shas.push(format!("sha256 ({harness}): {sha256}"));
    }

    Ok(format!(
        "Packed {}, dist/{CATALOG}, and the installers for {tag}.\n{}",
        parts.join(", "),
        shas.join("\n")
    ))
}

fn write_catalog(
    dist: &Path,
    staging: &Path,
    repository: &str,
    sha256: &str,
    tag: &str,
) -> Result<(), Failure> {
    println!("> Writing {CATALOG}");
    let local_path = dist
        .join(CLAUDE_CODE.harness)
        .join(".claude-plugin")
        .join(CATALOG);
    let local_text = fs::read_to_string(&local_path).map_err(naming(&local_path))?;
    let mut catalog: serde_json::Value = serde_json::from_str(&local_text)
        .map_err(|error| io::Error::other(format!("{}: {error}", local_path.display())))?;
    let entry = catalog
        .pointer_mut("/plugins/0/source")
        .ok_or_else(|| io::Error::other(format!("{} has no plugin", local_path.display())))?;
    *entry = serde_json::json!({
        "source": "archive",
        "url": format!("{repository}/releases/download/{tag}/{}", CLAUDE_CODE.file),
        "sha256": sha256,
    });
    let catalog_path = staging.join(CATALOG);
    let catalog_text = serde_json::to_string_pretty(&catalog).map_err(io::Error::other)? + "\n";
    fs::write(&catalog_path, catalog_text).map_err(naming(&catalog_path))?;
    Ok(())
}

fn write_installers(root: &Path, staging: &Path, sha256: &str, tag: &str) -> Result<(), Failure> {
    for (installer, [tag_line, sha256_line]) in INSTALLERS {
        println!("> Writing {installer} for {tag}");
        let source = root.join(installer);
        let text = fs::read_to_string(&source).map_err(naming(&source))?;
        let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
        for (prefix, value) in [(tag_line, tag), (sha256_line, sha256)] {
            let empty = format!("{prefix}\"\"");
            let mut matching = lines.iter_mut().filter(|line| **line == empty);
            let (Some(line), None) = (matching.next(), matching.next()) else {
                return Err(io::Error::other(format!("{installer} needs one line {empty}")).into());
            };
            *line = format!("{prefix}\"{value}\"");
        }
        let target = staging.join(installer);
        fs::write(&target, lines.join("\n") + "\n").map_err(naming(&target))?;
    }
    Ok(())
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
