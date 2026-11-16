//! Installing the Cursor plugin.

use super::*;

#[test]
fn scenario_12_interactive_and_flag_driven_install_cursor() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agent", AGENT_VERSION);
        let plugins = env.home_dir.join(".cursor/plugins/local");
        let removed_upstream = [plugins.join("tightship/skills/removed/SKILL.md")];
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
        for companion in COMPANIONS {
            assert!(
                !plugins.join(companion).exists(),
                "{installer:?}: {companion}"
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
