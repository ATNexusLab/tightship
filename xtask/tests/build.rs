mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use common::{Repository, assert_fails_with, assert_succeeds, harness_files, stderr_of};

fn piece<'path>(path: &'path str, body: &str) -> (&'path str, String) {
    let name = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| *stem != "SKILL")
        .or_else(|| path.split('/').nth(2))
        .expect("a piece name");
    let access = if path.starts_with("content/agents/") {
        "access = \"read-only\"\n"
    } else {
        ""
    };
    let text = format!("+++\nname = \"{name}\"\ndescription = \"A piece\"\n{access}+++\n{body}");
    (path, text)
}

fn on_line(line: usize, text: &str) -> String {
    let first_body_line = 5;
    "\n".repeat(line - first_body_line) + text + "\n"
}

#[test]
fn references_render_under_the_plugin_names() {
    let repository = Repository::with(&[
        piece(
            "content/commands/spec.md",
            "Use {{skill:spec-writing}}, then {{command:epic}}.\nMemory is `{{option:memory}}`.\n",
        ),
        piece("content/commands/epic.md", "Open the epic.\n"),
        piece("content/skills/spec-writing/SKILL.md", "Write the spec.\n"),
        options(&[("memory", "Keep notes")]),
    ]);

    assert_succeeds(&repository.run("build"));
    let spec = fs::read_to_string(repository.path("dist/claude-code/skills/spec/SKILL.md"))
        .expect("the rendered command");
    assert!(spec.contains("Use tightship:spec-writing, then /tightship:epic."));
    assert!(spec.contains("Memory is `${user_config.memory}`."));
    assert!(
        repository
            .path("dist/claude-code/skills/spec-writing")
            .is_dir()
    );
    assert!(repository.path("dist/claude-code/skills/epic").is_dir());
}

#[test]
fn a_token_naming_a_missing_piece_fails_and_leaves_no_plugin() {
    let repository = Repository::with(&[piece(
        "content/commands/spec.md",
        &on_line(12, "Use {{skill:spec-writer}}."),
    )]);
    fs::create_dir_all(repository.path("dist/claude-code")).expect("a previous build");

    assert_fails_with(
        &repository.run("build"),
        "error: content/commands/spec.md:12: {{skill:spec-writer}} names no skill in content/",
    );
    assert!(!repository.path("dist/claude-code").exists());
}

#[test]
fn an_unknown_token_kind_fails() {
    let repository = Repository::with(&[piece(
        "content/skills/memory/SKILL.md",
        &on_line(5, "Run {{script:vault}}."),
    )]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/skills/memory/SKILL.md:5: unknown token kind \"script\" in {{script:vault}}",
    );
}

#[test]
fn an_undefined_capability_fails() {
    let repository = Repository::with(&[piece(
        "content/skills/planning/SKILL.md",
        &on_line(6, "Track it in {{capability:todo-list}}."),
    )]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/skills/planning/SKILL.md:6: capability \"todo-list\" has no entry in harnesses/claude-code/harness.toml",
    );
}

#[test]
fn the_option_token_misused_fails() {
    let cases = [
        (
            piece("content/commands/x.md", &on_line(5, "{{option:nope}}")),
            "content/commands/x.md:5: {{option:nope}} names no boolean option in content/options.toml",
        ),
        (
            piece(
                "content/commands/x.md",
                &on_line(5, "{{option:vault_folder}}"),
            ),
            "content/commands/x.md:5: {{option:vault_folder}} names no boolean option in content/options.toml",
        ),
        (
            (
                "content/contract.md",
                "# Contract\n\n{{option:memory}}\n".to_owned(),
            ),
            "content/contract.md:3: {{option:memory}} works only in a skill, a command, or an agent",
        ),
        (
            piece(
                "content/commands/x.md",
                &on_line(5, "${user_config.memory}"),
            ),
            "content/commands/x.md:5: \"${user_config\" belongs to claude-code; use a {{capability:…}} token defined in harnesses/claude-code/harness.toml",
        ),
    ];
    for (file, message) in cases {
        let repository = Repository::with(&[
            file,
            typed_options(&[
                ("memory", "boolean", "Keep notes"),
                ("vault_folder", "directory", "Your vault"),
            ]),
        ]);

        assert_fails_with(&repository.run("build"), &format!("error: {message}"));
    }
}

