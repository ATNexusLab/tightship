//! The contract parts and how the options gate them.

use super::*;

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

    let output = contract_part(&sandbox, "part-1.md");
    assert_succeeds(&output);
    let stdout = stdout_of(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(
        json["injectSteps"][0]["systemMessage"]["systemMessage"],
        "# Contract (1/2)\nRule 1: Always verify.\n"
    );

    let output = contract_part(&sandbox, "memory.md");
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
        .expect("the injected message");
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

    let output = contract_part(&sandbox, "part-1.md");
    assert_succeeds(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout_of(&output)).expect("valid JSON");
    assert_eq!(
        json["injectSteps"][0]["systemMessage"]["systemMessage"],
        "# Contract (1/2)\nRule 1\n"
    );

    let output = contract_part(&sandbox, "memory.md");
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
    let output = contract_part(&sandbox, "part-1.md");
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "");

    let output = contract_part(&sandbox, "memory.md");
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
