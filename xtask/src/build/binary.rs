use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Binary {
    pub platform: &'static str,
    pub path: PathBuf,
}

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
