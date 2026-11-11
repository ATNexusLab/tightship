use std::io;
use std::path::{Path, PathBuf};

use super::binary::Binary;
use super::content::{Content, Kind, Piece, split_front_matter};
use super::contract::Part;
use super::harness::Harness;
use super::options::{self, OptionsFile};
use super::render;

pub fn render(
    root: &Path,
    harness: &Harness,
    content: &Content,
    parts: &[Part],
    binaries: &[Binary],
) -> io::Result<(PathBuf, usize)> {
    let options = content.options.as_ref();
    let mut extra = Vec::new();
    if let Some(contract) = &content.contract {
        extra.push((
            "skills/contract/SKILL.md".to_owned(),
            contract_skill(contract, harness)?,
        ));
    }
    for (file, launcher) in [
        ("hooks.json", "bin/tightship"),
        ("hooks.windows.json", r"bin\tightship"),
    ] {
        if let Some(hooks) = hooks(parts, options, launcher, harness.clean.is_some())? {
            extra.push((file.to_owned(), hooks));
        }
    }
    let layout = render::Layout {
        manifest: render::manifest(root, harness, None, None)?,
        commands_as_files: false,
        document,
        extra,
    };
    render::assemble(root, harness, content, parts, binaries, layout)
}

fn contract_skill(contract: &super::contract::Contract, harness: &Harness) -> io::Result<String> {
    let body = match split_front_matter(&contract.text) {
        Some((_, body, _)) => body,
        None => &contract.text,
    };
    let resolved = render::resolve(body, &contract.path, harness)?;
    let mut document = String::from("---\n");
    document.push_str("name: \"contract\"\n");
    document.push_str("description: \"Reload the workflow's global working contract\"\n");
    document.push_str("---\n\n");
    document.push_str(resolved.trim_start_matches('\n'));
    Ok(document)
}

fn hooks(
    parts: &[Part],
    options: Option<&OptionsFile>,
    launcher: &str,
    clean: bool,
) -> io::Result<Option<String>> {
    let declares = |name| options::declared(options, name);
    let command = |arguments: String| {
        serde_json::json!({
            "type": "command",
            "command": format!("{launcher} {arguments}"),
        })
    };
    let mut session_hooks: Vec<serde_json::Value> = parts
        .iter()
        .map(|part| command(format!("hook contract-part {}", part.file)))
        .collect();
    if !parts.is_empty() && declares("contract") {
        session_hooks.push(command("hook map".to_owned()));
    }
    if declares("memory") {
        session_hooks.push(command("hook start".to_owned()));
    }
    if clean {
        session_hooks.push(command("hook clean".to_owned()));
    }
    if declares("status_line") {
        session_hooks.push(command("hook status-line".to_owned()));
    }
    if declares("auto_update") {
        session_hooks.push(command("hook update".to_owned()));
    }

    if session_hooks.is_empty() {
        return Ok(None);
    }

    let hooks = serde_json::json!({ "tightship": { "SessionStart": session_hooks } });
    render::pretty_json(&hooks).map(Some)
}

fn document(piece: &Piece, harness: &Harness) -> io::Result<String> {
    let front_matter = piece.front_matter.as_ref().ok_or_else(|| {
        io::Error::other(format!(
            "{}: rendered with invalid front matter",
            piece.path
        ))
    })?;

    let mut document = String::from("---\n");
    document.push_str(&format!(
        "name: {}\n",
        serde_json::Value::from(piece.name.as_str())
    ));
    document.push_str(&format!(
        "description: {}\n",
        serde_json::Value::from(front_matter.description.as_str())
    ));

    if piece.kind == Kind::Agent {
        document.push_str("tools:\n");
        for tool in harness
            .access
            .read_only
            .split(',')
            .map(str::trim)
            .filter(|tool| !tool.is_empty())
        {
            document.push_str(&format!("  - {tool}\n"));
        }
        document.push_str("subagent: true\n");
        document.push_str("mainAgent: false\n");
        document.push_str("model: inherit\n");
    }

    document.push_str("---\n");
    document.push_str(&render::resolve(&piece.body, &piece.path, harness)?);
    Ok(document)
}