#[test]
fn a_malformed_token_fails() {
    let repository = Repository::with(&[piece(
        "content/skills/planning/SKILL.md",
        &on_line(7, "Load {{skill grilling}} first."),
    )]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/skills/planning/SKILL.md:7: \"{{skill grilling}}\" is not a token; write {{<kind>:<name>}}",
    );
}

#[test]
fn a_literal_reference_fails() {
    let repository = Repository::with(&[
        piece("content/commands/spec.md", "Write the spec.\n"),
        piece(
            "content/commands/sprint.md",
            &on_line(30, "Run `/spec` for each item."),
        ),
        (
            "content/vault-template/AGENTS.md",
            "# Vault\n\nRun `/spec` first.\n".to_owned(),
        ),
    ]);

    let output = repository.run("build");
    assert_fails_with(
        &output,
        "error: content/commands/sprint.md:30: \"/spec\" refers to the command \"spec\"; write {{command:spec}}",
    );
    assert_fails_with(
        &output,
        "error: content/vault-template/AGENTS.md:3: \"/spec\" refers to the command \"spec\"; write {{command:spec}}",
    );
}

#[test]
fn a_path_containing_a_command_name_is_not_a_reference() {
    let repository = Repository::with(&[
        piece("content/commands/spec.md", "Write the spec.\n"),
        piece("content/commands/requirements.md", "Elicit them.\n"),
        piece(
            "content/skills/project-docs/SKILL.md",
            "Specs live in docs/specs/<epic>/, the SRS in docs/requirements/, the index in docs/spec.\n",
        ),
    ]);

    assert_succeeds(&repository.run("build"));
}

#[test]
fn a_harness_term_in_content_fails() {
    let repository = Repository::with(&[piece(
        "content/skills/grilling/SKILL.md",
        &on_line(60, "End the turn with ExitPlanMode."),
    )]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/skills/grilling/SKILL.md:60: \"ExitPlanMode\" belongs to claude-code; use a {{capability:…}} token defined in harnesses/claude-code/harness.toml",
    );
}

#[test]
fn a_word_containing_a_term_is_not_a_match() {
    let repository = Repository::with(&[piece(
        "content/skills/api-design/SKILL.md",
        "Prefer cursor-based pagination once the data set is Explored.\n",
    )]);

    assert_succeeds(&repository.run("build"));
}

#[test]
fn an_unknown_front_matter_key_fails() {
    let repository = Repository::with(&[(
        "content/agents/adversarial-verifier.md",
        "+++\nname = \"adversarial-verifier\"\ndescription = \"Verify\"\naccess = \"read-only\"\ntools = \"Read\"\n+++\nVerify.\n"
            .to_owned(),
    )]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/agents/adversarial-verifier.md: front matter: unknown key \"tools\"",
    );
}

#[test]
fn a_name_that_differs_from_its_file_fails() {
    let repository = Repository::with(&[(
        "content/commands/spec.md",
        "+++\nname = \"specs\"\ndescription = \"Spec\"\n+++\nWrite the spec.\n".to_owned(),
    )]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/commands/spec.md: name \"specs\" differs from its file name \"spec\"",
    );
}

#[test]
fn a_name_taken_by_two_pieces_fails() {
    let repository = Repository::with(&[
        piece("content/skills/audit/SKILL.md", "Audit.\n"),
        piece("content/commands/audit.md", "Audit.\n"),
    ]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/skills/audit/SKILL.md: the name \"audit\" already belongs to content/commands/audit.md",
    );
}

#[test]
fn every_error_is_reported_in_one_run_sorted_by_path_and_line() {
    let repository = Repository::with(&[
        piece(
            "content/skills/grilling/SKILL.md",
            &on_line(8, "Use {{skill:missing}}."),
        ),
        piece(
            "content/commands/spec.md",
            &on_line(9, "Then ExitPlanMode in Claude Code."),
        ),
    ]);

    let output = repository.run("build");
    let errors: Vec<String> = stderr_of(&output).lines().map(str::to_owned).collect();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        errors,
        [
            "error: content/commands/spec.md:9: \"Claude Code\" belongs to claude-code; use a {{capability:…}} token defined in harnesses/claude-code/harness.toml",
            "error: content/commands/spec.md:9: \"ExitPlanMode\" belongs to claude-code; use a {{capability:…}} token defined in harnesses/claude-code/harness.toml",
            "error: content/skills/grilling/SKILL.md:8: {{skill:missing}} names no skill in content/",
        ]
    );
}

