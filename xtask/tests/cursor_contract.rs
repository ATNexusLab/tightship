mod common;

use std::fs;
use std::process::Command;

use common::{Repository, assert_succeeds, harness_files};

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

#[test]
fn cursor_contract_hooks_call_contract_part_on_session_start_and_every_prompt() {
    let harness_files = harness_files("cursor");
    let mut files: Vec<(&str, String)> = harness_files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    files.extend([
        contract(
            "[gated]\nMemory = \"memory\"\n",
            "# Working Contract\n\nThe preamble.\n\n## A\n\nRule A.\n\n## Memory\n\nUse the notes.\n",
        ),
        options(&["contract", "memory"]),
    ]);
    let repository = Repository::with(&files);
    assert_succeeds(&repository.run("build"));
    let hooks: serde_json::Value =
        serde_json::from_str(&read(&repository, "dist/cursor/hooks/hooks.json"))
            .expect("valid JSON");
    let part = "${CURSOR_PLUGIN_ROOT}/bin/tightship hook contract-part part-1.md";
    let memory = "${CURSOR_PLUGIN_ROOT}/bin/tightship hook contract-part memory.md";
    assert_eq!(hooks["version"], serde_json::json!(1));
    assert_eq!(
        hooks["hooks"]["sessionStart"],
        serde_json::json!([{ "command": part }, { "command": memory }])
    );
    assert_eq!(
        hooks["hooks"]["beforeSubmitPrompt"],
        hooks["hooks"]["sessionStart"]
    );
    let binary = repository
        .path("dist/cursor/bin/windows-x64/tightship.exe");
    if !binary.is_file() {
        return;
    }
    let plugin_root = repository.path("dist/cursor");
    let output = Command::new(binary)
        .args(["hook", "contract-part", "part-1.md"])
        .env("CURSOR_PLUGIN_ROOT", &plugin_root)
        .output()
        .expect("contract-part runs");
    assert!(output.status.success());
    let line = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(line.trim()).expect("one JSON line");
    assert!(value["additional_context"]
        .as_str()
        .is_some_and(|text| text.contains("Rule A.")));
}
