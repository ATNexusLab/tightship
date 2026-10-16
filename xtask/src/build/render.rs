use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::binary::Binary;
use super::content::{Content, SupportingBody};
use super::harness::Harness;
use super::tokens;
use crate::diagnostics::naming;

pub fn resolve(text: &str, path: &str, harness: &Harness) -> io::Result<String> {
    let (rendered, unresolved) = tokens::render(text, harness);
    if unresolved.is_empty() {
        Ok(rendered)
    } else {
        Err(io::Error::other(format!(
            "{path}: rendered with unresolved tokens {}",
            unresolved.join(", ")
        )))
    }
}

pub fn workspace_version(root: &Path) -> io::Result<String> {
    #[derive(Deserialize)]
    struct CargoManifest {
        workspace: Workspace,
    }
    #[derive(Deserialize)]
    struct Workspace {
        package: Package,
    }
    #[derive(Deserialize)]
    struct Package {
        version: String,
    }
    let cargo_path = root.join("Cargo.toml");
    let cargo_text = fs::read_to_string(&cargo_path).map_err(naming(&cargo_path))?;
    let cargo: CargoManifest = toml::from_str(&cargo_text)
        .map_err(|error| io::Error::other(format!("Cargo.toml: {}", error.message())))?;
    Ok(cargo.workspace.package.version)
}

pub fn supporting_files(
    staging: &Path,
    content: &Content,
    harness: &Harness,
) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
    for supporting in &content.supporting {
        let bytes = match &supporting.body {
            SupportingBody::Markdown(text) => {
                resolve(text, &supporting.path, harness)?.into_bytes()
            }
            SupportingBody::Copied(bytes) => bytes.clone(),
        };
        let file = staging
            .join("skills")
            .join(&supporting.skill)
            .join(&supporting.relative);
        files.push((file, bytes));
    }
    Ok(files)
}

pub fn binary_files(staging: &Path, binaries: &[Binary]) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
    for binary in binaries {
        let file_name = binary.path.file_name().unwrap_or_default();
        let copy = staging.join("bin").join(binary.platform).join(file_name);
        let bytes = fs::read(&binary.path).map_err(naming(&binary.path))?;
        files.push((copy, bytes));
    }
    Ok(files)
}
