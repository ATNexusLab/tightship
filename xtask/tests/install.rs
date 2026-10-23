use std::ffi::OsString;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, System, ZipWriter};

const RELEASE_TAG: &str = "v1.0.0";

const REMOVED_VARIABLES: [&str; 6] = [
    "TIGHTSHIP_ROOT",
    "TIGHTSHIP_DATA",
    "TIGHTSHIP_SETTINGS",
    "PLUGIN_ROOT",
    "PLUGIN_DATA",
    "AGY_CONFIG_DIR",
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
    case "$target" in
        *ponytail)
            mkdir -p "$HOME/.gemini/config/plugins/ponytail"
            ;;
        *i-have-adhd)
            mkdir -p "$HOME/.gemini/config/plugins/i-have-adhd"
            ;;
        *)
            mkdir -p "$HOME/.gemini/config/plugins/tightship"
            if [ -d "$target" ]; then
                cp -R "$target"/* "$HOME/.gemini/config/plugins/tightship/"
            fi
            ;;
    esac
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

fn download_cache_name(url: &str) -> String {
    url.replace("https://", "").replace('/', "_")
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
        fs::write(downloads_dir.join("SKILL.md"), "humanizer\n").expect("the humanizer download");
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
            while let Ok(read @ 1..) = reader.read(&mut buffer) {
                if sender.send(buffer[..read].to_vec()).is_err() {
                    break;
                }
            }
        });

        let mut screen = String::new();
        let mut consumed = 0;
        let mut answered = 0;
        let mut cursor_replies = 0;
        let started = Instant::now();
        let status = loop {
            assert!(
                started.elapsed() < Duration::from_secs(90),
                "{installer:?}: stuck after {answered} answers:
{screen}"
            );
            if let Ok(chunk) = receiver.recv_timeout(Duration::from_millis(50)) {
                screen.push_str(&String::from_utf8_lossy(&chunk));
            }
            while cursor_replies < screen.matches("\x1b[6n").count() {
                writer.write_all(b"\x1b[1;1R").expect("a cursor reply");
                cursor_replies += 1;
            }
            if let Some((prompt, keys)) = answers.get(answered)
                && let Some(position) = screen[consumed..].find(prompt)
            {
                consumed += position + prompt.len();
                std::thread::sleep(Duration::from_millis(150));
                writer.write_all(keys.as_bytes()).expect("the keys");
                writer.flush().expect("the keys are sent");
                answered += 1;
            }
            if let Some(status) = child.try_wait().expect("the installer's status") {
                while let Ok(chunk) = receiver.recv_timeout(Duration::from_millis(300)) {
                    screen.push_str(&String::from_utf8_lossy(&chunk));
                }
                break status;
            }
        };
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

fn companion_zip(plugin: &str) -> Vec<u8> {
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

    let temp = tempfile::tempdir().expect("a temporary directory");
    let root_name = format!("{plugin}-main");
    let folder = temp.path().join(&root_name);
    let manifest = folder.join(".claude-plugin/plugin.json");
    fs::create_dir_all(manifest.parent().expect("manifest parent")).expect("companion folders");
    fs::write(&manifest, format!(r#"{{"name":"{plugin}"}}"#)).expect("companion manifest");

    let mut files = Vec::new();
    walk(&folder, &format!("{root_name}/"), &mut files);
    files.sort();

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for file in &files {
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .system(System::Unix)
            .unix_permissions(0o644);
        writer.start_file(file, options).expect("a zip entry");
        writer
            .write_all(&fs::read(temp.path().join(file)).expect("a companion file"))
            .expect("the zip entry");
    }
    writer.finish().expect("a zip").into_inner()
}

#[test]
#[cfg(unix)]
fn companion_zip_unpacks_like_github_main_branch() {
    let bytes = companion_zip("ponytail");
    let temp = tempfile::tempdir().expect("a temporary directory");
    let zip_path = temp.path().join("ponytail.zip");
    fs::write(&zip_path, bytes).expect("write zip");
    let status = Command::new("unzip")
        .arg("-q")
        .arg(&zip_path)
        .arg("-d")
        .arg(temp.path())
        .status()
        .expect("unzip runs");
    assert!(status.success(), "unzip failed: {status:?}");
    assert!(
        temp.path()
            .join("ponytail-main/.claude-plugin/plugin.json")
            .is_file()
    );
}

fn seed_companion_downloads(downloads: &Path) {
    for (plugin, repo) in [
        ("ponytail", "DietrichGebert/ponytail"),
        ("i-have-adhd", "ayghri/i-have-adhd"),
        ("humanizer", "blader/humanizer"),
    ] {
        let url = format!("https://github.com/{repo}/archive/refs/heads/main.zip");
        let key = download_cache_name(&url);
        fs::write(downloads.join(key), companion_zip(plugin)).expect("companion zip");
    }
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

fn assert_fails_with(installer: Installer, output: &Output, message: &str) {
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

#[test]
fn scenario_10_interactive_and_flag_driven_install_antigravity_cli() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);

        let from_dir = from_dir();
        let output = env.run(
            installer,
            &[
                "--harness",
                "antigravity-cli",
                "--contract",
                "true",
                "--memory",
                "true",
                "--status-line",
                "true",
                "--vault-folder",
                "~/ai-memory",
                "--from-dir",
                &from_dir,
                "--yes",
            ],
        );

        assert_exit(installer, &output, 0);
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("Installed tightship for Antigravity CLI.")
        );
        for stream in [&output.stdout, &output.stderr] {
            assert!(
                !stream.contains(&0x1b),
                "{installer:?}: color outside a terminal"
            );
        }
        let plugins = env.home_dir.join(".gemini/config/plugins");
        assert!(plugins.join("ponytail").is_dir(), "{installer:?}");
        assert!(plugins.join("i-have-adhd").is_dir(), "{installer:?}");
        assert!(
            plugins.join("tightship/plugin.json").is_file(),
            "{installer:?}"
        );
        let hooks: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(plugins.join("tightship/hooks.json")).expect("read hooks.json"),
        )
        .expect("valid hooks.json");
        let launcher = if cfg!(windows) {
            r"bin\tightship "
        } else {
            "bin/tightship "
        };
        assert!(
            hooks["tightship"]["SessionStart"][0]["command"]
                .as_str()
                .is_some_and(|command| command.starts_with(launcher)),
            "{installer:?}: {hooks}"
        );
        assert!(
            !plugins.join("tightship/hooks.windows.json").exists(),
            "{installer:?}"
        );
        assert!(
            env.home_dir
                .join(".gemini/config/skills/humanizer/SKILL.md")
                .is_file(),
            "{installer:?}"
        );

        let options_text = fs::read_to_string(
            env.home_dir
                .join(".gemini/config/plugin_data/tightship/options.toml"),
        )
        .expect("read options.toml");
        assert!(options_text.contains("contract = true"), "{installer:?}");
        assert!(options_text.contains("memory = true"), "{installer:?}");
        assert!(options_text.contains("status_line = true"), "{installer:?}");
        assert!(options_text.contains("auto_update = true"), "{installer:?}");
        assert!(
            options_text.contains(r#"vault_folder = "~/ai-memory""#),
            "{installer:?}"
        );

        let settings_text =
            fs::read_to_string(env.home_dir.join(".gemini/antigravity-cli/settings.json"))
                .expect("read settings.json");
        assert!(settings_text.contains("statusLine"), "{installer:?}");
    }
}

#[test]
fn scenario_11_installer_rejects_unknown_harness() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();

        let output = env.run(installer, &["--harness", "unknown-harness", "--yes"]);
        assert_fails_with(
            installer,
            &output,
            "error: unknown harness unknown-harness; expected antigravity-cli, claude-code, cursor, or all",
        );
    }
}

#[test]
fn scenario_11_installer_rejects_an_option_that_is_not_a_boolean() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);

        let from_dir = from_dir();
        let output = env.run(
            installer,
            &[
                "--harness",
                "antigravity-cli",
                "--memory",
                "flase",
                "--from-dir",
                &from_dir,
                "--yes",
            ],
        );
        assert_fails_with(
            installer,
            &output,
            &format!(
                "error: {} expects true or false; got \"flase\"",
                installer.flag("memory")
            ),
        );
    }
}

#[test]
fn scenario_11_installer_rejects_outdated_agy() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script(
            "agy",
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "agy 1.3.0"
    exit 0
fi
exit 0
"#,
        );

        let output = env.run(installer, &["--harness", "antigravity-cli", "--yes"]);
        assert_fails_with(
            installer,
            &output,
            "error: agy 1.4.0 or later is required; found 1.3.0",
        );
        assert!(
            !env.home_dir
                .join(".gemini/config/plugins/tightship")
                .exists()
        );
        assert!(
            !env.home_dir
                .join(".gemini/antigravity-cli/settings.json")
                .exists()
        );
    }
}

#[test]
fn scenario_11_installer_rejects_missing_agy() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();

        let output = env.run(installer, &["--harness", "antigravity-cli", "--yes"]);
        assert_fails_with(
            installer,
            &output,
            "error: agy 1.4.0 or later is required; found not installed",
        );
    }
}

#[test]
fn scenario_11_installer_rejects_outdated_claude() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script(
            "claude",
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "claude 2.0.0"
    exit 0
fi
exit 0
"#,
        );

        let output = env.run(installer, &["--harness", "claude-code", "--yes"]);
        assert_fails_with(
            installer,
            &output,
            "error: claude 2.1.295 or later is required; found 2.0.0",
        );
    }
}

#[test]
fn scenario_11_installer_rejects_missing_claude() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();

        let output = env.run(installer, &["--harness", "claude-code", "--yes"]);
        assert_fails_with(
            installer,
            &output,
            "error: claude 2.1.295 or later is required; found not installed",
        );
    }
}

#[test]
fn non_interactive_without_yes_fails_with_standard_input_error() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();

        let output = env.run(installer, &["--harness", "antigravity-cli"]);
        assert_fails_with(
            installer,
            &output,
            &format!(
                "error: standard input is not a terminal; pass {} or provide required flags",
                installer.flag("yes")
            ),
        );
    }
}

#[test]
fn an_installer_outside_a_release_downloads_nothing() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);

        let output = env.run(installer, &["--harness", "antigravity-cli", "--yes"]);
        assert_fails_with(
            installer,
            &output,
            &format!(
                "error: this {file} belongs to no release; run https://github.com/ATNexusLab/tightship/releases/latest/download/{file}, or pass {} <folder>",
                installer.flag("from-dir"),
                file = installer.file()
            ),
        );
        assert!(!env.home_dir.join(".gemini/config/plugins").exists());
    }
}

#[test]
fn a_released_installer_installs_the_zip_whose_sha256_it_carries() {
    let (zip, sha256) = release_zip("antigravity-cli");
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);
        fs::write(
            env.downloads_dir.join("tightship-antigravity-cli.zip"),
            &zip,
        )
        .expect("the release download");

        let released = env.released(installer, &[&sha256]);
        let output = env.run_script(
            installer,
            &released,
            &["--harness", "antigravity-cli", "--yes"],
        );

        assert_exit(installer, &output, 0);
        assert!(
            env.home_dir
                .join(".gemini/config/plugins/tightship/plugin.json")
                .is_file(),
            "{installer:?}"
        );
    }
}

#[test]
fn a_released_installer_refuses_a_zip_whose_sha256_differs() {
    let (zip, sha256) = release_zip("antigravity-cli");
    let published = "0".repeat(64);
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);
        fs::write(
            env.downloads_dir.join("tightship-antigravity-cli.zip"),
            &zip,
        )
        .expect("the release download");

        let released = env.released(installer, &[&published]);
        let output = env.run_script(
            installer,
            &released,
            &["--harness", "antigravity-cli", "--yes"],
        );

        assert_fails_with(
            installer,
            &output,
            &format!(
                "error: tightship-antigravity-cli.zip has sha256 {sha256}; {RELEASE_TAG} published {published}"
            ),
        );
        assert!(
            !env.home_dir
                .join(".gemini/config/plugins/tightship")
                .exists(),
            "{installer:?}"
        );
    }
}

#[test]
fn piped_installer_never_installs_a_dist_folder_from_the_current_directory() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        let agy_log = env.home_dir.join("agy.log");
        env.add_mock_script(
            "agy",
            &format!(
                r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "agy 1.4.0"
    exit 0
fi
echo "$@" >> "{}"
exit 0
"#,
                agy_log.display()
            ),
        );

        let untrusted_clone = tempfile::tempdir().expect("a temporary directory");
        let planted = untrusted_clone.path().join("dist/antigravity-cli");
        fs::create_dir_all(planted.join("bin")).expect("planted dist");
        fs::write(planted.join("plugin.json"), "{}").expect("planted plugin.json");

        let released = env.released(installer, &[&"0".repeat(64)]);
        let output = env.run_piped(
            installer,
            &released,
            untrusted_clone.path(),
            &["--harness", "antigravity-cli", "--yes"],
        );

        assert_fails_with(
            installer,
            &output,
            "error: failed to download tightship-antigravity-cli.zip",
        );
        let agy_calls = fs::read_to_string(&agy_log).unwrap_or_default();
        assert!(
            !agy_calls.contains("dist/antigravity-cli"),
            "{installer:?}: the planted dist folder was installed: {agy_calls}"
        );
    }
}

#[test]
fn a_failed_plugin_install_stops_the_installer_and_names_the_step() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script(
            "agy",
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "agy 1.4.0"
    exit 0
fi
case "$3" in
    *ponytail|*i-have-adhd) exit 0 ;;
esac
exit 3
"#,
        );

        let from_dir = from_dir();
        let output = env.run(
            installer,
            &[
                "--harness",
                "antigravity-cli",
                "--from-dir",
                &from_dir,
                "--yes",
            ],
        );

        assert_fails_with(
            installer,
            &output,
            "error: Installing tightship for Antigravity CLI failed",
        );
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains("Installed tightship"),
            "{installer:?}"
        );
    }
}

#[test]
fn existing_options_are_preserved_on_update() {
    let (antigravity_zip, antigravity_sha256) = release_zip("antigravity-cli");
    let (cursor_zip, cursor_sha256) = release_zip("cursor");
    let vault_folder = r#"C:\Users\me "work"\ai-memory"#;
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        let antigravity_options = env
            .home_dir
            .join(".gemini/config/plugin_data/tightship/options.toml");
        let cursor_options = env.home_dir.join(".cursor/tightship/options.json");
        fs::create_dir_all(antigravity_options.parent().expect("a parent")).expect("options dir");
        fs::create_dir_all(cursor_options.parent().expect("a parent")).expect("options dir");
        fs::write(
            &antigravity_options,
            "contract = false\nmemory = true\nstatus_line = true\nauto_update = false\nvault_folder = \"C:\\\\Users\\\\me \\\"work\\\"\\\\ai-memory\"\n",
        )
        .expect("write options.toml");
        fs::write(
            &cursor_options,
            serde_json::json!({
                "contract": false,
                "memory": true,
                "status_line": true,
                "auto_update": false,
                "vault_folder": vault_folder,
            })
            .to_string(),
        )
        .expect("write options.json");
        fs::write(
            env.downloads_dir.join("tightship-antigravity-cli.zip"),
            &antigravity_zip,
        )
        .expect("the Antigravity CLI download");
        fs::write(env.downloads_dir.join("tightship-cursor.zip"), &cursor_zip)
            .expect("the Cursor download");
        seed_companion_downloads(&env.downloads_dir);
        env.add_mock_script("agy", AGY_THAT_INSTALLS);
        env.add_mock_script("agent", AGENT_VERSION);
        env.add_mock_script(
            "claude",
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "claude 2.1.295"
    exit 0
fi
echo "$@" >> "$HOME/claude-arguments"
exit 0
"#,
        );

        let released = env.released(installer, &[&antigravity_sha256, &cursor_sha256]);
        let output = env.run_script(installer, &released, &["--harness", "all", "--yes"]);

        assert_exit(installer, &output, 0);
        let toml_text = fs::read_to_string(&antigravity_options).expect("read options.toml");
        let toml_options: toml::Table = toml_text.parse().unwrap_or_else(|error| {
            panic!("{installer:?}: options.toml is valid TOML ({error}): {toml_text}")
        });
        assert_eq!(
            toml_options["contract"].as_bool(),
            Some(false),
            "{installer:?}"
        );
        assert_eq!(
            toml_options["auto_update"].as_bool(),
            Some(false),
            "{installer:?}"
        );
        assert_eq!(
            toml_options["vault_folder"].as_str(),
            Some(vault_folder),
            "{installer:?}"
        );
        let json_text = fs::read_to_string(&cursor_options).expect("read options.json");
        let json_options: serde_json::Value =
            serde_json::from_str(&json_text).expect("valid options.json");
        assert_eq!(json_options["contract"], false, "{installer:?}");
        assert_eq!(json_options["auto_update"], false, "{installer:?}");
        assert_eq!(json_options["vault_folder"], vault_folder, "{installer:?}");
        let claude_arguments = fs::read_to_string(env.home_dir.join("claude-arguments"))
            .expect("read claude arguments");
        for kept in ["contract=false", "auto_update=false"] {
            assert!(
                claude_arguments.contains(kept),
                "{installer:?}: {claude_arguments}"
            );
        }
        let leftovers: Vec<_> = fs::read_dir(&env.system_temp_dir)
            .expect("read the system temp folder")
            .collect();
        assert!(leftovers.is_empty(), "{installer:?}: {leftovers:?}");
    }
}

#[test]
fn a_rerun_updates_the_installed_claude_code_plugin_and_keeps_its_options() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script(
            "claude",
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "claude 2.1.295"
    exit 0
fi
echo "$@" >> "$HOME/claude-arguments"
case "$*" in
    "plugin list --json")
        echo '[{"id": "tightship@tightship", "version": "1.1.1"}]'
        ;;
    "plugin configure tightship@tightship --json")
        printf '%s\n' '{"schema": {"contract": {"default": true}}, "inputs": {"contract": "false", "memory": "true", "status_line": "true", "vault_folder": "C:\\Users\\me \"work\""}}'
        ;;
    "plugin configure tightship@tightship --values-stdin")
        cat > "$HOME/claude-values"
        ;;
esac
exit 0
"#,
        );

        let from_dir = from_dir();
        let output = env.run(
            installer,
            &["--harness", "claude-code", "--from-dir", &from_dir, "--yes"],
        );

        assert_exit(installer, &output, 0);
        let claude_arguments = fs::read_to_string(env.home_dir.join("claude-arguments"))
            .expect("read claude arguments");
        for called in [
            "plugin marketplace update ponytail",
            "plugin update ponytail@ponytail",
            "plugin marketplace update tightship",
            "plugin update tightship@tightship",
        ] {
            assert!(
                claude_arguments.lines().any(|line| line == called),
                "{installer:?}: {called} was not called: {claude_arguments}"
            );
        }
        assert!(
            !claude_arguments.contains("plugin install"),
            "{installer:?}: {claude_arguments}"
        );
        let values: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(env.home_dir.join("claude-values")).expect("read the values"),
        )
        .expect("JSON values");
        assert_eq!(
            values,
            serde_json::json!({
                "contract": "false",
                "memory": "true",
                "status_line": "true",
                "auto_update": "true",
                "vault_folder": r#"C:\Users\me "work""#,
            }),
            "{installer:?}"
        );
    }
}

#[test]
fn enter_on_every_question_installs_the_checked_harnesses_with_the_defaults() {
    let (antigravity_zip, antigravity_sha256) = release_zip("antigravity-cli");
    let (cursor_zip, cursor_sha256) = release_zip("cursor");
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        fs::write(
            env.downloads_dir.join("tightship-antigravity-cli.zip"),
            &antigravity_zip,
        )
        .expect("the Antigravity CLI download");
        fs::write(env.downloads_dir.join("tightship-cursor.zip"), &cursor_zip)
            .expect("the Cursor download");
        seed_companion_downloads(&env.downloads_dir);
        env.add_mock_script("agy", AGY_THAT_INSTALLS);
        env.add_mock_script("agent", AGENT_VERSION);
        env.add_mock_script(
            "claude",
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "claude 2.1.295"
    exit 0
fi
echo "$@" >> "$HOME/claude-arguments"
exit 0
"#,
        );

        let released = env.released(installer, &[&antigravity_sha256, &cursor_sha256]);
        let (screen, code) = env.run_in_terminal(
            installer,
            &released,
            &[],
            &[
                ("Where should tightship be installed?", "\x1b[B \r"),
                ("Load the working contract in every session?", "\r"),
                ("Turn on the memory layer?", "\r"),
                ("Show tightship's status line?", "\r"),
                ("Keep tightship updated automatically?", "\r"),
                ("Where is your vault?", "\r"),
                ("Install with these choices?", "\r"),
            ],
        );

        assert_eq!(
            code, 0,
            "{installer:?}:
{screen}"
        );
        assert!(
            screen.contains("Harnesses"),
            "{installer:?}:
{screen}"
        );
        assert!(
            env.home_dir
                .join(".gemini/config/plugins/tightship/plugin.json")
                .is_file(),
            "{installer:?}:
{screen}"
        );
        assert!(
            env.home_dir
                .join(".cursor/plugins/local/tightship/commands/spec.md")
                .is_file(),
            "{installer:?}:
{screen}"
        );
        let claude_arguments =
            fs::read_to_string(env.home_dir.join("claude-arguments")).unwrap_or_default();
        assert!(
            !claude_arguments.contains("plugin install"),
            "{installer:?}: {claude_arguments}"
        );
        let toml_text = fs::read_to_string(
            env.home_dir
                .join(".gemini/config/plugin_data/tightship/options.toml"),
        )
        .expect("read options.toml");
        let toml_options: toml::Table = toml_text.parse().expect("valid options.toml");
        for option in ["contract", "memory", "status_line", "auto_update"] {
            assert_eq!(
                toml_options[option].as_bool(),
                Some(true),
                "{installer:?}: {toml_text}"
            );
        }
        assert!(
            !toml_options.contains_key("vault_folder"),
            "{installer:?}: {toml_text}"
        );
        let json_text = fs::read_to_string(env.home_dir.join(".cursor/tightship/options.json"))
            .expect("read options.json");
        let json_options: serde_json::Value =
            serde_json::from_str(&json_text).expect("valid options.json");
        for option in ["contract", "memory", "status_line", "auto_update"] {
            assert_eq!(json_options[option], true, "{installer:?}: {json_text}");
        }
    }
}

#[test]
fn both_installers_take_help_and_a_harness_name_in_any_case() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);

        let help = env.run_script(installer, &installer.built(), &["-h"]);
        assert_exit(installer, &help, 0);
        assert!(
            String::from_utf8_lossy(&help.stdout)
                .starts_with(&format!("Usage: {}", installer.file())),
            "{installer:?}: {}",
            String::from_utf8_lossy(&help.stdout)
        );

        let from_dir = from_dir();
        let output = env.run(
            installer,
            &[
                "--harness",
                "Antigravity-CLI",
                "--from-dir",
                &from_dir,
                "--yes",
            ],
        );
        assert_exit(installer, &output, 0);
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("Installed tightship for Antigravity CLI."),
            "{installer:?}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn init_vault_expands_the_tilde() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);
        let working_dir = tempfile::tempdir().expect("a temporary directory");

        let from_dir = from_dir();
        let output = env.run_piped(
            installer,
            &installer.built(),
            working_dir.path(),
            &[
                "--harness",
                "antigravity-cli",
                "--vault-folder",
                "~/v",
                "--init-vault",
                "true",
                "--from-dir",
                &from_dir,
                "--yes",
            ],
        );

        assert_exit(installer, &output, 0);
        assert!(env.home_dir.join("v").is_dir(), "{installer:?}");
        assert_eq!(
            fs::read_dir(working_dir.path())
                .expect("read the working folder")
                .count(),
            0,
            "{installer:?}"
        );
    }
}

#[test]
fn a_failed_install_through_iex_leaves_the_session_open() {
    let env = MockEnvironment::new();

    let output = env
        .command("pwsh", &repo_root())
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Invoke-Expression (Get-Content -Raw {}); \"still here $ErrorActionPreference\"",
                quoted(&Installer::Pwsh.built().to_string_lossy())
            ),
        ])
        .output()
        .expect("pwsh executes");

    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "error: standard input is not a terminal; pass -Yes or provide required flags"
        ),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim_end(),
        "still here Continue"
    );
}

#[test]
fn scenario_12_interactive_and_flag_driven_install_cursor() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        seed_companion_downloads(&env.downloads_dir);
        env.add_mock_script("agent", AGENT_VERSION);
        let plugins = env.home_dir.join(".cursor/plugins/local");
        let removed_upstream = [
            plugins.join("tightship/skills/removed/SKILL.md"),
            plugins.join("ponytail/skills/removed/SKILL.md"),
        ];
        for stale in &removed_upstream {
            fs::create_dir_all(stale.parent().expect("a parent")).expect("stale folder");
            fs::write(stale, "stale\n").expect("stale skill");
        }

        let from_dir = cursor_from_dir();
        let output = env.run(
            installer,
            &[
                "--harness",
                "cursor",
                "--contract",
                "true",
                "--memory",
                "true",
                "--status-line",
                "true",
                "--vault-folder",
                "~/ai-memory",
                "--from-dir",
                &from_dir,
                "--yes",
            ],
        );

        assert_exit(installer, &output, 0);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Installed tightship for Cursor."),
            "{installer:?}"
        );
        for stale in &removed_upstream {
            assert!(
                !stale.exists(),
                "{installer:?}: {} survived",
                stale.display()
            );
        }
        for companion in ["ponytail", "i-have-adhd", "humanizer"] {
            assert!(
                plugins
                    .join(companion)
                    .join(".claude-plugin/plugin.json")
                    .is_file(),
                "{installer:?}: companion {companion} missing"
            );
        }
        assert!(
            plugins.join("tightship/commands/spec.md").is_file(),
            "{installer:?}: tightship plugin tree missing under {}",
            plugins.join("tightship").display()
        );
        let options_text = fs::read_to_string(env.home_dir.join(".cursor/tightship/options.json"))
            .expect("read options.json");
        let options: serde_json::Value =
            serde_json::from_str(&options_text).expect("valid options.json");
        assert_eq!(options["contract"], true, "{installer:?}");
        assert_eq!(options["memory"], true, "{installer:?}");
        assert_eq!(options["status_line"], true, "{installer:?}");
        assert_eq!(options["auto_update"], true, "{installer:?}");
        assert_eq!(options["vault_folder"], "~/ai-memory", "{installer:?}");

        let settings_text =
            fs::read_to_string(env.home_dir.join(".cursor/cli-config.json")).expect("cli-config");
        assert!(settings_text.contains("statusLine"), "{installer:?}");
    }
}
