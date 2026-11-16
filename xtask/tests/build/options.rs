//! The options `content/options.toml` declares.

use super::*;

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