#[test]
fn an_identity_term_in_a_shipped_file_fails_without_printing_it() {
    let repository = Repository::with(&[piece(
        "content/skills/grilling/SKILL.md",
        &on_line(7, "Ask ghost-handle-0451 before planning."),
    )]);
    repository.write(".identity-terms", "Ghost-Handle-0451\n");

    let output = repository.run("build");
    assert_fails_with(
        &output,
        "error: dist/claude-code/skills/grilling/SKILL.md:7: holds an identity term of the maintainer",
    );
    assert!(
        !stderr_of(&output)
            .to_lowercase()
            .contains("ghost-handle-0451")
    );
    assert!(!repository.path("dist/claude-code").exists());
}

#[test]
fn a_file_named_with_an_identity_term_fails_without_printing_it() {
    let repository = Repository::with(&[
        piece("content/skills/grilling/SKILL.md", "Grill.\n"),
        (
            "content/skills/grilling/ghost-handle-0451.md",
            "Notes.\n".to_owned(),
        ),
    ]);
    repository.write(".identity-terms", "ghost-handle-0451\n");

    let output = repository.run("build");
    assert_fails_with(
        &output,
        "error: dist/claude-code/skills/grilling: holds a file or folder named with an identity term of the maintainer",
    );
    assert!(!stderr_of(&output).contains("ghost-handle-0451"));
}

#[test]
fn a_missing_identity_terms_file_fails() {
    let repository = Repository::with(&[piece("content/skills/grilling/SKILL.md", "Grill.\n")]);
    fs::remove_file(repository.path(".identity-terms")).expect("the terms file");

    assert_fails_with(
        &repository.run("build"),
        "error: .identity-terms is missing; create it with one line per name, email address, user name, and handle of the maintainer",
    );
}

#[test]
fn a_missing_git_identity_fails() {
    let repository = Repository::with(&[piece("content/skills/grilling/SKILL.md", "Grill.\n")]);
    repository.git(&["config", "--unset", "user.email"]);

    assert_fails_with(
        &repository.run("build"),
        "error: git config user.name or user.email is empty",
    );
}

fn harness_without_companions() -> &'static str {
    include_str!("../../harnesses/claude-code/harness.toml")
        .split_once("[[companions]]")
        .map_or("", |(base, _)| base)
}

fn json_at(repository: &Repository, path: &str) -> serde_json::Value {
    let text = fs::read_to_string(repository.path(path)).expect("a generated JSON file");
    serde_json::from_str(&text).expect("valid JSON")
}

fn canonical_content(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(relative);
    fs::read_to_string(path).expect("canonical content")
}

fn claude_code_neutrality_terms() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../harnesses/claude-code/harness.toml");
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

fn cursor_fixture_files(extra: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut files = harness_files("cursor");
    files.extend(extra);
    files
}

fn cursor_repository(extra: Vec<(String, String)>) -> Repository {
    let built = cursor_fixture_files(extra);
    let files: Vec<(&str, String)> = built
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    Repository::with(&files)
}

#[test]
fn the_manifest_lists_the_companions_and_the_options_and_the_catalog_allows_them() {
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
    assert_eq!(
        manifest["dependencies"],
        serde_json::json!([
            "ponytail@ponytail",
            "i-have-adhd@i-have-adhd",
            "humanizer@humanizer"
        ])
    );
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
    assert_eq!(
        catalog["allowCrossMarketplaceDependenciesOn"],
        serde_json::json!(["ponytail", "i-have-adhd", "humanizer"])
    );
}

