use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::TempDir;

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

const CURL_THAT_WRITES: &str = r#"#!/bin/sh
while [ $# -gt 0 ]; do
    if [ "$1" = "-o" ]; then
        mkdir -p "$(dirname "$2")"
        echo "humanizer" > "$2"
        exit 0
    fi
    shift
done
exit 0
"#;

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
    fn run_sh_piped(
        &self,
        repo_root: &Path,
        working_dir: &Path,
        args: &[&str],
    ) -> std::process::Output {
        use std::io::Write;

        let script = fs::read(repo_root.join("install.sh")).expect("read install.sh");
        let mut child = Command::new("sh")
            .arg("-s")
            .arg("--")
            .args(args)
            .current_dir(working_dir)
            .env("HOME", &self.home_dir)
            .env("USERPROFILE", &self.home_dir)
            .env("PATH", self.isolated_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("sh executes");
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(&script)
            .expect("pipe install.sh");
        child.wait_with_output().expect("install.sh finishes")
    }

    fn run_sh_detached(&self, repo_root: &Path, args: &[&str]) -> std::process::Output {
        let install_sh = repo_root.join("install.sh");

        let mut cmd = Command::new("sh");
        cmd.arg(install_sh).args(args);

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                cmd.pre_exec(|| {
                    if libc::setsid() == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }

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

    env.add_mock_script("agy", AGY_THAT_INSTALLS);

    env.add_mock_script("curl", CURL_THAT_WRITES);

    let from_dir = root.join("dist/antigravity-cli");
    let output = env.run_sh(
        &root,
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
fn scenario_11_installer_rejects_an_option_that_is_not_a_boolean() {
    let root = repo_root();
    let env = MockEnvironment::new();
    env.add_mock_script("agy", AGY_THAT_INSTALLS);

    let output = env.run_sh(
        &root,
        &["--harness", "antigravity-cli", "--memory", "flase", "--yes"],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: --memory expects true or false; got \"flase\""),
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
fn piped_installer_never_installs_a_dist_folder_from_the_current_directory() {
    let root = repo_root();
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
    env.add_mock_script(
        "curl",
        r#"#!/bin/sh
while [ $# -gt 0 ]; do
    if [ "$1" = "-o" ]; then
        case "$2" in
            *.zip) exit 22 ;;
        esac
        mkdir -p "$(dirname "$2")"
        echo "humanizer" > "$2"
        exit 0
    fi
    shift
done
exit 0
"#,
    );

    let untrusted_clone = tempfile::tempdir().expect("a temporary directory");
    let planted = untrusted_clone.path().join("dist/antigravity-cli");
    fs::create_dir_all(planted.join("bin")).expect("planted dist");
    fs::write(planted.join("plugin.json"), "{}").expect("planted plugin.json");

    let output = env.run_sh_piped(
        &root,
        untrusted_clone.path(),
        &["--harness", "antigravity-cli", "--yes"],
    );

    assert_ne!(
        output.status.code(),
        Some(0),
        "the release download was attempted and failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let agy_calls = fs::read_to_string(&agy_log).unwrap_or_default();
    assert!(
        !agy_calls.contains("dist/antigravity-cli"),
        "the planted dist folder was installed: {agy_calls}"
    );
}

#[test]
fn a_failed_plugin_install_stops_the_installer_and_names_the_step() {
    let root = repo_root();
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
    env.add_mock_script("curl", CURL_THAT_WRITES);

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

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("error: Installing tightship for Antigravity CLI failed")
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Installed tightship"));
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
        "contract = false\nmemory = true\nstatus_line = true\nvault_folder = \"C:\\\\Users\\\\me \\\"work\\\"\\\\ai-memory\"\n",
    )
    .expect("write options");

    env.add_mock_script("agy", AGY_THAT_INSTALLS);
    env.add_mock_script("curl", CURL_THAT_WRITES);

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
    let options: toml::Table = options_text
        .parse()
        .unwrap_or_else(|error| panic!("options.toml is valid TOML ({error}): {options_text}"));
    assert_eq!(options["contract"].as_bool(), Some(false));
    assert_eq!(
        options["vault_folder"].as_str(),
        Some(r#"C:\Users\me "work"\ai-memory"#)
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

#[test]
fn both_installers_install_every_companion_the_harness_declares() {
    let root = repo_root();
    let harness: toml::Table =
        fs::read_to_string(root.join("harnesses/antigravity-cli/harness.toml"))
            .expect("read harness.toml")
            .parse()
            .expect("harness.toml is valid TOML");
    let companions = harness["companions"].as_array().expect("companions");
    for installer in ["install.sh", "install.ps1"] {
        let script = fs::read_to_string(root.join(installer)).expect("read installer");
        for companion in companions {
            let marketplace = companion["marketplace"].as_str().expect("marketplace");
            let repository = marketplace
                .strip_prefix("https://github.com/")
                .expect("a GitHub marketplace");
            assert!(
                script.contains(repository),
                "{installer} does not install {repository}"
            );
        }
    }
}
