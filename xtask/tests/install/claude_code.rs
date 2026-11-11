//! Installing the Claude Code plugin: the status line, the vault, and a rerun.

use super::*;

#[test]
fn a_rerun_updates_the_installed_claude_code_plugin_and_keeps_its_options() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script(
            "claude",
            &r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "claude 2.1.295"
    exit 0
fi
echo "$@" >> "$HOME/claude-arguments"
case "$*" in
    "plugin list --json")
        echo '[{"id": "tightship@tightship", "version": "1.1.1", "installPath": INSTALL_PATH}]'
        ;;
    "plugin configure tightship@tightship --json")
        printf '%s\n' '{"schema": {"contract": {"default": true}}, "inputs": {"contract": "false", "memory": "true", "status_line": "true", "vault_folder": "C:\\Users\\me \"work\""}}'
        ;;
    "plugin configure tightship@tightship --values-stdin")
        cat > "$HOME/claude-values"
        ;;
esac
exit 0
"#
            .replace("INSTALL_PATH", &claude_code_install_path()),
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
        for companion in COMPANIONS {
            assert!(
                !claude_arguments.contains(companion),
                "{installer:?}: {claude_arguments}"
            );
        }
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
        assert_claude_code_status_line(installer, &env);
    }
}

fn assert_claude_code_status_line(installer: Installer, env: &MockEnvironment) {
    let settings_text = fs::read_to_string(env.home_dir.join(".claude/settings.json"))
        .unwrap_or_else(|_| panic!("{installer:?}: Claude Code's settings.json was not written"));
    let settings: serde_json::Value =
        serde_json::from_str(&settings_text).expect("valid settings.json");
    let command = settings["statusLine"]["command"]
        .as_str()
        .unwrap_or_default();
    assert!(
        command.ends_with(" status-line render"),
        "{installer:?}: {settings_text}"
    );
}

#[test]
fn a_fresh_claude_code_install_applies_the_status_line_and_creates_the_vault() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("claude", &claude_that_installs());

        let from_dir = from_dir();
        let output = env.run(
            installer,
            &[
                "--harness",
                "claude-code",
                "--from-dir",
                &from_dir,
                "--vault-folder",
                "~/ai-memory",
                "--init-vault",
                "true",
                "--yes",
            ],
        );

        assert_exit(installer, &output, 0);
        let claude_arguments = fs::read_to_string(env.home_dir.join("claude-arguments"))
            .expect("read claude arguments");
        assert!(
            claude_arguments.contains("vault_folder=~/ai-memory"),
            "{installer:?}: {claude_arguments}"
        );
        assert_claude_code_status_line(installer, &env);
        assert!(
            env.home_dir.join("ai-memory/AGENTS.md").is_file(),
            "{installer:?}: the vault was not created"
        );
    }
}
