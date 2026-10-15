mod common;

use std::fs;
use std::process::Command;

use common::{Repository, assert_fails_with, assert_succeeds};

const HARNESS: &str = include_str!("../../harnesses/claude-code/harness.toml");

fn contract(front_matter: &str, body: &str) -> (&'static str, String) {
    let text = if front_matter.is_empty() {
        body.to_owned()
    } else {
        format!("+++\n{front_matter}+++\n\n{body}")
    };
    ("content/contract.md", text)
}

fn options(names: &[&str]) -> (&'static str, String) {
    let text = names
        .iter()
        .map(|name| {
            format!("[[options]]\nname = \"{name}\"\ntitle = \"An option\"\ndescription = \"An option\"\ntype = \"boolean\"\n")
        })
        .collect::<Vec<String>>()
        .join("\n");
    ("content/options.toml", text)
}

fn read(repository: &Repository, path: &str) -> String {
    fs::read_to_string(repository.path(path)).expect("a built file")
}

fn built_with_memory_gated() -> Repository {
    let repository = Repository::with(&[
        contract(
            "[gated]\nMemory = \"memory\"\n",
            "# Working Contract\n\nThe preamble.\n\n## A\n\nRule A.\n\n## Memory\n\nUse the notes.\n\n## B\n\nRule B.\n\n### B.1\n\nA detail.\n\n## C\n\n```sh\n## not a heading\n```\n",
        ),
        options(&["contract", "memory"]),
    ]);
    assert_succeeds(&repository.run("build"));
    repository
}

#[test]
fn the_contract_renders_into_parts_gated_files_and_session_start_hooks() {
    let repository = built_with_memory_gated();

    let mut files: Vec<String> = fs::read_dir(repository.path("dist/claude-code/contract"))
        .expect("the contract folder")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    files.sort();
    assert_eq!(files, ["memory.md", "part-1.md"]);
    assert_eq!(
        read(&repository, "dist/claude-code/contract/part-1.md"),
        "# Working Contract (1/1)\n\nThe preamble.\n\n## A\n\nRule A.\n\n## B\n\nRule B.\n\n### B.1\n\nA detail.\n\n## C\n\n```sh\n## not a heading\n```\n"
    );
    assert_eq!(
        read(&repository, "dist/claude-code/contract/memory.md"),
        "# Working Contract (Memory)\n\n## Memory\n\nUse the notes.\n"
    );
    let hooks: serde_json::Value =
        serde_json::from_str(&read(&repository, "dist/claude-code/hooks/hooks.json"))
            .expect("valid JSON");
    let memory_hook = |variables: &str, subcommand: &str| {
        serde_json::json!({
            "type": "command",
            "command": format!("[ \"$CLAUDE_PLUGIN_OPTION_MEMORY\" = false ] || TIGHTSHIP_VAULT=\"$CLAUDE_PLUGIN_OPTION_VAULT_FOLDER\" {variables}\"${{CLAUDE_PLUGIN_ROOT}}/bin/tightship\" hook {subcommand}"),
        })
    };
    assert_eq!(
        hooks,
        serde_json::json!({
            "hooks": {
                "SessionStart": [{
                    "matcher": "startup|clear|compact",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "[ \"$CLAUDE_PLUGIN_OPTION_CONTRACT\" = false ] || cat \"${CLAUDE_PLUGIN_ROOT}/contract/part-1.md\"",
                        },
                        {
                            "type": "command",
                            "command": "[ \"$CLAUDE_PLUGIN_OPTION_CONTRACT\" = false ] || [ \"$CLAUDE_PLUGIN_OPTION_MEMORY\" = false ] || cat \"${CLAUDE_PLUGIN_ROOT}/contract/memory.md\"",
                        },
                    ],
                }, {
                    "matcher": "startup|clear|compact",
                    "hooks": [memory_hook("TIGHTSHIP_ENV_FILE=\"$CLAUDE_ENV_FILE\" ", "start")],
                }, {
                    "matcher": "resume|fork",
                    "hooks": [memory_hook("TIGHTSHIP_ENV_FILE=\"$CLAUDE_ENV_FILE\" ", "export")],
                }, {
                    "matcher": "compact",
                    "hooks": [memory_hook("", "checkpoint")],
                }],
                "UserPromptSubmit": [{
                    "hooks": [memory_hook("TIGHTSHIP_DATA=\"$CLAUDE_PLUGIN_DATA\" ", "prompt")],
                }],
            },
        })
    );
}

