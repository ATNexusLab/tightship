#[allow(dead_code)]
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use common::{Sandbox, assert_succeeds, note, run, stderr_of, stdout_of};

fn project(sandbox: &Sandbox, name: &str) -> PathBuf {
    let folder = sandbox.path(name);
    fs::create_dir_all(folder.join(".git")).expect("the project's .git");
    folder
}

fn hook(sandbox: &Sandbox, subcommand: &str, working_directory: &Path) -> Command {
    let mut command = sandbox.tightship(&["hook", subcommand]);
    command
        .current_dir(working_directory)
        .env("TIGHTSHIP_VAULT", sandbox.path("vault"))
        .env("TIGHTSHIP_ENV_FILE", sandbox.path("env-file"))
        .env("TIGHTSHIP_DATA", sandbox.path("data"));
    command
}

fn hook_output(sandbox: &Sandbox) -> PathBuf {
    sandbox.write(
        "hook-output.json",
        include_str!("../../harnesses/antigravity-cli/plugin/hook-output.json"),
    )
}

fn export_line(sandbox: &Sandbox) -> String {
    format!(
        "export TIGHTSHIP_VAULT='{}'\n",
        sandbox.path("vault").display()
    )
}

#[test]
fn start_lists_the_pinned_notes_of_the_project_then_all() {
    let sandbox = Sandbox::new();
    note(
        &sandbox,
        "tightship",
        "mem-0002",
        "pinned: true\n",
        "Pinned here.",
    );
    note(&sandbox, "tightship", "mem-0003", "", "Not pinned.");
    note(
        &sandbox,
        "all",
        "mem-0001",
        "pinned: true\n",
        "Pinned everywhere.",
    );
    note(
        &sandbox,
        "other",
        "mem-0004",
        "pinned: true\n",
        "Pinned elsewhere.",
    );
    let working_directory = project(&sandbox, "tightship");
    fs::create_dir_all(working_directory.join("src")).expect("a subfolder");

    let output = run(&mut hook(&sandbox, "start", &working_directory.join("src")));

    assert_succeeds(&output);
    let vault = sandbox.path("vault");
    let vault = vault.display();
    assert_eq!(
        stdout_of(&output),
        format!(
            "# Vault — curated cross-harness memory\n\n\
             Scope `tightship` and `all`. Before non-trivial work, search them for the subsystem and the task:\n\
             `tightship vault search <term> <term>` — one line per match, its `why` stating\n\
             what not knowing it costs. Open one at `{vault}/<path>`. Rules: `{vault}/AGENTS.md`.\n\
             Listed below, only the pinned notes. A hint to verify against the repo, never truth.\n\
             Write with `tightship vault new --help`.\n\n\
             ## Scope — tightship\n\n\
             - [[mem-0002-a-note]] (2026-10-01) — the mem-0002 why costs real time\n\n\
             ## Scope — all\n\n\
             - [[mem-0001-a-note]] (2026-10-01) — the mem-0001 why costs real time\n\n\
             [1 more notes in these scopes — `tightship vault search <terms>`]\n\
             [other scopes: other(1) — `tightship vault search --everywhere <terms>`]\n"
        )
    );
    let environment = fs::read_to_string(sandbox.path("env-file")).expect("the environment file");
    assert!(
        environment.ends_with(&export_line(&sandbox)),
        "{environment}"
    );
}

#[test]
fn start_shows_the_handoff_apart_from_the_pinned_notes() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "vault/notes/tightship/mem-0005-handoff.md",
        "---\nid: mem-0005\ntype: handoff\nproject: tightship\nwhy: the next session picks up here\npinned: true\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-14\n---\n\n# Handoff\n\nStep 3 of 5.\n",
    );
    let working_directory = project(&sandbox, "tightship");

    let output = run(&mut hook(&sandbox, "start", &working_directory));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains(
            "\n\n## Handoff — tightship · mem-0005 · 2026-10-01 → 2026-10-14\n\nClaim it with the handoff command once you pick the work up.\n\n# Handoff\n\nStep 3 of 5.\n"
        ),
        "{stdout}"
    );
    assert!(!stdout.contains("## Scope —"), "{stdout}");
}

