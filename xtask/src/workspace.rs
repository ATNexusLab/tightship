use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::diagnostics::with_path;

pub fn root() -> io::Result<PathBuf> {
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

pub fn version(root: &Path) -> io::Result<String> {
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
    let cargo_text = fs::read_to_string(&cargo_path).map_err(with_path(&cargo_path))?;
    let cargo: CargoManifest = toml::from_str(&cargo_text)
        .map_err(|error| io::Error::other(format!("Cargo.toml: {}", error.message())))?;
    Ok(cargo.workspace.package.version)
}