#[test]
fn the_hooks_honor_the_options() {
    let repository = built_with_memory_gated();
    let hooks: serde_json::Value =
        serde_json::from_str(&read(&repository, "dist/claude-code/hooks/hooks.json"))
            .expect("valid JSON");
    let commands: Vec<&str> = hooks["hooks"]["SessionStart"][0]["hooks"]
        .as_array()
        .expect("the hooks")
        .iter()
        .map(|hook| hook["command"].as_str().expect("a command"))
        .collect();
    assert_eq!(commands.len(), 2);
    let part = read(&repository, "dist/claude-code/contract/part-1.md");
    let memory = read(&repository, "dist/claude-code/contract/memory.md");
    fs::create_dir_all(repository.path("vault")).expect("a vault");
    let run = |command: &str, variables: &[(&str, &str)]| {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .env("CLAUDE_PLUGIN_ROOT", repository.path("dist/claude-code"))
            .env(
                "CLAUDE_PLUGIN_OPTION_VAULT_FOLDER",
                repository.path("vault"),
            )
            .env("CLAUDE_ENV_FILE", repository.path("env-file"))
            .env("CLAUDE_PLUGIN_DATA", repository.path("data"))
            .env_remove("CLAUDE_PLUGIN_OPTION_CONTRACT")
            .env_remove("CLAUDE_PLUGIN_OPTION_MEMORY")
            .envs(variables.iter().copied())
            .output()
            .expect("sh runs");
        assert_eq!(output.status.code(), Some(0), "{command}");
        String::from_utf8(output.stdout).expect("UTF-8 output")
    };

    assert_eq!(run(commands[0], &[]), part);
    assert_eq!(run(commands[1], &[]), memory);
    let memory_off = [("CLAUDE_PLUGIN_OPTION_MEMORY", "false")];
    assert_eq!(run(commands[0], &memory_off), part);
    assert_eq!(run(commands[1], &memory_off), "");
    let contract_off = [("CLAUDE_PLUGIN_OPTION_CONTRACT", "false")];
    assert_eq!(run(commands[0], &contract_off), "");
    assert_eq!(run(commands[1], &contract_off), "");

    let memory_lines: Vec<&str> = hooks["hooks"]["SessionStart"]
        .as_array()
        .expect("the SessionStart groups")
        .iter()
        .chain(
            hooks["hooks"]["UserPromptSubmit"]
                .as_array()
                .expect("the UserPromptSubmit groups"),
        )
        .skip(1)
        .map(|group| group["hooks"][0]["command"].as_str().expect("a command"))
        .collect();
    assert_eq!(memory_lines.len(), 4);
    for line in &memory_lines {
        assert_eq!(run(line, &memory_off), "", "{line}");
    }
    assert!(!repository.path("env-file").exists());
    assert!(!repository.path("data").exists());
    assert!(run(memory_lines[0], &[]).starts_with("# Vault — curated cross-harness memory\n"));
}

#[test]
fn sections_overflow_into_a_second_part_counted_in_utf16() {
    let repository = Repository::with(&[contract(
        "",
        &format!(
            "# Working Contract\n\nThe preamble.\n\n## A\n\n{}\n\n## B\n\n{}\n",
            "a".repeat(6000),
            "é".repeat(5000)
        ),
    )]);

    assert_succeeds(&repository.run("build"));
    let first = read(&repository, "dist/claude-code/contract/part-1.md");
    let second = read(&repository, "dist/claude-code/contract/part-2.md");
    assert!(first.starts_with("# Working Contract (1/2)\n\nThe preamble.\n\n## A\n"));
    assert!(second.starts_with("# Working Contract (2/2)\n\n## B\n"));
    assert!(!first.contains("## B"));
    assert!(first.encode_utf16().count() <= 10000);
    assert!(second.encode_utf16().count() <= 10000);
}