#[test]
fn start_cuts_the_pinned_notes_at_the_cap() {
    let sandbox = Sandbox::new();
    let body = "x".repeat(10);
    for number in 1..=40 {
        sandbox.write(
            &format!("vault/notes/tightship/mem-{number:04}-long.md"),
            &format!(
                "---\nid: mem-{number:04}\ntype: gotcha\nproject: tightship\nwhy: {}\npinned: true\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-01\n---\n\n{body}\n",
                "é".repeat(260)
            ),
        );
    }
    let working_directory = project(&sandbox, "tightship");

    let output = run(&mut hook(&sandbox, "start", &working_directory));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(stdout.encode_utf16().count() <= 9000, "{}", stdout.len());
    let listed = stdout.matches("- [[mem-").count();
    assert!(listed > 0 && listed < 40, "{listed}");
    assert!(
        stdout.ends_with(&format!(
            "[{} pinned notes cut at the 9000-character cap — `tightship vault lint` names the scope]\n",
            40 - listed
        )),
        "{stdout}"
    );
}

#[test]
fn start_takes_the_scope_whose_name_prefixes_the_project_folder() {
    let sandbox = Sandbox::new();
    note(
        &sandbox,
        "tightship",
        "mem-0001",
        "pinned: true\n",
        "Pinned.",
    );
    fs::create_dir_all(sandbox.path("vault/notes/all")).expect("the all scope");
    let working_directory = project(&sandbox, "tightship-docs");

    let output = run(&mut hook(&sandbox, "start", &working_directory));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(stdout.contains("Scope `tightship` and `all`."), "{stdout}");
    assert!(stdout.contains("- [[mem-0001-a-note]]"), "{stdout}");
}

#[test]
fn start_without_a_usable_vault_says_so_and_exports_nothing() {
    let sandbox = Sandbox::new();
    let gone = sandbox.path("gone");
    let gone = gone.to_str().expect("a UTF-8 path");
    let cases = [
        (
            None,
            "Memory is on and no vault folder is set, so this session has no vault.".to_owned(),
        ),
        (
            Some("vault"),
            "Memory is on and the vault folder `vault` is not an absolute path, so this session has no".to_owned(),
        ),
        (
            Some(gone),
            format!("Memory is on and the vault folder `{gone}` does not exist, so this session has no vault."),
        ),
    ];

    for (vault, opening) in cases {
        let mut command = hook(&sandbox, "start", &sandbox.path(""));
        command.env_remove("TIGHTSHIP_VAULT");
        if let Some(vault) = vault {
            command.env("TIGHTSHIP_VAULT", vault);
        }
        let output = run(&mut command);
        assert_succeeds(&output);
        let stdout = stdout_of(&output);
        assert!(
            stdout.starts_with(&format!("# Vault\n\n{opening}")),
            "{stdout}"
        );
    }
    assert!(!sandbox.path("env-file").exists());
}

#[test]
fn start_leaves_out_files_that_are_not_notes() {
    let sandbox = Sandbox::new();
    note(
        &sandbox,
        "tightship",
        "mem-0001",
        "pinned: true\n",
        "Pinned.",
    );
    sandbox.write("vault/notes/tightship/index.md", "# Index\n\n- a link\n");
    sandbox.write(
        "vault/notes/tightship/mem-0002-open.md",
        "---\nid: mem-0002\npinned: true\n\n# No closing line\n",
    );
    let working_directory = project(&sandbox, "tightship");

    let output = run(&mut hook(&sandbox, "start", &working_directory));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(stdout.contains("- [[mem-0001-a-note]]"), "{stdout}");
    assert!(!stdout.contains("mem-0002"), "{stdout}");
    assert!(!stdout.contains("more notes"), "{stdout}");
}

#[test]
fn export_writes_the_vault_to_the_environment_and_prints_nothing() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.path("vault")).expect("the vault");

    let output = run(&mut hook(&sandbox, "export", &sandbox.path("")));

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
    let environment = fs::read_to_string(sandbox.path("env-file")).expect("the environment file");
    assert_eq!(environment, export_line(&sandbox));
}

#[test]
fn prompt_is_no_longer_a_subcommand() {
    let sandbox = Sandbox::new();

    let output = run(&mut hook(&sandbox, "prompt", &sandbox.path("")));

    common::assert_fails_with(&output, "error: unknown subcommand prompt");
}

