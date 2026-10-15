mod common;

use std::fs;
use std::path::Path;

use common::{Repository, assert_fails_with, assert_succeeds, stderr_of};

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
            "Use {{skill:spec-writing}}, then {{command:epic}}.\n",
        ),
        piece("content/commands/epic.md", "Open the epic.\n"),
        piece("content/skills/spec-writing/SKILL.md", "Write the spec.\n"),
    ]);

    assert_succeeds(&repository.run("build"));
    let spec = fs::read_to_string(repository.path("dist/claude-code/skills/spec/SKILL.md"))
        .expect("the rendered command");
    assert!(spec.contains("Use tightship:spec-writing, then /tightship:epic."));
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
    ]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/commands/sprint.md:30: \"/spec\" refers to the command \"spec\"; write {{command:spec}}",
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

#[test]
fn the_manifest_lists_the_companions_and_the_options_and_the_catalog_allows_them() {
    let repository = Repository::with(&[
        piece("content/commands/epic.md", "Open the epic.\n"),
        options(&[
            ("status_line", "Show the status line"),
            ("contract", "Load the rules"),
            ("memory", "Keep notes out of {{capability:plan-mode}}"),
        ]),
    ]);

    assert_succeeds(&repository.run("build"));
    let manifest = json_at(&repository, "dist/claude-code/.claude-plugin/plugin.json");
    let options = manifest["userConfig"]
        .as_object()
        .expect("userConfig is an object");
    let names: Vec<&str> = options.keys().map(String::as_str).collect();
    assert_eq!(names, ["status_line", "contract", "memory"]);
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
            .all(|option| option["type"] == "boolean" && option["default"] == true)
    );
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
    let text = entries
        .iter()
        .map(|(name, description)| {
            format!("[[options]]\nname = \"{name}\"\ntitle = \"An option\"\ndescription = \"{description}\"\n")
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
fn an_option_name_outside_its_domain_fails() {
    let repository = Repository::with(&[options(&[("status-line", "Show the status line")])]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/options.toml: \"status-line\" is not an option name; use 1 to 64 lowercase letters, digits, and underscores, starting with a letter",
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
        "error: content/options.toml:9: {{command:nope}} names no command in content/",
    );
}
