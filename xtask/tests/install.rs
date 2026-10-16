use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::TempDir;

struct MockEnvironment {
    _temp_dir: TempDir,
    bin_dir: PathBuf,
    home_dir: PathBuf,
}

impl MockEnvironment {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("a temporary directory");
        let bin_dir = temp_dir.path().join("bin");
        let home_dir = temp_dir.path().join("home");
        fs::create_dir_all(&bin_dir).expect("bin directory");
        fs::create_dir_all(&home_dir).expect("home directory");
        Self {
            _temp_dir: temp_dir,
            bin_dir,
            home_dir,
        }
    }

    fn isolated_path(&self) -> String {
        let system_paths = std::env::var("PATH").unwrap_or_default();
        let filtered: Vec<&str> = system_paths
            .split(':')
            .filter(|entry| {
                !entry.contains(".gemini")
                    && !entry.contains("claude")
                    && !entry.contains("codex")
                    && !entry.contains("mise")
                    && !entry.contains(".local/bin")
                    && !entry.contains("omarchy")
            })
            .collect();
        format!("{}:{}", self.bin_dir.display(), filtered.join(":"))
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
    }

    fn run_sh(&self, repo_root: &Path, args: &[&str]) -> std::process::Output {
        let install_sh = repo_root.join("install.sh");

        Command::new("sh")
            .arg(install_sh)
            .args(args)
            .current_dir(repo_root)
            .env("HOME", &self.home_dir)
            .env("USERPROFILE", &self.home_dir)
            .env("PATH", self.isolated_path())
            .stdin(Stdio::null())
            .output()
            .expect("install.sh executes")
    }

    #[allow(dead_code)]
    fn run_pwsh(&self, repo_root: &Path, args: &[&str]) -> std::process::Output {
        let install_ps1 = repo_root.join("install.ps1");

        Command::new("pwsh")
            .arg("-NoProfile")
            .arg("-File")
            .arg(install_ps1)
            .args(args)
            .current_dir(repo_root)
            .env("HOME", &self.home_dir)
            .env("USERPROFILE", &self.home_dir)
            .env("PATH", self.isolated_path())
            .stdin(Stdio::null())
            .output()
            .expect("install.ps1 executes")
    }
    fn run_sh_detached(&self, repo_root: &Path, args: &[&str]) -> std::process::Output {
        let install_sh = repo_root.join("install.sh");

        #[cfg(unix)]
        let mut cmd = Command::new("setsid");
        #[cfg(unix)]
        cmd.arg("sh").arg(install_sh).args(args);

        #[cfg(not(unix))]
        let mut cmd = Command::new("sh");
        #[cfg(not(unix))]
        cmd.arg(install_sh).args(args);

        cmd.current_dir(repo_root)
            .env("HOME", &self.home_dir)
            .env("USERPROFILE", &self.home_dir)
            .env("PATH", self.isolated_path())
            .stdin(Stdio::null())
            .output()
            .expect("install.sh executes")
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn scenario_10_interactive_and_flag_driven_install_antigravity_cli() {
    let root = repo_root();
    let env = MockEnvironment::new();

    // Mock agy CLI
    env.add_mock_script(
        "agy",
        r#"#!/bin/sh
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
"#,
    );

    // Mock curl for humanizer SKILL.md download
    env.add_mock_script(
        "curl",
        r#"#!/bin/sh
while [ $# -gt 0 ]; do
    if [ "$1" = "-o" ]; then
        dest="$2"
        mkdir -p "$(dirname "$dest")"
        echo "---\nname: humanizer\n---" > "$dest"
        exit 0
    fi
    shift
done
exit 0
"#,
    );

    let from_dir = root.join("dist/antigravity-cli");
    let output = env.run_sh(
        &root,
        &[
            "--harness",
            "antigravity-cli",
            "--contract",
            "strict",
            "--memory",
            "true",
            "--status-line",
            "true",
            "--vault-folder",
            "~/ai-memory",
            "--from-dir",
            from_dir.to_str().expect("valid utf-8 path"),
            "--yes",
        ],
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr:\n{stderr}\nstdout:\n{stdout}"
    );
    assert!(stdout.contains("Installed tightship for Antigravity CLI."));

    // Check ponytail and i-have-adhd
    assert!(
        env.home_dir
            .join(".gemini/config/plugins/ponytail")
            .is_dir(),
        "ponytail plugin installed"
    );
    assert!(
        env.home_dir
            .join(".gemini/config/plugins/i-have-adhd")
            .is_dir(),
        "i-have-adhd plugin installed"
    );

    // Check humanizer
    let humanizer_skill = env
        .home_dir
        .join(".gemini/config/skills/humanizer/SKILL.md");
    assert!(humanizer_skill.is_file(), "humanizer SKILL.md installed");

    // Check tightship plugin
    assert!(
        env.home_dir
            .join(".gemini/config/plugins/tightship/plugin.json")
            .is_file(),
        "tightship plugin installed"
    );

    // Check options.toml
    let options_file = env
        .home_dir
        .join(".gemini/config/plugin_data/tightship/options.toml");
    assert!(options_file.is_file(), "options.toml installed");
    let options_text = fs::read_to_string(&options_file).expect("read options.toml");
    assert!(options_text.contains("contract = true"));
    assert!(options_text.contains("memory = true"));
    assert!(options_text.contains("status_line = true"));
    assert!(options_text.contains(r#"vault_folder = "~/ai-memory""#));

    // Check settings.json statusLine
    let settings_file = env.home_dir.join(".gemini/antigravity-cli/settings.json");
    assert!(settings_file.is_file(), "settings.json created");
    let settings_text = fs::read_to_string(&settings_file).expect("read settings.json");
    assert!(settings_text.contains("statusLine"));
}

#[test]
fn scenario_11_installer_rejects_unknown_harness() {
    let root = repo_root();
    let env = MockEnvironment::new();

    let output = env.run_sh(&root, &["--harness", "unknown-harness", "--yes"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "error: unknown harness unknown-harness; expected claude-code, antigravity-cli, or all"
        ),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn scenario_11_installer_rejects_outdated_agy() {
    let root = repo_root();
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

    let output = env.run_sh(&root, &["--harness", "antigravity-cli", "--yes"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: agy 1.4.0 or later is required; found 1.3.0"),
        "unexpected stderr: {stderr}"
    );

    // Check no files were created
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

#[test]
fn scenario_11_installer_rejects_missing_agy() {
    let root = repo_root();
    let env = MockEnvironment::new();

    let output = env.run_sh(&root, &["--harness", "antigravity-cli", "--yes"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(1),
        "code={:?}, stderr={stderr}, stdout={stdout}",
        output.status.code()
    );
    assert!(
        stderr.contains("error: agy 1.4.0 or later is required; found not installed"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn scenario_11_installer_rejects_outdated_claude() {
    let root = repo_root();
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

    let output = env.run_sh(&root, &["--harness", "claude-code", "--yes"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: claude 2.1.295 or later is required; found 2.0.0"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn scenario_11_installer_rejects_missing_claude() {
    let root = repo_root();
    let env = MockEnvironment::new();

    let output = env.run_sh(&root, &["--harness", "claude-code", "--yes"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: claude 2.1.295 or later is required; found not installed"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn non_interactive_without_yes_fails_with_standard_input_error() {
    let root = repo_root();
    let env = MockEnvironment::new();

    let output = env.run_sh_detached(&root, &["--harness", "antigravity-cli"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "error: standard input is not a terminal; pass --yes or provide required flags"
        ),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn existing_options_are_preserved_on_update() {
    let root = repo_root();
    let env = MockEnvironment::new();

    // Pre-create options.toml with contract = false
    let options_dir = env.home_dir.join(".gemini/config/plugin_data/tightship");
    fs::create_dir_all(&options_dir).expect("options dir");
    fs::write(
        options_dir.join("options.toml"),
        "contract = false\nmemory = true\nstatus_line = true\n",
    )
    .expect("write options");

    env.add_mock_script(
        "agy",
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "agy 1.4.0"
    exit 0
fi
exit 0
"#,
    );

    env.add_mock_script(
        "curl",
        r#"#!/bin/sh
while [ $# -gt 0 ]; do
    if [ "$1" = "-o" ]; then
        dest="$2"
        mkdir -p "$(dirname "$dest")"
        echo "---\nname: humanizer\n---" > "$dest"
        exit 0
    fi
    shift
done
exit 0
"#,
    );

    let from_dir = root.join("dist/antigravity-cli");
    let output = env.run_sh(
        &root,
        &[
            "--harness",
            "antigravity-cli",
            "--from-dir",
            from_dir.to_str().expect("valid utf-8 path"),
            "--yes",
        ],
    );

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let options_text =
        fs::read_to_string(options_dir.join("options.toml")).expect("read options.toml");
    assert!(
        options_text.contains("contract = false"),
        "contract = false was preserved: {options_text}"
    );
}

#[test]
fn powershell_installer_scenarios_if_pwsh_available() {
    let pwsh_check = Command::new("pwsh").arg("--version").output();
    if pwsh_check.is_err() {
        eprintln!("pwsh is not available on this platform; skipping PowerShell test");
        return;
    }

    let root = repo_root();
    let env = MockEnvironment::new();

    let output = env.run_pwsh(&root, &["-Harness", "unknown-harness", "-Yes"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(
        "error: unknown harness unknown-harness; expected claude-code, antigravity-cli, or all"
    ));
}
