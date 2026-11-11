//! Tests of the two installers, run against mock harness CLIs in a temporary home.

use std::ffi::OsString;
use std::fs;
use std::io::{Cursor, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, System, ZipWriter};

mod antigravity_cli;
mod checks;
mod claude_code;
mod companions;
mod cursor;
mod interactive;
mod release;
mod update;

const RELEASE_TAG: &str = "v1.0.0";

const REMOVED_VARIABLES: [&str; 8] = [
    "TIGHTSHIP_ROOT",
    "TIGHTSHIP_DATA",
    "TIGHTSHIP_SETTINGS",
    "PLUGIN_ROOT",
    "PLUGIN_DATA",
    "AGY_CONFIG_DIR",
    "CLAUDE_CONFIG_DIR",
    "TIGHTSHIP_STATUS_LINE",
];

const AGENT_VERSION: &str = r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "agent 2026.10.16"
    exit 0
fi
exit 0
"#;

const AGY_THAT_INSTALLS: &str = r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "agy 1.4.0"
    exit 0
fi
if [ "$1" = "plugin" ] && [ "$2" = "install" ]; then
    target="$3"
    mkdir -p "$HOME/.gemini/config/plugins/tightship"
    if [ -d "$target" ]; then
        cp -R "$target"/* "$HOME/.gemini/config/plugins/tightship/"
    fi
    exit 0
fi
exit 0
"#;

#[derive(Clone, Copy, Debug)]
enum Installer {
    Sh,
    Pwsh,
}

const INSTALLERS: [Installer; 2] = [Installer::Sh, Installer::Pwsh];

impl Installer {
    fn file(self) -> &'static str {
        match self {
            Installer::Sh => "install.sh",
            Installer::Pwsh => "install.ps1",
        }
    }

    fn built(self) -> PathBuf {
        repo_root().join("dist").join(self.file())
    }

    fn flag(self, name: &str) -> String {
        match self {
            Installer::Sh => format!("--{name}"),
            Installer::Pwsh => {
                let words = name.split('-').map(|word| {
                    let (first, rest) = word.split_at(1);
                    first.to_uppercase() + rest
                });
                format!("-{}", words.collect::<String>())
            }
        }
    }

    fn release_lines(self) -> [&'static str; 3] {
        match self {
            Installer::Sh => ["RELEASE_TAG=", "ANTIGRAVITY_CLI_SHA256=", "CURSOR_SHA256="],
            Installer::Pwsh => [
                "$ReleaseTag = ",
                "$AntigravityCliSha256 = ",
                "$CursorSha256 = ",
            ],
        }
    }
}

struct MockEnvironment {
    temp_dir: TempDir,
    bin_dir: PathBuf,
    home_dir: PathBuf,
    downloads_dir: PathBuf,
    system_temp_dir: PathBuf,
}

impl MockEnvironment {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("a temporary directory");
        let bin_dir = temp_dir.path().join("bin");
        let home_dir = temp_dir.path().join("home");
        let downloads_dir = temp_dir.path().join("downloads");
        let system_temp_dir = temp_dir.path().join("tmp");
        for folder in [&bin_dir, &home_dir, &downloads_dir, &system_temp_dir] {
            fs::create_dir_all(folder).expect("a mock folder");
        }
        let environment = Self {
            temp_dir,
            bin_dir,
            home_dir,
            downloads_dir,
            system_temp_dir,
        };
        environment.add_mock_script(
            "curl",
            &format!(
                r#"#!/bin/sh
url=""
dest=""
while [ $# -gt 0 ]; do
    case "$1" in
        -o) dest="$2"; shift 2 ;;
        -*) shift ;;
        *) url="$1"; shift ;;
    esac
done
key=$(printf '%s' "$url" | sed 's|https://||; s|/|_|g')
source="{}/$key"
if [ ! -f "$source" ]; then
  source="{}/${{url##*/}}"
