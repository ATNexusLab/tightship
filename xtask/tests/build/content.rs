//! The front matter and names of the pieces, and how errors are reported.

use super::*;

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
