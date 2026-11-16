//! Installing the Antigravity CLI plugin.

use super::*;

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
        for companion in COMPANIONS {
            assert!(
                !plugins.join(companion).exists(),
                "{installer:?}: {companion}"
            );
        }
        assert!(
            !env.home_dir.join(".gemini/config/skills").exists(),
            "{installer:?}"
        );
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
