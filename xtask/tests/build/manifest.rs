//! The manifest and launcher of every plugin, built from the canonical content.

use super::*;

#[test]
fn harnesses_naming_different_repositories_fail() {
    let (path, epic) = piece("content/commands/epic.md", "Open the epic.\n");
    let repository = cursor_repository(vec![(path.to_owned(), epic)]);
    let manifest = repository.path("harnesses/cursor/harness.toml");
    let text = fs::read_to_string(&manifest).expect("the cursor harness");
    repository.write(
        "harnesses/cursor/harness.toml",
        &text.replace(
            "repository = \"https://github.com/ATNexusLab/tightship\"",
            "repository = \"https://github.com/fork/tightship\"",
        ),
    );

    assert_fails_with(
        &repository.run("build"),
        "error: the harnesses name different repositories: claude-code \"https://github.com/ATNexusLab/tightship\", cursor \"https://github.com/fork/tightship\"; use one",
    );
}

#[test]
fn the_manifest_lists_the_options_and_depends_on_no_other_plugin() {
    let repository = Repository::with(&[
        piece("content/commands/epic.md", "Open the epic.\n"),
        typed_options(&[
            ("status_line", "boolean", "Show the status line"),
            ("contract", "boolean", "Load the rules"),
            (
                "memory",
                "boolean",
                "Keep notes out of {{capability:plan-mode}}",
            ),
            ("vault_folder", "directory", "Your vault"),
        ]),
    ]);

    assert_succeeds(&repository.run("build"));
    let manifest = json_at(&repository, "dist/claude-code/.claude-plugin/plugin.json");
    let options = manifest["userConfig"]
        .as_object()
        .expect("userConfig is an object");
    let names: Vec<&str> = options.keys().map(String::as_str).collect();
    assert_eq!(names, ["status_line", "contract", "memory", "vault_folder"]);
    assert_eq!(
        options["memory"],
        serde_json::json!({
            "type": "boolean",
            "title": "An option",
            "description": "Keep notes out of plan mode",
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
    assert_eq!(manifest["license"], "MIT");
    assert!(manifest.get("dependencies").is_none());
    let catalog = json_at(
        &repository,
        "dist/claude-code/.claude-plugin/marketplace.json",
    );
    assert_eq!(catalog["name"], "tightship");
    assert_eq!(
        catalog["plugins"],
        serde_json::json!([{
            "name": "tightship",
            "source": "./",
            "description": manifest["description"],
        }])
    );
    assert!(catalog.get("allowCrossMarketplaceDependenciesOn").is_none());
}

#[test]
fn the_launcher_runs_the_binary_built_for_this_machine() {
    let repository = Repository::with(&[piece("content/commands/epic.md", "Open the epic.\n")]);
    let vault = repository.path("vault");
    fs::create_dir(&vault).expect("a vault folder");

    assert_succeeds(&repository.run("build"));
    assert!(
        repository
            .path("dist/claude-code/bin/tightship.cmd")
            .is_file()
    );
    let output = Command::new("sh")
        .arg(repository.path("dist/claude-code/bin/tightship"))
        .args(["vault", "path"])
        .env("TIGHTSHIP_VAULT", &vault)
        .output()
        .expect("the launcher runs");
    assert_succeeds(&output);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{}\n", vault.display())
    );
}

#[test]
fn both_harnesses_build_from_canonical_content_and_pass_validation() {
    let harness_files = harness_files("antigravity-cli");
    let mut files: Vec<(&str, String)> = harness_files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    files.extend([
        typed_options(&[
            ("contract", "boolean", "Contract"),
            ("memory", "boolean", "Memory"),
            ("status_line", "boolean", "Status line"),
            ("vault_folder", "directory", "Vault folder"),
        ]),
        (
            "content/contract.md",
            "+++\n[gated]\nMemory = \"memory\"\n+++\n\n# Global Working Contract\n\nPreamble.\n\n## Section 1\n\nSection 1 text.\n\n## Memory\n\nMemory text.\n".to_owned(),
        ),
        (
            "content/agents/adversarial-verifier.md",
            "+++\nname = \"adversarial-verifier\"\ndescription = \"Verifier\"\naccess = \"read-only\"\n+++\nVerifier body.\n".to_owned(),
        ),
    ]);
    let skill_names = [
        "architecture-reading",
        "spec-writing",
        "frontend-architecture",
        "technical-writing",
        "hexagonal-architecture",
        "requirements-engineering",
        "grilling",
        "security-audit",
        "project-docs",
        "api-design",
        "performance-analysis",
        "testing-contract",
        "database-design",
        "prose-editing",
    ];
    let command_names = [
        "epic",
        "audit",
        "sync",
        "spec",
        "requirements",
        "sprint",
        "pr-review",
        "closeout",
        "handoff",
        "bootstrap",
    ];
    let pieces: Vec<(String, String)> = skill_names
        .iter()
        .map(|name| {
            (
                format!("content/skills/{name}/SKILL.md"),
                format!(
                    "+++\nname = \"{name}\"\ndescription = \"Skill {name}\"\n+++\nBody of {name}.\n"
                ),
            )
        })
        .chain(command_names.iter().map(|name| {
            (
                format!("content/commands/{name}.md"),
                format!("+++\nname = \"{name}\"\ndescription = \"Command {name}\"\nargument-hint = \"[arg]\"\n+++\nBody of {name}.\n"),
            )
        }))
        .collect();
    files.extend(
        pieces
            .iter()
            .map(|(path, text)| (path.as_str(), text.clone())),
    );

    let repository = Repository::with(&files);
    assert_succeeds(&repository.run("build"));

    assert!(repository.path("dist/claude-code").is_dir());
    assert!(repository.path("dist/antigravity-cli").is_dir());

    for name in skill_names {
        assert!(
            repository
                .path(&format!("dist/antigravity-cli/skills/{name}/SKILL.md"))
                .is_file()
        );
    }
    for name in command_names {
        let skill_file = repository.path(&format!("dist/antigravity-cli/skills/{name}/SKILL.md"));
        assert!(skill_file.is_file());
        let text = fs::read_to_string(&skill_file).expect("command as skill");
        assert!(!text.contains("argument-hint"));
    }
    assert!(
        repository
            .path("dist/antigravity-cli/skills/contract/SKILL.md")
            .is_file()
    );

    let verifier =
        fs::read_to_string(repository.path("dist/antigravity-cli/agents/adversarial-verifier.md"))
            .expect("verifier agent");
    assert!(verifier.contains("view_file"));
    assert!(verifier.contains("list_dir"));
    assert!(verifier.contains("find_by_name"));
    assert!(verifier.contains("grep_search"));
    assert!(verifier.contains("run_command"));
    assert!(!verifier.contains("write_to_file"));
    assert!(!verifier.contains("replace_file_content"));

    let part_1 = fs::read_to_string(repository.path("dist/antigravity-cli/contract/part-1.md"))
        .expect("part-1.md");
    assert!(part_1.contains(
        "After any context compaction (/compact), load the contract skill to restore this contract in full."
    ));

    if Command::new("which")
        .arg("agy")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
    {
        let validate = Command::new("agy")
            .arg("plugin")
            .arg("validate")
            .arg(repository.path("dist/antigravity-cli"))
            .arg("--strict")
            .output()
            .expect("agy validate runs");
        assert!(
            validate.status.success(),
            "agy plugin validate: {}",
            String::from_utf8_lossy(&validate.stderr)
        );
    }
}