#[test]
fn checkpoint_prints_the_checkpoint_only_with_a_vault() {
    let sandbox = Sandbox::new();

    let output = run(&mut hook(&sandbox, "checkpoint", &sandbox.path("")));

    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    assert!(
        stdout
            .starts_with("MEMORY CHECKPOINT — did anything this session become a durable fact?\n")
    );
    assert!(stdout.ends_with("Nothing durable? Ignore this and carry on.\n"));

    let mut without_vault = hook(&sandbox, "checkpoint", &sandbox.path(""));
    without_vault.env_remove("TIGHTSHIP_VAULT");
    assert_eq!(stdout_of(&run(&mut without_vault)), "");
}

#[cfg(unix)]
#[test]
fn an_unreadable_vault_or_note_or_data_folder_fails_with_exit_1() {
    use std::os::unix::fs::PermissionsExt;

    use common::assert_fails_with;

    let sandbox = Sandbox::new();
    let unreadable = fs::Permissions::from_mode(0o000);
    let readable = fs::Permissions::from_mode(0o755);
    let working_directory = project(&sandbox, "tightship");
    let vault = sandbox.path("vault");
    let secret = note(&sandbox, "tightship", "mem-0001", "", "Hidden.");

    fs::set_permissions(&vault, unreadable.clone()).expect("an unreadable vault");
    let output = run(&mut hook(&sandbox, "start", &working_directory));
    fs::set_permissions(&vault, readable).expect("a readable vault");
    assert_fails_with(
        &output,
        &format!(
            "error: cannot read the vault folder {}: Permission denied (os error 13)",
            vault.display()
        ),
    );

    fs::set_permissions(&secret, unreadable).expect("an unreadable note");
    assert_fails_with(
        &run(&mut hook(&sandbox, "start", &working_directory)),
        "error: cannot read notes/tightship/mem-0001-a-note.md: Permission denied (os error 13)",
    );

    sandbox.write("data", "a file, not a folder");
    assert_fails_with(
        &with_stdin(
            &mut hook(&sandbox, "prompt", &sandbox.path("")),
            r#"{"session_id":"abc"}"#,
        ),
        &format!(
            "error: cannot write {}: Not a directory (os error 20)",
            sandbox.path("data/checkpoints/abc").display()
        ),
    );
}

#[test]
fn scenario_4_session_start_injects_contract_and_memory_as_inject_steps_json() {
    let sandbox = Sandbox::new();
    let vault = sandbox.path("vault");
    sandbox.write(
        "data/options.toml",
        &format!(
            "contract = true\nmemory = true\nvault_folder = '{}'\n",
            vault.display()
        ),
    );
    note(
        &sandbox,
        "tightship",
        "mem-0001",
        "pinned: true\n",
        "Pinned note.",
    );
    sandbox.write(
        "vault/notes/tightship/mem-0002-handoff.md",
        "---\nid: mem-0002\ntype: handoff\nproject: tightship\nwhy: next session picks up here\npinned: true\nsource: session\ncreated: 2026-10-01\nupdated: 2026-10-14\n---\n\n# Handoff\n\n**Goal:** ship plugin\n**Done:** options\n**Next step:** tests\n**Open decisions:** none\n**Repo state:** clean\n",
    );
    let working_directory = project(&sandbox, "tightship");
    sandbox.write(
        "contract/part-1.md",
        "# Contract (1/2)\nRule 1: Always verify.\n",
    );
    sandbox.write(
        "contract/memory.md",
        "# Memory (2/2)\nRule 2: Keep the vault.\n",
    );

    let mut contract_cmd = sandbox.tightship(&["hook", "contract-part", "part-1.md"]);
    contract_cmd
        .env("TIGHTSHIP_ROOT", sandbox.path(""))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(&sandbox));
    let output = run(&mut contract_cmd);
    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(
        json["injectSteps"][0]["systemMessage"]["systemMessage"],
        "# Contract (1/2)\nRule 1: Always verify.\n"
    );

    let mut memory_cmd = sandbox.tightship(&["hook", "contract-part", "memory.md"]);
    memory_cmd
        .env("TIGHTSHIP_ROOT", sandbox.path(""))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(&sandbox));
    let output = run(&mut memory_cmd);
    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(
        json["injectSteps"][0]["systemMessage"]["systemMessage"],
        "# Memory (2/2)\nRule 2: Keep the vault.\n"
    );

    let mut start_cmd = sandbox.tightship(&["hook", "start"]);
    start_cmd
        .current_dir(&working_directory)
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_ENV_FILE", sandbox.path("env-file"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(&sandbox));
    let output = run(&mut start_cmd);
    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let msg = json["injectSteps"][0]["systemMessage"]["systemMessage"]
        .as_str()
        .unwrap();
    assert!(msg.contains("mem-0001"), "{msg}");
    assert!(msg.contains("**Goal:** ship plugin"), "{msg}");
}

