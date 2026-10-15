use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

struct Repository {
    root: TempDir,
}

impl Repository {
    fn with(files: &[(&str, String)]) -> Self {
        let root = tempfile::tempdir().expect("a temporary directory");
        let harness = include_str!("../../harnesses/claude-code/harness.toml");
        let base = [
            (
                "Cargo.toml",
                "[workspace.package]\nversion = \"0.1.0\"\n".to_owned(),
            ),
            ("harnesses/claude-code/harness.toml", harness.to_owned()),
        ];
        for (path, text) in base.iter().chain(files) {
            let file = root.path().join(path);
            fs::create_dir_all(file.parent().expect("a parent folder")).expect("the parent folder");
            fs::write(file, text).expect("the fixture file");
        }
        Self { root }
    }

    fn build(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .arg("build")
            .current_dir(self.root.path())
            .output()
            .expect("xtask runs")
    }

    fn path(&self, relative: &str) -> std::path::PathBuf {
        self.root.path().join(relative)
    }
}

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

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_fails_with(output: &Output, line: &str) {
    let stderr = stderr_of(output);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(
        stderr.lines().any(|found| found == line),
        "missing {line:?} in:\n{stderr}"
    );
}

fn assert_succeeds(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        stderr_of(output)
    );
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

    assert_succeeds(&repository.build());
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
        &repository.build(),
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
        &repository.build(),
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
        &repository.build(),
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
        &repository.build(),
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
        &repository.build(),
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

    assert_succeeds(&repository.build());
}

#[test]
fn a_harness_term_in_content_fails() {
    let repository = Repository::with(&[piece(
        "content/skills/grilling/SKILL.md",
        &on_line(60, "End the turn with ExitPlanMode."),
    )]);

    assert_fails_with(
        &repository.build(),
        "error: content/skills/grilling/SKILL.md:60: \"ExitPlanMode\" belongs to claude-code; use a {{capability:…}} token defined in harnesses/claude-code/harness.toml",
    );
}

#[test]
fn a_word_containing_a_term_is_not_a_match() {
    let repository = Repository::with(&[piece(
        "content/skills/api-design/SKILL.md",
        "Prefer cursor-based pagination once the data set is Explored.\n",
    )]);

    assert_succeeds(&repository.build());
}

#[test]
fn an_unknown_front_matter_key_fails() {
    let repository = Repository::with(&[(
        "content/agents/adversarial-verifier.md",
        "+++\nname = \"adversarial-verifier\"\ndescription = \"Verify\"\naccess = \"read-only\"\ntools = \"Read\"\n+++\nVerify.\n"
            .to_owned(),
    )]);

    assert_fails_with(
        &repository.build(),
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
        &repository.build(),
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
        &repository.build(),
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

    let output = repository.build();
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
