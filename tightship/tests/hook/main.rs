//! Tests of the `tightship hook` subcommands, run in a sandbox with a fixture vault.

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{Sandbox, assert_succeeds, note, project, run, stderr_of, stdout_of, with_stdin};

mod checkpoint;
mod contract_part;
mod map;
mod start;

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
        include_str!("../../../harnesses/antigravity-cli/plugin/hook-output.json"),
    )
}

fn contract_part(sandbox: &Sandbox, file: &str) -> Output {
    run(sandbox
        .tightship(&["hook", "contract-part", file])
        .env("TIGHTSHIP_ROOT", sandbox.path(""))
        .env("TIGHTSHIP_DATA", sandbox.path("data"))
        .env("TIGHTSHIP_HOOK_OUTPUT", hook_output(sandbox)))
}

fn export_line(sandbox: &Sandbox) -> String {
    format!(
        "export TIGHTSHIP_VAULT='{}'\n",
        sandbox.path("vault").display()
    )
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
