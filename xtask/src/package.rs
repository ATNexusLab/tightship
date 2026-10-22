use std::fs;
use std::io::{self, Write};
use std::path::Path;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipWriter};

use crate::build::binary::PLATFORMS;
use crate::build::harness;
use crate::build::scripts::pascal;
use crate::diagnostics::{Diagnostic, naming};
use crate::{Failure, stop_on_errors};

const CLAUDE_CATALOG: &str = "marketplace.json";

fn release_catalog_name(harness: &str) -> String {
    if harness == "claude-code" {
        CLAUDE_CATALOG.to_owned()
    } else {
        format!("{harness}-marketplace.json")
    }
}

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
        for entry in fs::read_dir(&staging).map_err(naming(&staging))? {
            let file = entry?.file_name();
            let target = dist.join(&file);
            fs::rename(staging.join(&file), &target).map_err(naming(&target))?;
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
    let harnesses = harness::read_all(root, &mut diagnostics)?;
    let mut manifests = Vec::new();

    for harness in &harnesses {
        let name = &harness.name;
        println!("> Reading dist/{name}/");
        let plugin = dist.join(name);
        let manifest_path = plugin.join(&harness.dist.manifest);
        if !manifest_path.is_file() {
            diagnostics.push(Diagnostic::general(format!(
                "dist/{name}/{} is missing; run cargo xtask build --binaries <folder> first",
                harness.dist.manifest
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
                    "dist/{name}/bin/{platform}/{binary} is missing; run cargo xtask build --binaries <folder> first"
                )));
            }
        }
        manifests.push(manifest);
    }
    for installer in ["install.sh", "install.ps1"] {
        if !dist.join(installer).is_file() {
            diagnostics.push(Diagnostic::general(format!(
                "dist/{installer} is missing; run cargo xtask build --binaries <folder> first"
            )));
        }
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

    match fs::remove_dir_all(staging) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            return Err(naming(staging)(error).into());
        }
        _ => {}
    }
    fs::create_dir_all(staging).map_err(naming(staging))?;

    let mut parts = Vec::new();
    let mut catalogs = Vec::new();
    let mut values = vec![("release-tag".to_owned(), tag.to_owned(), true)];
    for (harness, manifest) in harnesses.iter().zip(&manifests) {
        let archive = format!("{}-{}.zip", harness.plugin.name, harness.name);
        println!("> Packing {archive}");
        let plugin = dist.join(&harness.name);
        let mut files = Vec::new();
        list_files(&plugin, "", &mut files)?;
        files.retain(|file| Some(file) != harness.dist.catalog.as_ref());
        files.sort();

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
        let archive_path = staging.join(&archive);
        fs::write(&archive_path, &bytes).map_err(naming(&archive_path))?;
        let sha256: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();

        if let Some(catalog) = &harness.dist.catalog {
            let url = format!("{}/releases/download/{tag}/{archive}", manifest.repository);
            let release_catalog = release_catalog_name(&harness.name);
            write_catalog(
                &plugin.join(catalog),
                staging,
                &release_catalog,
                &url,
                &sha256,
            )?;
            catalogs.push(format!("dist/{release_catalog}"));
        }
        parts.push(format!("dist/{archive} ({} files)", files.len()));
        values.push((format!("{}-sha256", harness.name), sha256, false));
    }
    write_installers(&dist, staging, &values, tag)?;

    let shas: Vec<String> = harnesses
        .iter()
        .zip(&values[1..])
        .map(|(harness, (_, sha256, _))| format!("sha256 ({}): {sha256}", harness.name))
        .collect();
    let catalog_summary = if catalogs.is_empty() {
        String::new()
    } else {
        format!(", {}", catalogs.join(", "))
    };
    Ok(format!(
        "Packed {}{}, and the installers for {tag}.\n{}",
        parts.join(", "),
        catalog_summary,
        shas.join("\n")
    ))
}

fn write_catalog(
    local_path: &Path,
    staging: &Path,
    release_name: &str,
    url: &str,
    sha256: &str,
) -> Result<(), Failure> {
    println!("> Writing {release_name}");
    let local_text = fs::read_to_string(local_path).map_err(naming(local_path))?;
    let mut catalog: serde_json::Value = serde_json::from_str(&local_text)
        .map_err(|error| io::Error::other(format!("{}: {error}", local_path.display())))?;
    let entry = catalog
        .pointer_mut("/plugins/0/source")
        .ok_or_else(|| io::Error::other(format!("{} has no plugin", local_path.display())))?;
    *entry = serde_json::json!({
        "source": "archive",
        "url": url,
        "sha256": sha256,
    });
    let catalog_path = staging.join(release_name);
    let catalog_text = serde_json::to_string_pretty(&catalog).map_err(io::Error::other)? + "\n";
    fs::write(&catalog_path, catalog_text).map_err(naming(&catalog_path))?;
    Ok(())
}

fn write_installers(
    dist: &Path,
    staging: &Path,
    values: &[(String, String, bool)],
    tag: &str,
) -> Result<(), Failure> {
    for installer in ["install.sh", "install.ps1"] {
        println!("> Writing {installer} for {tag}");
        let source = dist.join(installer);
        let text = fs::read_to_string(&source).map_err(naming(&source))?;
        let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
        for (name, value, required) in values {
            let prefix = if installer == "install.sh" {
                format!("{}=", name.replace('-', "_").to_uppercase())
            } else {
                format!("${} = ", pascal(name))
            };
            let empty = format!("{prefix}\"\"");
            let mut matching = lines.iter_mut().filter(|line| **line == empty);
            match (matching.next(), matching.next(), required) {
                (Some(line), None, _) => *line = format!("{prefix}\"{value}\""),
                (None, None, false) => {}
                _ => {
                    return Err(
                        io::Error::other(format!("{installer} needs one line {empty}")).into(),
                    );
                }
            }
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
