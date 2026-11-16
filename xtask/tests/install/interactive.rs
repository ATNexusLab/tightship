//! The questions an interactive install asks, and a failure under iex.

use super::*;

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
                ("does not exist. Create it now?", "\r"),
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
        assert_eq!(
            toml_options["vault_folder"].as_str(),
            Some("~/ai-memory"),
            "{installer:?}: {toml_text}"
        );
        let json_text = fs::read_to_string(env.home_dir.join(".cursor/tightship/options.json"))
            .expect("read options.json");
        let json_options: serde_json::Value =
            serde_json::from_str(&json_text).expect("valid options.json");
        for option in ["contract", "memory", "status_line", "auto_update"] {
            assert_eq!(json_options[option], true, "{installer:?}: {json_text}");
        }
        assert_eq!(
            json_options["vault_folder"], "~/ai-memory",
            "{installer:?}: {json_text}"
        );
        assert!(
            env.home_dir.join("ai-memory/AGENTS.md").is_file(),
            "{installer:?}:
{screen}"
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
fn a_dash_on_the_vault_question_leaves_the_vault_unset() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agent", AGENT_VERSION);

        let from_dir = cursor_from_dir();
        let (screen, code) = env.run_in_terminal(
            installer,
            &installer.built(),
            &[
                "--harness",
                "cursor",
                "--contract",
                "true",
                "--memory",
                "true",
                "--status-line",
                "true",
                "--auto-update",
                "true",
                "--from-dir",
                &from_dir,
            ],
            &[
                ("Where is your vault?", "-\r"),
                ("Install with these choices?", "\r"),
            ],
        );

        assert_eq!(
            code, 0,
            "{installer:?}:
{screen}"
        );
        let options_text = fs::read_to_string(env.home_dir.join(".cursor/tightship/options.json"))
            .expect("read options.json");
        let options: serde_json::Value =
            serde_json::from_str(&options_text).expect("valid options.json");
        assert!(
            options.get("vault_folder").is_none(),
            "{installer:?}: {options_text}"
        );
        assert!(!env.home_dir.join("ai-memory").exists(), "{installer:?}");
    }
}