#[test]
fn the_launcher_runs_the_binary_built_for_this_machine() {
    let repository = Repository::with(&[piece("content/commands/epic.md", "Open the epic.\n")]);
    let vault = repository.path("vault");
    fs::create_dir(&vault).expect("a vault folder");

    assert_succeeds(&repository.run("build"));
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
fn a_companion_without_a_marketplace_fails_and_leaves_no_plugin() {
    let repository = Repository::with(&[(
        "harnesses/claude-code/harness.toml",
        format!(
            "{}[[companions]]\nplugin = \"ponytail\"\n",
            harness_without_companions()
        ),
    )]);
    fs::create_dir_all(repository.path("dist/claude-code")).expect("a previous build");

    assert_fails_with(
        &repository.run("build"),
        "error: harnesses/claude-code/harness.toml: missing field `marketplace`",
    );
    assert!(!repository.path("dist/claude-code").exists());
}

#[test]
fn no_companion_and_no_option_leave_out_their_keys() {
    let repository = Repository::with(&[
        piece("content/commands/epic.md", "Open the epic.\n"),
        (
            "harnesses/claude-code/harness.toml",
            harness_without_companions().to_owned(),
        ),
    ]);

    assert_succeeds(&repository.run("build"));
    let manifest = json_at(&repository, "dist/claude-code/.claude-plugin/plugin.json");
    assert!(manifest.get("dependencies").is_none());
    assert!(manifest.get("userConfig").is_none());
    let catalog = json_at(
        &repository,
        "dist/claude-code/.claude-plugin/marketplace.json",
    );
    assert!(catalog.get("allowCrossMarketplaceDependenciesOn").is_none());
}

fn options(entries: &[(&str, &str)]) -> (&'static str, String) {
    typed_options(
        &entries
            .iter()
            .map(|(name, description)| (*name, "boolean", *description))
            .collect::<Vec<_>>(),
    )
}

fn typed_options(entries: &[(&str, &str, &str)]) -> (&'static str, String) {
    let text = entries
        .iter()
        .map(|(name, kind, description)| {
            format!("[[options]]\nname = \"{name}\"\ntitle = \"An option\"\ndescription = \"{description}\"\ntype = \"{kind}\"\n")
        })
        .collect::<Vec<String>>()
        .join("\n");
    ("content/options.toml", text)
}

#[test]
fn an_option_missing_a_key_fails_and_leaves_no_plugin() {
    let repository = Repository::with(&[(
        "content/options.toml",
        "[[options]]\nname = \"memory\"\ndescription = \"Keep notes\"\n".to_owned(),
    )]);
    fs::create_dir_all(repository.path("dist/claude-code")).expect("a previous build");

    assert_fails_with(
        &repository.run("build"),
        "error: content/options.toml: missing field `title`",
    );
    assert!(!repository.path("dist/claude-code").exists());
}

#[test]
fn an_option_name_or_type_outside_its_domain_fails() {
    let repository = Repository::with(&[typed_options(&[
        ("status-line", "boolean", "Show the status line"),
        ("vault_folder", "string", "Your vault"),
    ])]);

    let output = repository.run("build");
    assert_fails_with(
        &output,
        "error: content/options.toml: \"status-line\" is not an option name; use 1 to 64 lowercase letters, digits, and underscores, starting with a letter",
    );
    assert_fails_with(
        &output,
        "error: content/options.toml: the type of \"vault_folder\" must be boolean or directory",
    );
}

#[test]
fn an_option_description_too_long_fails() {
    let repository = Repository::with(&[options(&[("memory", &"a".repeat(121))])]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/options.toml: the description of \"memory\" must be 1 to 120 characters",
    );
}

#[test]
fn an_option_declared_twice_fails() {
    let repository = Repository::with(&[options(&[
        ("memory", "Keep notes"),
        ("memory", "Keep notes again"),
        ("memory", "Keep notes once more"),
    ])]);
    let line = "error: content/options.toml: the option \"memory\" is declared twice";

    let output = repository.run("build");
    assert_fails_with(&output, line);
    assert_eq!(stderr_of(&output).matches(line).count(), 1);
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
    assert!(
        repository
            .path("dist/cursor/bin/tightship.cmd")
            .is_file()
    );
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
    let rendered = fs::read_to_string(
        repository.path("dist/cursor/skills/grilling/SKILL.md"),
    )
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
    let hooks: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repository.path("dist/cursor/hooks/hooks.json")).expect("hooks"))
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
    let rendered = fs::read_to_string(
        repository.path("dist/cursor/agents/adversarial-verifier.md"),
    )
    .expect("rendered verifier");
    assert!(
        rendered.contains("readonly: true\n") || rendered.contains("readonly: \"true\"\n"),
        "expected readonly: true in front matter:\n{rendered}"
    );
    assert!(rendered.contains("tools: \"Read, Grep, Glob, Shell\"\n"));
    assert!(rendered.contains("You are an adversarial verifier."));
}

#[test]
fn an_option_naming_the_harness_or_a_missing_piece_fails() {
    let repository = Repository::with(&[options(&[
        ("memory", "Keep notes in ~/.claude"),
        ("contract", "Load the rules before {{command:nope}}"),
    ])]);

    let output = repository.run("build");
    assert_fails_with(
        &output,
        "error: content/options.toml:4: \"~/.claude\" belongs to claude-code; use a {{capability:…}} token defined in harnesses/claude-code/harness.toml",
    );
    assert_fails_with(
        &output,
        "error: content/options.toml:10: {{command:nope}} names no command in content/",
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
