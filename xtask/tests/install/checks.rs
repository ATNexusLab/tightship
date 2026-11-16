//! What the installers refuse or print before they install anything.

use super::*;

#[test]
fn installers_print_ascii_only() {
    for name in ["install.sh", "install.ps1"] {
        let text = fs::read_to_string(repo_root().join("installer").join(name)).expect(name);
        for (line, content) in text.lines().enumerate() {
            if let Some(column) = content.find(|character: char| !character.is_ascii()) {
                panic!(
                    "installer/{name}:{}:{}: a character outside ASCII reads garbled in a console \
                     outside UTF-8",
                    line + 1,
                    column + 1
                );
            }
        }
    }
}

#[test]
fn scenario_11_installer_rejects_unknown_harness() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();

        let output = env.run(installer, &["--harness", "unknown-harness", "--yes"]);
        assert_installer_fails_with(
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
        assert_installer_fails_with(
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
        assert_installer_fails_with(
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
        assert_installer_fails_with(
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
        assert_installer_fails_with(
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
        assert_installer_fails_with(
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
        assert_installer_fails_with(
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

        assert_installer_fails_with(
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
