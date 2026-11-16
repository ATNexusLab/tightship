//! Removing the companion plugins tightship replaces.

use super::*;

fn seed(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("a leftover folder");
    fs::write(path, text).expect("a leftover file");
}

#[test]
fn scenario_2_the_installer_removes_the_companions_in_cursor_and_antigravity_cli() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agent", AGENT_VERSION);
        env.add_mock_script(
            "agy",
            &AGY_THAT_INSTALLS.replacen(
                "if [ \"$1\" = \"--version\" ]",
                "echo \"$*\" >> \"$HOME/agy-arguments\"\nif [ \"$1\" = \"--version\" ]",
                1,
            ),
        );
        let home = &env.home_dir;
        for companion in COMPANIONS {
            seed(
                &home.join(format!(".cursor/plugins/local/{companion}/plugin.json")),
                "{}",
            );
        }
        seed(&home.join(".cursor/.ponytail-active"), "full");
        seed(&home.join(".config/ponytail/band/ledger.jsonl"), "{}");
        seed(
            &home.join(".gemini/config/plugins/ponytail/plugin.json"),
            "{}",
        );
        seed(
            &home.join(".gemini/config/plugins/i-have-adhd/plugin.json"),
            "{}",
        );
        seed(
            &home.join(".gemini/config/skills/humanizer/SKILL.md"),
            "humanizer",
        );
        seed(
            &home.join(".gemini/config/config.json"),
            r#"{"plugins": {"ponytail": {}, "i-have-adhd": {}, "other": {}}}"#,
        );
        seed(
            &home.join(".gemini/config/import_manifest.json"),
            r#"{"imports": [{"name": "ponytail"}, {"name": "i-have-adhd"}, {"name": "other"}]}"#,
        );

        for (harness, from_dir) in [
            ("cursor", cursor_from_dir()),
            ("antigravity-cli", from_dir()),
        ] {
            let output = env.run(
                installer,
                &["--harness", harness, "--from-dir", &from_dir, "--yes"],
            );
            assert_exit(installer, &output, 0);
            assert!(
                String::from_utf8_lossy(&output.stdout)
                    .contains("ok  Removing the plugins tightship replaces"),
                "{installer:?} {harness}: {}",
                String::from_utf8_lossy(&output.stdout)
            );
        }

        for gone in [
            ".cursor/plugins/local/ponytail",
            ".cursor/plugins/local/i-have-adhd",
            ".cursor/plugins/local/humanizer",
            ".cursor/.ponytail-active",
            ".config/ponytail",
            ".gemini/config/plugins/ponytail",
            ".gemini/config/plugins/i-have-adhd",
            ".gemini/config/skills/humanizer",
        ] {
            assert!(!home.join(gone).exists(), "{installer:?}: {gone} is left");
        }
        let json = |file: &str| -> serde_json::Value {
            serde_json::from_str(&fs::read_to_string(home.join(file)).expect("a JSON file"))
                .expect("valid JSON")
        };
        assert_eq!(
            json(".gemini/config/config.json"),
            serde_json::json!({ "plugins": { "other": {} } }),
            "{installer:?}"
        );
        assert_eq!(
            json(".gemini/config/import_manifest.json"),
            serde_json::json!({ "imports": [{ "name": "other" }] }),
            "{installer:?}"
        );
        let agy_arguments = fs::read_to_string(home.join("agy-arguments")).expect("agy ran");
        for uninstalled in ["plugin uninstall ponytail", "plugin uninstall i-have-adhd"] {
            assert!(
                agy_arguments.lines().any(|line| line == uninstalled),
                "{installer:?}: {agy_arguments}"
            );
        }
        assert!(
            home.join(".cursor/plugins/local/tightship").is_dir()
                && home.join(".gemini/config/plugins/tightship").is_dir(),
            "{installer:?}"
        );
    }
}