fi
[ -f "$source" ] || exit 22
mkdir -p "$(dirname "$dest")"
cp "$source" "$dest"
"#,
                environment.downloads_dir.display(),
                environment.downloads_dir.display()
            ),
        );
        environment
    }

    fn isolated_path(&self) -> String {
        let harness_tools = ["agy", "claude", "codex", "cursor-agent"];
        let system_path = std::env::var_os("PATH").unwrap_or_default();
        let without_harnesses = std::env::split_paths(&system_path).filter(|folder| {
            !harness_tools.iter().any(|tool| {
                folder.join(tool).is_file() || folder.join(format!("{tool}.exe")).is_file()
            })
        });
        let entries = std::iter::once(self.bin_dir.clone()).chain(without_harnesses);
        std::env::join_paths(entries)
            .expect("a joinable PATH")
            .to_string_lossy()
            .into_owned()
    }

    fn add_mock_script(&self, name: &str, content: &str) {
        let script_path = self.bin_dir.join(name);
        fs::write(&script_path, content).expect("write mock script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).expect("chmod +x");
        }
        #[cfg(windows)]
        fs::write(
            self.bin_dir.join(format!("{name}.cmd")),
            format!("@sh \"%~dp0{name}\" %*\r\n"),
        )
        .expect("write mock shim");
    }

    fn released(&self, installer: Installer, sha256s: &[&str]) -> PathBuf {
        let source = installer.built();
        let mut text = fs::read_to_string(&source).expect("read installer");
        let values = std::iter::once(RELEASE_TAG).chain(sha256s.iter().copied());
        for (prefix, value) in installer.release_lines().into_iter().zip(values) {
            let empty = format!("{prefix}\"\"");
            assert!(text.contains(&empty), "{} lacks {empty}", installer.file());
            text = text.replacen(&empty, &format!("{prefix}\"{value}\""), 1);
        }
        let released = self.temp_dir.path().join(installer.file());
        fs::write(&released, text).expect("write released installer");
        released
    }

    fn environment(&self) -> [(&'static str, OsString); 6] {
        [
            ("HOME", self.home_dir.clone().into()),
            ("USERPROFILE", self.home_dir.clone().into()),
            ("PATH", self.isolated_path().into()),
            ("TMPDIR", self.system_temp_dir.clone().into()),
            ("TEMP", self.system_temp_dir.clone().into()),
            ("TMP", self.system_temp_dir.clone().into()),
        ]
    }

    fn command(&self, program: &str, working_dir: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(working_dir)
            .envs(self.environment())
            .stdin(Stdio::null());
        for variable in REMOVED_VARIABLES {
            command.env_remove(variable);
        }
        command
    }

    fn pwsh_script(&self, invocation: &str, args: &[&str]) -> String {
        let arguments: Vec<String> = args
            .iter()
            .map(|argument| match argument.strip_prefix("--") {
                Some(name) => Installer::Pwsh.flag(name),
                None if argument.starts_with('-') => argument.to_string(),
                None => quoted(argument),
            })
            .collect();
        let downloads = quoted(&self.downloads_dir.to_string_lossy());
        format!(
            "function Invoke-RestMethod([string]$Uri, [string]$OutFile) {{
    $key = ($Uri -replace '^https://','') -replace '/','_'
    $source = Join-Path {downloads} $key
    if (-not (Test-Path $source)) {{
        $source = Join-Path {downloads} ($Uri.Split('/')[-1])
    }}
    if (-not (Test-Path $source)) {{ throw \"404 $Uri\" }}
    New-Item -ItemType Directory -Force -Path (Split-Path $OutFile) | Out-Null
    Copy-Item $source $OutFile
}}
& {invocation} {}
exit $LASTEXITCODE",
            arguments.join(" ")
        )
    }

    fn pwsh(&self, working_dir: &Path, invocation: &str, args: &[&str]) -> Output {
        self.command("pwsh", working_dir)
            .args([
                "-NoProfile",
                "-Command",
                &self.pwsh_script(invocation, args),
            ])
            .output()
            .expect("pwsh executes")
    }

    fn program(&self, name: &str) -> PathBuf {
        std::env::split_paths(&self.isolated_path())
            .flat_map(|folder| [folder.join(name), folder.join(format!("{name}.exe"))])
            .find(|candidate| candidate.is_file())
            .unwrap_or_else(|| panic!("{name} is on PATH"))
    }

    fn run_in_terminal(
        &self,
        installer: Installer,
        script: &Path,
        args: &[&str],
        answers: &[(&str, &str)],
    ) -> (String, u32) {
        let pty = native_pty_system()
            .openpty(PtySize {
                rows: 50,
                cols: 140,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("a pseudo-terminal");
        let mut command = match installer {
            Installer::Sh => {
                let mut command = CommandBuilder::new(self.program("sh"));
                command.arg(script);
                command.args(args);
                command
            }
            Installer::Pwsh => {
                let mut command = CommandBuilder::new(self.program("pwsh"));
                let invocation = quoted(&script.to_string_lossy());
                command.args([
                    "-NoProfile",
                    "-Command",
                    &self.pwsh_script(&invocation, args),
                ]);
                command
            }
        };
        command.cwd(repo_root());
        for (name, value) in std::env::vars_os().chain(
            self.environment()
                .map(|(name, value)| (OsString::from(name), value)),
        ) {
            command.env(name, value);
        }
        for variable in REMOVED_VARIABLES {
            command.env_remove(variable);
        }
        let mut child = pty
            .slave
            .spawn_command(command)
            .expect("the installer starts");
        drop(pty.slave);
        let mut reader = pty.master.try_clone_reader().expect("a pty reader");
        let mut writer = pty.master.take_writer().expect("a pty writer");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            let end = loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break "end of file".to_string(),
                    Ok(read) => {
                        if sender.send(buffer[..read].to_vec()).is_err() {
                            return;
                        }
                    }
                    Err(error) if error.kind() == ErrorKind::Interrupted => {}
                    Err(error) => break error.to_string(),
                }
            };
            let _ = sender.send(format!("\n[the terminal closed: {end}]").into_bytes());
        });
        let find = |bytes: &[u8], needle: &[u8]| {
            bytes
                .windows(needle.len())
                .position(|window| window == needle)
        };

        let mut output = Vec::new();
        let mut consumed = 0;
        let mut answered = 0;
        let mut cursor_replies = 0;
        let started = Instant::now();
        let status = loop {
            assert!(
                started.elapsed() < Duration::from_secs(90),
                "{installer:?}: stuck after {answered} answers:
{}",
                output.escape_ascii()
            );
            if let Ok(chunk) = receiver.recv_timeout(Duration::from_millis(50)) {
                output.extend(chunk);
            }
            let cursor_queries = output
                .windows(4)
                .filter(|window| *window == b"\x1b[6n")
                .count();
            while cursor_replies < cursor_queries {
                writer.write_all(b"\x1b[1;1R").expect("a cursor reply");
                cursor_replies += 1;
            }
            if let Some((prompt, keys)) = answers.get(answered)
                && let Some(position) = find(&output[consumed..], prompt.as_bytes())
            {
                consumed += position + prompt.len();
                std::thread::sleep(Duration::from_millis(150));
                writer.write_all(keys.as_bytes()).expect("the keys");
                writer.flush().expect("the keys are sent");
                answered += 1;
            }
            if let Some(status) = child.try_wait().expect("the installer's status") {
                while let Ok(chunk) = receiver.recv_timeout(Duration::from_millis(300)) {
                    output.extend(chunk);
                }
                break status;
            }
        };
        let screen = String::from_utf8_lossy(&output).into_owned();
        assert_eq!(
            answered,
            answers.len(),
            "{installer:?}: exited before every answer:
{screen}"
        );
        (screen, status.exit_code())
    }

    fn run(&self, installer: Installer, args: &[&str]) -> Output {
        self.run_script(installer, &installer.built(), args)
    }

    fn run_script(&self, installer: Installer, script: &Path, args: &[&str]) -> Output {
        match installer {
            Installer::Sh => {
                let mut command = self.command("sh", &repo_root());
                command.arg(script).args(args);
                #[cfg(unix)]
                {
                    use std::os::unix::process::CommandExt;
                    unsafe {
                        command.pre_exec(|| {
                            if libc::setsid() == -1 {
                                return Err(std::io::Error::last_os_error());
                            }
                            Ok(())
                        });
                    }
                }
                command.output().expect("install.sh executes")
            }
            Installer::Pwsh => self.pwsh(&repo_root(), &quoted(&script.to_string_lossy()), args),
        }
    }

    fn run_piped(
        &self,
        installer: Installer,
        script: &Path,
        working_dir: &Path,
        args: &[&str],
    ) -> Output {
        match installer {
            Installer::Sh => {
                let text = fs::read(script).expect("read install.sh");
                let mut child = self
                    .command("sh", working_dir)
                    .arg("-s")
                    .arg("--")
                    .args(args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("sh executes");
                child
                    .stdin
                    .take()
                    .expect("piped stdin")
                    .write_all(&text)
                    .expect("pipe install.sh");
                child.wait_with_output().expect("install.sh finishes")
            }
            Installer::Pwsh => {
                let invocation = format!(
                    "([scriptblock]::Create((Get-Content -Raw {})))",
                    quoted(&script.to_string_lossy())
                );
                self.pwsh(working_dir, &invocation, args)
            }
        }
    }
}

fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn from_dir() -> String {
    repo_root()
        .join("dist/antigravity-cli")
        .to_str()
        .expect("valid utf-8 path")
        .to_owned()
}

fn cursor_from_dir() -> String {
    repo_root()
        .join("dist/cursor")
        .to_str()
        .expect("valid utf-8 path")
        .to_owned()
}

fn assert_exit(installer: Installer, output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{installer:?}\nstderr:\n{}\nstdout:\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

fn assert_installer_fails_with(installer: Installer, output: &Output, message: &str) {
    assert_exit(installer, output, 1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(message),
        "{installer:?}: missing {message:?} in:\n{stderr}"
    );
}

fn release_zip(harness: &str) -> (Vec<u8>, String) {
    fn walk(folder: &Path, prefix: &str, files: &mut Vec<String>) {
        for entry in fs::read_dir(folder).expect("a folder") {
            let entry = entry.expect("an entry");
            let relative = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.file_type().expect("a file type").is_dir() {
                walk(&entry.path(), &format!("{relative}/"), files);
            } else {
                files.push(relative);
            }
        }
    }
    let mut files = Vec::new();
    let folder = repo_root().join("dist").join(harness);
    walk(&folder, "", &mut files);
    files.sort();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for file in &files {
        let mode = if file.starts_with("bin/") {
            0o755
        } else {
            0o644
        };
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .system(System::Unix)
            .unix_permissions(mode);
        writer.start_file(file, options).expect("a zip entry");
        writer
            .write_all(&fs::read(folder.join(file)).expect("a plugin file"))
            .expect("the zip entry");
    }
    let zip = writer.finish().expect("a zip").into_inner();
    let sha256 = Sha256::digest(&zip)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    (zip, sha256)
}

fn claude_code_install_path() -> String {
    serde_json::to_string(
        repo_root()
            .join("dist/claude-code")
            .to_str()
            .expect("valid utf-8 path"),
    )
    .expect("a JSON string")
}

fn claude_that_installs() -> String {
    r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "claude 2.1.295"
    exit 0
fi
echo "$@" >> "$HOME/claude-arguments"
case "$*" in
    "plugin list --json")
        if [ -f "$HOME/claude-installed" ]; then
            echo '[{"id": "tightship@tightship", "version": "1.2.0", "installPath": INSTALL_PATH}]'
        else
            echo '[]'
        fi
        ;;
    "plugin install tightship@tightship"*)
        touch "$HOME/claude-installed"
        ;;
esac
exit 0
"#
    .replace("INSTALL_PATH", &claude_code_install_path())
}

const COMPANIONS: [&str; 3] = ["ponytail", "i-have-adhd", "humanizer"];
