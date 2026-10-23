mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use common::{Repository, assert_succeeds, harness_files};

fn built() -> Repository {
    let harness_files = harness_files("antigravity-cli");
    let mut files: Vec<(&str, String)> = harness_files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    files.extend([
        (
            "content/contract.md",
            "+++\n[gated]\nMemory = \"memory\"\n+++\n\n# Working Contract\n\nThe preamble.\n\n## A\n\nRule A.\n\n## Memory\n\nUse the notes.\n".to_owned(),
        ),
        (
            "content/options.toml",
            ["contract", "memory", "status_line", "auto_update"]
                .iter()
                .map(|name| format!("[[options]]\nname = \"{name}\"\ntitle = \"An option\"\ndescription = \"An option\"\ntype = \"boolean\"\n"))
                .collect::<Vec<String>>()
                .join("\n"),
        ),
    ]);
    let repository = Repository::with(&files);
    assert_succeeds(&repository.run("build"));
    repository
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("target folder");
    for entry in fs::read_dir(source).expect("source folder") {
        let entry = entry.expect("entry");
        let copy = target.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_tree(&entry.path(), &copy);
        } else {
            fs::copy(entry.path(), &copy).expect("copied file");
        }
    }
}

fn hooks(plugin: &Path, file: &str) -> serde_json::Value {
    let text = fs::read_to_string(plugin.join(file)).expect("a hooks file");
    serde_json::from_str(&text).expect("valid JSON")
}

#[test]
fn the_hooks_name_the_launcher_relative_to_the_plugin_folder_without_quotes() {
    let repository = built();
    let plugin = repository.path("dist/antigravity-cli");
    for (file, launcher) in [
        ("hooks.json", "bin/tightship"),
        ("hooks.windows.json", r"bin\tightship"),
    ] {
        let command = |arguments: &str| serde_json::json!({ "type": "command", "command": format!("{launcher} {arguments}") });
        assert_eq!(
            hooks(&plugin, file),
            serde_json::json!({
                "tightship": {
                    "SessionStart": [
                        command("hook contract-part part-1.md"),
                        command("hook contract-part memory.md"),
                        command("hook start"),
                        command("hook status-line"),
                        command("hook update"),
                    ],
                }
            }),
            "{file}"
        );
    }
}

#[test]
fn every_contract_hook_runs_through_the_shell_from_a_plugin_folder_with_a_space() {
    let repository = built();
    let plugin = repository.path("home/plugins with space/tightship");
    copy_tree(&repository.path("dist/antigravity-cli"), &plugin);
    let file = if cfg!(windows) {
        "hooks.windows.json"
    } else {
        "hooks.json"
    };
    let hooks = hooks(&plugin, file);
    let contract_hooks = hooks["tightship"]["SessionStart"]
        .as_array()
        .expect("SessionStart hooks")
        .iter()
        .filter_map(|hook| hook["command"].as_str())
        .filter(|command| command.contains("contract-part"));
    let mut outputs = Vec::new();
    for command in contract_hooks {
        let mut shell = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.arg("/c");
            cmd
        } else {
            let mut sh = Command::new("sh");
            sh.arg("-c");
            sh
        };
        let output = shell
            .arg(command)
            .current_dir(&plugin)
            .env("HOME", repository.path("home"))
            .env("USERPROFILE", repository.path("home"))
            .env_remove("TIGHTSHIP_ROOT")
            .env_remove("TIGHTSHIP_DATA")
            .env_remove("TIGHTSHIP_HOOK_OUTPUT")
            .output()
            .expect("the shell runs");
        assert!(
            output.status.success(),
            "{command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        outputs.push(String::from_utf8(output.stdout).expect("UTF-8 stdout"));
    }
    assert_eq!(outputs.len(), 2);
    assert!(outputs[0].contains("Rule A."), "{}", outputs[0]);
    assert!(outputs[1].contains("Use the notes."), "{}", outputs[1]);
}
