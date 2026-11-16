//! What the Cursor plugin renders.

use super::*;

fn canonical_content(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relative);
    fs::read_to_string(path).expect("canonical content")
}

fn claude_code_neutrality_terms() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../harnesses/claude-code/harness.toml");
    let text = fs::read_to_string(path).expect("claude-code harness");
    let section = text
        .split("[neutrality]")
        .nth(1)
        .and_then(|rest| rest.split("\n[").next())
        .expect("neutrality section");
    let mut terms = Vec::new();
    let mut chars = section.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            let mut term = String::new();
            for next in chars.by_ref() {
                if next == '"' {
                    break;
                }
                term.push(next);
            }
            if !term.is_empty() {
                terms.push(term);
            }
        }
    }
    terms
}

#[test]
fn cursor_plugin_renders_slash_commands_without_plugin_prefix() {
    let harness_files = harness_files("cursor");
    let mut files: Vec<(&str, String)> = harness_files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    files.extend([
        piece(
            "content/commands/spec.md",
            "Use {{skill:spec-writing}}, then {{command:epic}}.\n",
        ),
        piece("content/skills/spec-writing/SKILL.md", "Write the spec.\n"),
        piece("content/commands/epic.md", "Open the epic.\n"),
    ]);
    let repository = Repository::with(&files);

    assert_succeeds(&repository.run("build"));
    let spec = fs::read_to_string(repository.path("dist/cursor/commands/spec.md"))
        .expect("the rendered command");
    assert!(spec.contains("Use spec-writing, then /epic."));
    assert!(
        repository
            .path("dist/cursor/skills/spec-writing/SKILL.md")
            .is_file()
    );
    assert!(repository.path("dist/cursor/commands/epic.md").is_file());
    let manifest = json_at(&repository, "dist/cursor/.cursor-plugin/plugin.json");
    assert!(manifest.get("dependencies").is_none());
    assert!(manifest.get("userConfig").is_none());
    assert!(repository.path("dist/cursor/bin/tightship.cmd").is_file());
}

#[test]
fn cursor_manifest_lists_variables_from_options() {
    let (epic_path, epic_text) = piece("content/commands/epic.md", "Open the epic.\n");
    let (options_path, options_text) = typed_options(&[
        ("status_line", "boolean", "Show the status line"),
        ("contract", "boolean", "Load the rules"),
        (
            "memory",
            "boolean",
            "Keep notes out of {{capability:plan-mode}}",
        ),
        ("vault_folder", "directory", "Your vault"),
    ]);
    let repository = cursor_repository(vec![
        (epic_path.to_owned(), epic_text),
        (options_path.to_owned(), options_text),
    ]);

    assert_succeeds(&repository.run("build"));
    let manifest = json_at(&repository, "dist/cursor/.cursor-plugin/plugin.json");
    assert!(manifest.get("userConfig").is_none());
    assert!(manifest.get("dependencies").is_none());
    let options = manifest["variables"]
        .as_object()
        .expect("variables is an object");
    let names: Vec<&str> = options.keys().map(String::as_str).collect();
    assert_eq!(names, ["status_line", "contract", "memory", "vault_folder"]);
    assert_eq!(
        options["memory"],
        serde_json::json!({
            "type": "boolean",
            "title": "An option",
            "description": "Keep notes out of Plan mode",
            "default": true,
        })
    );
    assert!(
        options
            .values()
            .take(3)
            .all(|option| option["type"] == "boolean" && option["default"] == true)
    );
    assert_eq!(
        options["vault_folder"],
        serde_json::json!({
            "type": "directory",
            "title": "An option",
            "description": "Your vault",
        })
    );
    assert!(
        repository
            .path("dist/cursor/.cursor-plugin/marketplace.json")
            .is_file()
    );
}

#[test]
fn cursor_grilling_skill_holds_no_claude_code_literals() {
    let grilling = canonical_content("content/skills/grilling/SKILL.md");
    let repository = cursor_repository(vec![(
        "content/skills/grilling/SKILL.md".to_owned(),
        grilling,
    )]);

    assert_succeeds(&repository.run("build"));
    let rendered = fs::read_to_string(repository.path("dist/cursor/skills/grilling/SKILL.md"))
        .expect("rendered grilling skill");
    for term in claude_code_neutrality_terms() {
        assert!(
            !rendered.contains(&term),
            "dist/cursor/skills/grilling/SKILL.md must not contain claude-code term {:?}",
            term
        );
    }
}

#[test]
fn cursor_session_start_includes_status_line_hook() {
    let repository = cursor_repository(vec![(
        "content/options.toml".to_owned(),
        "[[options]]\nname = \"status_line\"\ntitle = \"Status line\"\ndescription = \"Status line\"\ntype = \"boolean\"\n".to_owned(),
    )]);

    assert_succeeds(&repository.run("build"));
    let hooks: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repository.path("dist/cursor/hooks/hooks.json")).expect("hooks"),
    )
    .expect("valid JSON");
    let binary = "${CURSOR_PLUGIN_ROOT}/bin/tightship";
    assert!(
        hooks["hooks"]["sessionStart"]
            .as_array()
            .expect("sessionStart hooks")
            .iter()
            .any(|hook| hook["command"] == format!("{binary} hook status-line")),
        "expected status-line hook in sessionStart: {hooks}"
    );
}

#[test]
fn cursor_verifier_agent_has_readonly_front_matter() {
    let verifier = canonical_content("content/agents/adversarial-verifier.md");
    let repository = cursor_repository(vec![(
        "content/agents/adversarial-verifier.md".to_owned(),
        verifier,
    )]);

    assert_succeeds(&repository.run("build"));
    let rendered =
        fs::read_to_string(repository.path("dist/cursor/agents/adversarial-verifier.md"))
            .expect("rendered verifier");
    assert!(
        rendered.contains("readonly: true\n"),
        "expected readonly: true in front matter:\n{rendered}"
    );
    assert!(!rendered.contains("readonly: \"true\""));
    assert!(rendered.contains("tools: \"Read, Grep, Glob, Shell\"\n"));
    assert!(rendered.contains("You are an adversarial verifier."));
}