#[test]
fn a_section_too_long_fails_and_leaves_no_plugin() {
    let repository = Repository::with(&[contract(
        "",
        &format!(
            "# Working Contract\n\nThe preamble.\n\n## Code I write\n\n{}\n",
            "a".repeat(10001)
        ),
    )]);
    fs::create_dir_all(repository.path("dist/claude-code")).expect("a previous build");

    assert_fails_with(
        &repository.run("build"),
        "error: content/contract.md:5: the section \"Code I write\" renders to 10045 characters with its title; claude-code takes at most 10000 per hook",
    );
    assert!(!repository.path("dist/claude-code").exists());
}

#[test]
fn a_gated_file_too_long_fails() {
    let memory = format!("## Memory\n\n{}", "a".repeat(6000));
    let vault = format!("## Vault\n\n{}", "b".repeat(6000));
    let repository = Repository::with(&[
        contract(
            "[gated]\nMemory = \"memory\"\nVault = \"memory\"\n",
            &format!("# Working Contract\n\n{memory}\n\n{vault}\n"),
        ),
        options(&["memory"]),
    ]);
    let rendered = format!("# Working Contract (Memory, Vault)\n\n{memory}\n\n{vault}\n");

    assert_fails_with(
        &repository.run("build"),
        &format!(
            "error: content/contract.md: the sections gated by \"memory\" render to {} characters with their title; claude-code takes at most 10000 per hook",
            rendered.encode_utf16().count()
        ),
    );
}

#[test]
fn a_gate_on_a_missing_section_fails() {
    let repository = Repository::with(&[
        contract(
            "[gated]\nMemories = \"memory\"\n",
            "# Working Contract\n\n## Memory\n\nUse the notes.\n",
        ),
        options(&["memory"]),
    ]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/contract.md: front matter: [gated] names the section \"Memories\", which the file does not have",
    );
}

#[test]
fn a_gate_on_an_undeclared_option_fails() {
    let repository = Repository::with(&[
        contract(
            "[gated]\nMemory = \"vault\"\n",
            "# Working Contract\n\n## Memory\n\nUse the notes.\n",
        ),
        options(&["memory"]),
    ]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/contract.md: front matter: the section \"Memory\" is gated by \"vault\", which content/options.toml does not declare",
    );
}

#[test]
fn a_contract_with_no_title_fails() {
    let repository = Repository::with(&[contract("", "## How I work\n\nRule.\n")]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/contract.md: the contract opens with no # title",
    );
}

#[test]
fn an_unknown_front_matter_key_fails() {
    let repository = Repository::with(&[contract(
        "order = 1\n",
        "# Working Contract\n\n## A\n\nRule A.\n",
    )]);

    assert_fails_with(
        &repository.run("build"),
        "error: content/contract.md: front matter: unknown key \"order\"",
    );
}

#[test]
fn a_harness_with_no_limit_fails() {
    let repository = Repository::with(&[
        contract("", "# Working Contract\n\n## A\n\nRule A.\n"),
        (
            "harnesses/claude-code/harness.toml",
            HARNESS.replace("[contract]\nlimit = 10000\n", ""),
        ),
    ]);

    assert_fails_with(
        &repository.run("build"),
        "error: harnesses/claude-code/harness.toml: content/contract.md exists, so [contract] limit is required",
    );
}

#[test]
fn no_contract_writes_no_hooks() {
    let repository = Repository::with(&[options(&["contract"])]);

    assert_succeeds(&repository.run("build"));
    assert!(repository.path("dist/claude-code/.claude-plugin").is_dir());
    assert!(!repository.path("dist/claude-code/contract").exists());
    assert!(!repository.path("dist/claude-code/hooks").exists());
}