#[test]
fn scenario_5_turning_contract_or_memory_off_suppresses_their_hooks() {
    let sandbox = Sandbox::new();
    let vault = sandbox.path("vault");
    sandbox.write(
        "data/options.toml",
        &format!(
            "contract = true\nmemory = false\nvault_folder = '{}'\n",
            vault.display()
        ),
    );
    let working_directory = project(&sandbox, "tightship");
    sandbox.write("contract/part-1.md", "# Contract (1/2)\nRule 1\n");
    sandbox.write("contract/memory.md", "# Memory (2/2)\nRule 2\n");

    let mut contract_cmd = sandbox.tightship(&["hook", "contract-part", "part-1.md"]);
    contract_cmd
        .env("TIGHTSHIP_ROOT", sandbox.path(""))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(&sandbox));
    let output = run(&mut contract_cmd);
    assert_succeeds(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout_of(&output)).expect("valid JSON");
    assert_eq!(
        json["injectSteps"][0]["systemMessage"]["systemMessage"],
        "# Contract (1/2)\nRule 1\n"
    );

    let mut memory_cmd = sandbox.tightship(&["hook", "contract-part", "memory.md"]);
    memory_cmd
        .env("TIGHTSHIP_ROOT", sandbox.path(""))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(&sandbox));
    let output = run(&mut memory_cmd);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");

    let mut start_cmd = sandbox.tightship(&["hook", "start"]);
    start_cmd
        .current_dir(&working_directory)
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(&sandbox));
    let output = run(&mut start_cmd);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");

    sandbox.write("data/options.toml", "contract = false\n");
    let mut contract_cmd1 = sandbox.tightship(&["hook", "contract-part", "part-1.md"]);
    contract_cmd1
        .env("TIGHTSHIP_ROOT", sandbox.path(""))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(&sandbox));
    let output = run(&mut contract_cmd1);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");

    let mut contract_cmd2 = sandbox.tightship(&["hook", "contract-part", "memory.md"]);
    contract_cmd2
        .env("TIGHTSHIP_ROOT", sandbox.path(""))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(&sandbox));
    let output = run(&mut contract_cmd2);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
}

#[test]
fn scenario_6_malformed_options_toml_fails_without_blocking_the_session() {
    let sandbox = Sandbox::new();
    let options_file = sandbox.write("data/options.toml", "contract = \"not-a-bool\"\n");
    let working_directory = project(&sandbox, "tightship");

    let mut start_cmd = sandbox.tightship(&["hook", "start"]);
    start_cmd
        .current_dir(&working_directory)
        .env("TIGHTSHIP_DATA", sandbox.path("data"));
    let output = run(&mut start_cmd);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr_of(&output),
        format!(
            "error: {} has an invalid value for contract\n",
            options_file.display()
        )
    );

    sandbox.write("data/options.toml", "contract = \n");
    let output = run(&mut start_cmd);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr_of(&output),
        format!("error: {} is not valid TOML\n", options_file.display())
    );
}

#[test]
fn contract_part_missing_file_fails_naming_it() {
    let sandbox = Sandbox::new();
    let mut command = sandbox.tightship(&["hook", "contract-part", "part-9.md"]);
    command.env("TIGHTSHIP_ROOT", sandbox.path(""));
    let output = run(&mut command);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr_of(&output),
        format!(
            "error: contract part {} is missing\n",
            sandbox.path("contract/part-9.md").display()
        )
    );
}

