//! Running the installer again over an installed plugin.

use super::*;

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
        env.add_mock_script("agy", AGY_THAT_INSTALLS);
        env.add_mock_script("agent", AGENT_VERSION);
        env.add_mock_script("claude", &claude_that_installs());

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
            .map(|entry| entry.expect("an entry").file_name())
            .filter(|name| name != ".dotnet")
            .collect();
        assert!(leftovers.is_empty(), "{installer:?}: {leftovers:?}");
    }
}
