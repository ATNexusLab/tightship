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

const ARCHIVE: &str = "tightship-claude-code.zip";
const CATALOG: &str = "marketplace.json";

pub fn run(root: &Path, tag: &str) -> Result<String, Failure> {
    let dist = root.join("dist");
    let staging = dist.join(".package.tmp");
    let packed = pack(&dist.join("claude-code"), &staging, tag);
    let published = packed.and_then(|summary| {
        for file in [ARCHIVE, CATALOG] {
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

fn pack(plugin: &Path, staging: &Path, tag: &str) -> Result<String, Failure> {
    #[derive(Deserialize)]
    struct Manifest {
        version: String,
        repository: String,
    }
    println!("> Reading dist/claude-code/");
    let manifest_path = plugin.join(".claude-plugin/plugin.json");
    let manifest_text = fs::read_to_string(&manifest_path).map_err(naming(&manifest_path))?;
    let manifest: Manifest = serde_json::from_str(&manifest_text)
        .map_err(|error| io::Error::other(format!("{}: {error}", manifest_path.display())))?;
    let mut diagnostics = Vec::new();
    let version = &manifest.version;
    if tag != format!("v{version}") {
        diagnostics.push(Diagnostic::general(format!(
            "the tag {tag} does not match the version {version}; tag v{version}"
        )));
    }
    if let Some((platform, _)) = PLATFORMS
        .iter()
        .find(|(platform, _)| !plugin.join("bin").join(platform).is_dir())
    {
        diagnostics.push(Diagnostic::general(format!(
            "dist/claude-code/bin/ has no {platform}; run cargo xtask build --binaries <folder> first"
        )));
    }
    stop_on_errors(&mut diagnostics)?;

    println!("> Packing {ARCHIVE}");
    let mut files = Vec::new();
    list_files(plugin, "", &mut files)?;
    files.retain(|file| file != ".claude-plugin/marketplace.json");
    files.sort();
    fs::create_dir_all(staging).map_err(naming(staging))?;
    let archive_path = staging.join(ARCHIVE);
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

    println!("> Writing {CATALOG}");
    let local_path = plugin.join(".claude-plugin").join(CATALOG);
    let local_text = fs::read_to_string(&local_path).map_err(naming(&local_path))?;
    let mut catalog: serde_json::Value = serde_json::from_str(&local_text)
        .map_err(|error| io::Error::other(format!("{}: {error}", local_path.display())))?;
    let entry = catalog
        .pointer_mut("/plugins/0/source")
        .ok_or_else(|| io::Error::other(format!("{} has no plugin", local_path.display())))?;
    *entry = serde_json::json!({
        "source": "archive",
        "url": format!("{}/releases/download/{tag}/{ARCHIVE}", manifest.repository),
        "sha256": sha256,
    });
    let catalog_path = staging.join(CATALOG);
    let catalog_text = serde_json::to_string_pretty(&catalog).map_err(io::Error::other)? + "\n";
    fs::write(&catalog_path, catalog_text).map_err(naming(&catalog_path))?;
    Ok(format!(
        "Packed dist/{ARCHIVE} ({} files) and dist/{CATALOG} for {tag}.\nsha256: {sha256}",
        files.len()
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