#[test]
fn contract_part_prints_through_the_hook_output_and_respects_the_options() {
    let sandbox = Sandbox::new();
    let plugin = sandbox.path("plugin");
    let part = plugin.join("contract/part-1.md");
    fs::create_dir_all(part.parent().expect("contract folder")).expect("folders");
    fs::write(&part, "Rule one.\n").expect("the part");
    let data = data_dir(&sandbox, "data");
    fs::write(data.join("options.json"), r#"{"contract":false}"#).expect("options");
    let mut command = sandbox.tightship(&["hook", "contract-part", "part-1.md"]);
    command
        .env("TIGHTSHIP_ROOT", &plugin)
        .env("TIGHTSHIP_DATA", &data)
        .env("TIGHTSHIP_HOOK_OUTPUT", context_output(&sandbox));
    let off = run(&mut command);
    assert_succeeds(&off);
    assert_eq!(stdout_of(&off), "");
    fs::write(
        data.join("options.json"),
        r#"{"contract":true,"memory":false}"#,
    )
    .expect("options");
    let output = run(&mut command);
    assert_succeeds(&output);
    let value: serde_json::Value =
        serde_json::from_str(stdout_of(&output).trim()).expect("one JSON line");
    assert_eq!(value["additional_context"], "Rule one.\n");
}

#[test]
fn contract_part_skips_gated_memory_when_memory_is_off() {
    let sandbox = Sandbox::new();
    let plugin = sandbox.path("plugin");
    let memory = plugin.join("contract/memory.md");
    fs::create_dir_all(memory.parent().expect("contract folder")).expect("folders");
    fs::write(&memory, "Vault rule.\n").expect("the part");
    let data = data_dir(&sandbox, "data-memory-off");
    fs::write(
        data.join("options.json"),
        r#"{"contract":true,"memory":false}"#,
    )
    .expect("options");
    let mut command = sandbox.tightship(&["hook", "contract-part", "memory.md"]);
    command
        .env("TIGHTSHIP_ROOT", &plugin)
        .env("TIGHTSHIP_DATA", &data);
    let output = run(&mut command);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");
}

fn context_output(sandbox: &Sandbox) -> PathBuf {
    sandbox.write("hook-output.json", r#"{"additional_context":"{text}"}"#)
}

fn data_dir(sandbox: &Sandbox, name: &str) -> PathBuf {
    let folder = sandbox.path(name);
    fs::create_dir_all(&folder).expect("data folder");
    folder
}

fn write_options_json(data: &Path, vault: &Path) {
    let body = serde_json::json!({
        "memory": true,
        "vault_folder": vault.to_string_lossy(),
    });
    fs::write(data.join("options.json"), body.to_string()).expect("options.json");
}

#[test]
fn start_reads_the_vault_from_options_json() {
    let sandbox = Sandbox::new();
    note(
        &sandbox,
        "tightship",
        "mem-0099",
        "pinned: true\n",
        "Shared from another harness.",
    );
    let vault = sandbox.path("vault");
    let data = data_dir(&sandbox, "data");
    write_options_json(&data, &vault);
    let working_directory = project(&sandbox, "tightship");
    let mut command = sandbox.tightship(&["hook", "start"]);
    command
        .current_dir(&working_directory)
        .env("TIGHTSHIP_DATA", &data)
        .env("TIGHTSHIP_HOOK_OUTPUT", context_output(&sandbox));
    let output = run(&mut command);
    assert_succeeds(&output);
    let value: serde_json::Value =
        serde_json::from_str(stdout_of(&output).trim()).expect("one JSON line");
    let text = value["additional_context"].as_str().expect("context");
    assert!(text.contains("[[mem-0099-a-note]]"), "{text}");
}

#[test]
fn checkpoint_prints_through_its_own_output_template() {
    let sandbox = Sandbox::new();
    let data = data_dir(&sandbox, "data");
    write_options_json(&data, &sandbox.path("vault"));
    fs::create_dir_all(sandbox.path("vault")).expect("vault");
    let checkpoint_output = sandbox.write("checkpoint-output.json", r#"{"user_message":"{text}"}"#);
    let mut command = sandbox.tightship(&["hook", "checkpoint"]);
    command
        .env("TIGHTSHIP_DATA", &data)
        .env("TIGHTSHIP_HOOK_OUTPUT", context_output(&sandbox))
        .env("TIGHTSHIP_CHECKPOINT_OUTPUT", checkpoint_output);
    let output = run(&mut command);
    assert_succeeds(&output);
    let value: serde_json::Value =
        serde_json::from_str(stdout_of(&output).trim()).expect("one JSON line");
    assert!(
        value["user_message"]
            .as_str()
            .is_some_and(|text| text.contains("MEMORY CHECKPOINT")),
        "{value}"
    );
}
