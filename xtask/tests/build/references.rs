//! Tokens and references between pieces.

use super::*;

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
fn a_broken_token_fails() {
    let repository = Repository::with(&[
        piece(
            "content/skills/planning/SKILL.md",
            &on_line(7, "Load {{skill:\ngrilling}} first."),
        ),
        piece(
            "content/commands/spec.md",
            &on_line(9, "Load {{skill:grilling} first."),
        ),
    ]);

    let output = repository.run("build");
    for line in [
        "error: content/skills/planning/SKILL.md:7: \"{{skill:\" is not a token; write {{<kind>:<name>}} on one line",
        "error: content/skills/planning/SKILL.md:8: \"grilling}}\" is not a token; write {{<kind>:<name>}} on one line",
        "error: content/commands/spec.md:9: \"{{skill:grilling}\" is not a token; write {{<kind>:<name>}} on one line",
    ] {
        assert_fails_with(&output, line);
    }
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
