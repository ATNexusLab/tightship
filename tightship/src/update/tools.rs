use std::io;
use std::path::Path;
use std::process::{Command, Output};

use crate::failure::Failure;

pub fn download(url: &str, file: &Path) -> Result<(), Failure> {
    let failed = |message| Failure::UpdateDownloadFailed(url.to_owned(), message);
    if !url.starts_with("https://") && !url.starts_with("file://") {
        return Err(failed("only https and file URLs are allowed".to_owned()));
    }
    #[cfg(windows)]
    let (mut command, tool) = {
        let mut command = powershell(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-RestMethod -UseBasicParsing -TimeoutSec 60 -Uri $env:TIGHTSHIP_UPDATE_URL -OutFile $env:TIGHTSHIP_UPDATE_FILE",
        );
        command
            .env("TIGHTSHIP_UPDATE_URL", url)
            .env("TIGHTSHIP_UPDATE_FILE", file);
        (command, "powershell")
    };
    #[cfg(not(windows))]
    let (mut command, tool) = {
        let mut command = Command::new("curl");
        command
            .args(["--fail", "--location", "--silent", "--show-error"])
            .args([
                "--max-time",
                "60",
                "--proto",
                "=https,file",
                "--proto-redir",
                "=https",
            ])
            .arg("--output")
            .arg(file)
            .arg(url);
        (command, "curl")
    };
    run(&mut command, tool, failed).map(drop)
}

pub fn sha256(file: &Path, archive: &str) -> Result<String, Failure> {
    let failed = |message| Failure::UpdateChecksumFailed(archive.to_owned(), message);
    #[cfg(windows)]
    let printed = {
        let mut command = powershell(
            "(Get-FileHash -Algorithm SHA256 -LiteralPath $env:TIGHTSHIP_UPDATE_FILE).Hash",
        );
        command.env("TIGHTSHIP_UPDATE_FILE", file);
        run(&mut command, "powershell", failed)?
    };
    #[cfg(not(windows))]
    let printed = match run(Command::new("sha256sum").arg(file), "sha256sum", failed) {
        Err(Failure::UpdateToolMissing(_)) => run(
            Command::new("shasum").args(["-a", "256"]).arg(file),
            "sha256sum",
            failed,
        )?,
        printed => printed?,
    };
    printed
        .split_whitespace()
        .next()
        .filter(|sha256| sha256.len() == 64 && sha256.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| failed(format!("it printed {:?}", printed.trim())))
}

pub fn extract(file: &Path, folder: &Path, archive: &str) -> Result<(), Failure> {
    let failed = |message| Failure::UpdateExtractFailed(archive.to_owned(), message);
    #[cfg(windows)]
    let (mut command, tool) = {
        let mut command = powershell(
            "Expand-Archive -Force -LiteralPath $env:TIGHTSHIP_UPDATE_FILE -DestinationPath $env:TIGHTSHIP_UPDATE_FOLDER",
        );
        command
            .env("TIGHTSHIP_UPDATE_FILE", file)
            .env("TIGHTSHIP_UPDATE_FOLDER", folder);
        (command, "powershell")
    };
    #[cfg(not(windows))]
    let (mut command, tool) = {
        let mut command = Command::new("unzip");
        command.arg("-q").arg(file).arg("-d").arg(folder);
        (command, "unzip")
    };
    run(&mut command, tool, failed).map(drop)
}

#[cfg(windows)]
fn powershell(script: &str) -> Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut command = Command::new("powershell");
    command
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg(format!(
            "$ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue'; try {{ {script} }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
        ))
        .env_remove("PSModulePath")
        .creation_flags(CREATE_NO_WINDOW);
    command
}

fn run(
    command: &mut Command,
    tool: &'static str,
    failed: impl Fn(String) -> Failure,
) -> Result<String, Failure> {
    match command.output() {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Err(Failure::UpdateToolMissing(tool))
        }
        Err(error) => Err(failed(error.to_string())),
        Ok(output) if !output.status.success() => Err(failed(message_of(&output))),
        Ok(output) => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
    }
}

fn message_of(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if message.is_empty() {
        output.status.to_string()
    } else {
        message
    }
}
