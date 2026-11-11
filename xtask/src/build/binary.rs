//! The `tightship` binaries a plugin ships: compiled for this machine, or copied from a release build.

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::diagnostics::Diagnostic;

/// Each platform folder under `bin/` and the binary's file name in it.
pub const PLATFORMS: [(&str, &str); 4] = [
    ("macos", "tightship"),
    ("linux-x64", "tightship"),
    ("linux-arm64", "tightship"),
    ("windows-x64", "tightship.exe"),
];

/// A built binary and the platform folder it goes into.
pub struct Binary {
    pub platform: &'static str,
    pub path: PathBuf,
}

/// The platform folder of the machine running the build.
///
/// # Errors
///
/// When the machine is none of the four platforms.
pub fn platform() -> io::Result<&'static str> {
    match (env::consts::OS, env::consts::ARCH) {
        ("macos", _) => Ok("macos"),
        ("linux", "x86_64") => Ok("linux-x64"),
        ("linux", "aarch64") => Ok("linux-arm64"),
        ("windows", "x86_64") => Ok("windows-x64"),
        (system, machine) => Err(io::Error::other(format!(
            "tightship has no platform folder for {system} {machine}"
        ))),
    }
}

/// Compiles `tightship` in release mode, with the home and workspace paths remapped out of it.
///
/// # Errors
///
/// When cargo cannot run or the build fails.
pub fn compile(platform: &'static str) -> io::Result<Binary> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("xtask/ has no parent folder"))?;
    let target = workspace.join("target").join("tightship");
    let mut remaps = Vec::new();
    if let Some(home) = env::home_dir() {
        remaps.push(format!("--remap-path-prefix={}=~", home.display()));
    }
    remaps.push(format!("--remap-path-prefix={}=.", workspace.display()));
    let status = Command::new("cargo")
        .args(["build", "--release", "--locked", "--package", "tightship"])
        .arg("--target-dir")
        .arg(&target)
        .current_dir(workspace)
        .env("CARGO_ENCODED_RUSTFLAGS", remaps.join("\u{1f}"))
        .status()
        .map_err(|error| io::Error::new(error.kind(), format!("cargo: {error}")))?;
    if !status.success() {
        return Err(io::Error::other("cargo build --package tightship failed"));
    }
    let file_name = format!("tightship{}", env::consts::EXE_SUFFIX);
    Ok(Binary {
        platform,
        path: target.join("release").join(file_name),
    })
}

/// Reports each of the four platform binaries missing from a `--binaries` folder.
pub fn check_prebuilt(folder: &Path, diagnostics: &mut Vec<Diagnostic>) {
    for (platform, file_name) in PLATFORMS {
        if !folder.join(platform).join(file_name).is_file() {
            diagnostics.push(Diagnostic::general(format!(
                "{}/{platform}/{file_name} is missing; --binaries needs macos, linux-x64, linux-arm64, and windows-x64",
                folder.display()
            )));
        }
    }
}

/// The four platform binaries of a `--binaries` folder.
pub fn prebuilt(folder: &Path) -> Vec<Binary> {
    PLATFORMS
        .into_iter()
        .map(|(platform, file_name)| Binary {
            platform,
            path: folder.join(platform).join(file_name),
        })
        .collect()
}
