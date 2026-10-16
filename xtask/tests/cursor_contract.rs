mod common;

use std::fs;
use std::path::{Path, PathBuf};
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

fn built_with_memory_gated() -> Repository {
    let harness_files = harness_files("cursor");
    let mut files: Vec<(&str, String)> = harness_files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    files.extend([
        contract(
            "[gated]\nMemory = \"memory\"\n",
            "# Working Contract\n\nThe preamble.\n\n## A\n\nRule A.\n\n## Memory\n\nUse the notes.\n\n## B\n\nRule B.\n",
        ),
        options(&["contract", "memory"]),
    ]);
    let repository = Repository::with(&files);
    assert_succeeds(&repository.run("build"));
    repository
}

fn host_binary(repository: &Repository) -> Option<PathBuf> {
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", _) => "macos",
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("windows", "x86_64") => "windows-x64",
        _ => return None,
    };
    let file_name = if platform == "windows-x64" {
        "tightship.exe"
    } else {
        "tightship"
    };
    let relative = format!("dist/cursor/bin/{platform}/{file_name}");
    let path = repository.path(&relative);
    path.is_file().then_some(path)
}

fn write_cursor_options(home: &Path, json: &str) {
    let folder = home.join(".cursor/tightship");
    fs::create_dir_all(&folder).expect("options folder");
    fs::write(folder.join("options.json"), json).expect("options.json");
}

fn run_contract_part(
    repository: &Repository,
    file: &str,
    home: &Path,
    options_json: Option<&str>,
) -> String {
    let binary = host_binary(repository).expect("a built binary for this platform");
    if let Some(json) = options_json {
        write_cursor_options(home, json);
    }
    let output = Command::new(&binary)
        .args(["hook", "contract-part", file])
        .env("CURSOR_PLUGIN_ROOT", repository.path("dist/cursor"))
        .env("HOME", home)
        .env("USERPROFILE", home)
        .output()
        .expect("contract-part runs");
    assert_eq!(output.status.code(), Some(0), "{file}");
    String::from_utf8(output.stdout).expect("UTF-8 stdout")
}

#[test]
fn cursor_contract_hooks_call_contract_part_on_session_start_and_every_prompt() {
    let repository = built_with_memory_gated();
    let hooks: serde_json::Value =
        serde_json::from_str(&read(&repository, "dist/cursor/hooks/hooks.json"))
            .expect("valid JSON");
    let binary = "${CURSOR_PLUGIN_ROOT}/bin/tightship";
    let part = format!(
        "{binary} options disabled contract || {binary} hook contract-part part-1.md"
    );
    let memory = format!(
        "{binary} options disabled contract || {binary} options disabled memory || {binary} hook contract-part memory.md"
    );
    assert_eq!(hooks["version"], serde_json::json!(1));
    assert_eq!(
        hooks["hooks"]["sessionStart"],
        serde_json::json!([{ "command": part }, { "command": memory }])
    );
    assert_eq!(
        hooks["hooks"]["beforeSubmitPrompt"],
        hooks["hooks"]["sessionStart"]
    );
    let Some(_) = host_binary(&repository) else {
        return;
    };
    let home = repository.path("home-default");
    let stdout = run_contract_part(&repository, "part-1.md", &home, None);
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one JSON line");
    assert!(value["additional_context"]
        .as_str()
        .is_some_and(|text| text.contains("Rule A.")));
}

#[test]
fn cursor_contract_hooks_honor_the_options() {
    let repository = built_with_memory_gated();
    let part = read(&repository, "dist/cursor/contract/part-1.md");
    let memory = read(&repository, "dist/cursor/contract/memory.md");
    let Some(_) = host_binary(&repository) else {
        return;
    };
    let home = repository.path("home-options");

    assert_eq!(run_contract_part(&repository, "part-1.md", &home, None), json_line(&part));
    assert_eq!(
        run_contract_part(&repository, "memory.md", &home, None),
        json_line(&memory)
    );

    let memory_off = r#"{"memory":false}"#;
    assert_eq!(
        run_contract_part(&repository, "part-1.md", &home, Some(memory_off)),
        json_line(&part)
    );
    assert_eq!(
        run_contract_part(&repository, "memory.md", &home, Some(memory_off)),
        ""
    );

    let contract_off = r#"{"contract":false}"#;
    assert_eq!(
        run_contract_part(&repository, "part-1.md", &home, Some(contract_off)),
        ""
    );
    assert_eq!(
        run_contract_part(&repository, "memory.md", &home, Some(contract_off)),
        ""
    );
}

fn json_line(text: &str) -> String {
    let payload = serde_json::json!({ "additional_context": text });
    serde_json::to_string(&payload).expect("JSON") + "\n"
}
