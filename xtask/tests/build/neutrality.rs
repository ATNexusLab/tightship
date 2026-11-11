//! Harness terms in the content and identity terms in what ships.

use super::*;

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
fn a_harness_term_in_the_binary_sources_fails() {
    let repository = Repository::with(&[(
        "tightship/src/status/render.rs",
        "fn prefix() -> &'static str {\n    \"Claude Code\"\n}\n".to_owned(),
    )]);

    assert_fails_with(
        &repository.run("build"),
        "error: tightship/src/status/render.rs:2: \"Claude Code\" belongs to claude-code; move it to harnesses/claude-code/ and pass it in through the launcher",
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
fn an_identity_term_in_an_installer_fragment_fails_without_printing_it() {
    let repository = Repository::with(&[
        piece("content/skills/grilling/SKILL.md", "Grill.\n"),
        (
            "installer/install.sh",
            "#!/bin/sh\n{{fragments}}\n".to_owned(),
        ),
        (
            "harnesses/claude-code/install.sh",
            "install_claude_code() {\n  echo ghost-handle-0451\n}\n".to_owned(),
        ),
    ]);
    repository.write(".identity-terms", "Ghost-Handle-0451\n");

    let output = repository.run("build");
    assert_fails_with(
        &output,
        "error: dist/install.sh:3: holds an identity term of the maintainer",
    );
    assert!(
        !stderr_of(&output)
            .to_lowercase()
            .contains("ghost-handle-0451")
    );
    assert!(!repository.path("dist/install.sh").exists());
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
