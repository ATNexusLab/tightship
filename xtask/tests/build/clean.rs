//! The clean data a harness declares and where it reaches the plugin.

use super::*;

fn harness_without_clean() -> &'static str {
    include_str!("../../../harnesses/claude-code/harness.toml")
        .split_once("[clean]")
        .map_or("", |(base, _)| base)
}

#[test]
fn the_clean_data_reaches_each_plugin_and_its_session_start() {
    let mut files = harness_files("cursor");
    files.extend(harness_files("antigravity-cli"));
    let files: Vec<(&str, String)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .chain([
            piece("content/commands/epic.md", "Open the epic.\n"),
            options(&[("status_line", "Show the status line")]),
        ])
        .collect();
    let repository = Repository::with(&files);

    assert_succeeds(&repository.run("build"));
    for harness in ["claude-code", "cursor", "antigravity-cli"] {
        let clean = json_at(&repository, &format!("dist/{harness}/clean.json"));
        assert_eq!(
            clean["names"],
            serde_json::json!(["ponytail", "i-have-adhd", "humanizer"]),
            "{harness}"
        );
        let paths = clean["paths"].as_array().expect("the paths");
        assert_eq!(
            paths.last(),
            Some(&serde_json::json!({ "path": "{appdata}/ponytail/" })),
            "{harness}"
        );
    }
    let hooks = json_at(&repository, "dist/claude-code/hooks/hooks.json");
    let startup = hooks["hooks"]["SessionStart"]
        .as_array()
        .expect("the session start hooks")
        .iter()
        .find(|entry| entry["matcher"] == "startup")
        .expect("a startup hook");
    let command = startup["hooks"][0]["command"].as_str().expect("a command");
    let (clean, status_line) = command.split_once("; ").expect("two commands");
    assert!(clean.ends_with("hook clean"), "{command}");
    assert!(status_line.ends_with("hook status-line"), "{command}");
    for file in [
        "dist/cursor/hooks/hooks.json",
        "dist/antigravity-cli/hooks.json",
    ] {
        let text = fs::read_to_string(repository.path(file)).expect("the hooks");
        assert!(text.contains("bin/tightship hook clean"), "{file}");
    }
}

#[test]
fn a_companion_named_outside_clean_fails_and_leaves_no_plugin() {
    let repository = Repository::with(&[
        piece("content/commands/epic.md", "Open the epic.\n"),
        (
            "harnesses/claude-code/harness.toml",
            format!(
                "{}[paths]\nmodes = \".claude/ponytail-modes\"\n",
                harness_without_clean()
            ),
        ),
    ]);
    fs::create_dir_all(repository.path("dist/claude-code")).expect("a previous build");

    assert_fails_with(
        &repository.run("build"),
        "error: harnesses/claude-code/harness.toml: ponytail is named outside [clean]; tightship depends on no other plugin",
    );
    assert!(!repository.path("dist/claude-code").exists());
}

#[test]
fn a_clean_path_outside_home_fails() {
    for path in ["/etc/ponytail/", "~/../ponytail/"] {
        let repository = Repository::with(&[
            piece("content/commands/epic.md", "Open the epic.\n"),
            (
                "harnesses/claude-code/harness.toml",
                format!(
                    "{}[clean]\nnames = [\"ponytail\"]\npaths = [{{ path = \"{path}\" }}]\n",
                    harness_without_clean()
                ),
            ),
        ]);

        assert_fails_with(
            &repository.run("build"),
            &format!(
                "error: harnesses/claude-code/harness.toml: [clean] path {path} is outside the home folder"
            ),
        );
    }
}

#[test]
fn a_plugin_file_on_a_rendered_path_fails_and_leaves_no_plugin() {
    let repository = Repository::with(&[
        piece("content/commands/epic.md", "Open the epic.\n"),
        (
            "harnesses/claude-code/plugin/skills/epic/SKILL.md",
            "A copied file.\n".to_owned(),
        ),
    ]);
    fs::create_dir_all(repository.path("dist/claude-code")).expect("a previous build");

    assert_fails_with(
        &repository.run("build"),
        "error: dist/claude-code/skills/epic/SKILL.md: two sources write this file",
    );
    assert!(!repository.path("dist/claude-code").exists());
}

#[test]
fn no_clean_and_no_option_leave_out_their_keys_and_files() {
    let repository = Repository::with(&[
        piece("content/commands/epic.md", "Open the epic.\n"),
        (
            "harnesses/claude-code/harness.toml",
            harness_without_clean().to_owned(),
        ),
    ]);

    assert_succeeds(&repository.run("build"));
    assert!(!repository.path("dist/claude-code/clean.json").exists());
    let manifest = json_at(&repository, "dist/claude-code/.claude-plugin/plugin.json");
    assert!(manifest.get("dependencies").is_none());
    assert!(manifest.get("userConfig").is_none());
    let catalog = json_at(
        &repository,
        "dist/claude-code/.claude-plugin/marketplace.json",
    );
    assert!(catalog.get("allowCrossMarketplaceDependenciesOn").is_none());
}
