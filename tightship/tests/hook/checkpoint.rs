//! The export and checkpoint hooks.

use super::*;

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
