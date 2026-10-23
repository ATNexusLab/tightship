use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::binary::Binary;
use super::content::{Content, Kind, Piece, split_front_matter};
use super::contract::Part;
use super::harness::Harness;
use super::options::OptionsFile;
use super::render;
use super::staging;

pub fn render(
    root: &Path,
    harness: &Harness,
    content: &Content,
    parts: &[Part],
    binaries: &[Binary],
) -> io::Result<(PathBuf, usize)> {
    let staging = staging::staging_dir(root, &harness.name);
    staging::remove_if_present(&staging)?;
    let mut written = vec![(
        staging.join(&harness.dist.manifest),
        manifest(root, harness)?.into_bytes(),
    )];

    for piece in &content.pieces {
        let file = match piece.kind {
            Kind::Agent => staging.join("agents").join(format!("{}.md", piece.name)),
            Kind::Skill | Kind::Command => {
                staging.join("skills").join(&piece.name).join("SKILL.md")
            }
        };
        written.push((file, document(piece, harness)?.into_bytes()));
    }

    written.extend(render::supporting_files(&staging, content, harness)?);

    for part in parts {
        let file = staging.join("contract").join(&part.file);
        written.push((file, part.text.clone().into_bytes()));
    }

    if let Some(contract) = &content.contract {
        let skill_file = staging.join("skills/contract/SKILL.md");
        let skill_content = contract_skill(contract, harness)?;
        written.push((skill_file, skill_content.into_bytes()));
    }

    for (file, launcher) in [
        ("hooks.json", "bin/tightship"),
        ("hooks.windows.json", r"bin\tightship"),
    ] {
        if let Some(hooks) = hooks(parts, content.options.as_ref(), launcher)? {
            written.push((staging.join(file), hooks.into_bytes()));
        }
    }

    written.extend(render::harness_files(root, &staging, harness)?);
    written.extend(render::binary_files(&staging, binaries)?);
    staging::write_files(&staging, &harness.name, &written)?;
    Ok((staging, written.len()))
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

fn manifest(root: &Path, harness: &Harness) -> io::Result<String> {
    #[derive(Serialize)]
    struct Author<'harness> {
        name: &'harness str,
    }
    #[derive(Serialize)]
    struct PluginManifest<'harness> {
        name: &'harness str,
        version: String,
        description: &'harness str,
        author: Author<'harness>,
        repository: &'harness str,
        license: &'harness str,
    }
    let version = render::workspace_version(root)?;
    let plugin = &harness.plugin;
    let manifest = PluginManifest {
        name: &plugin.name,
        version,
        description: &plugin.description,
        author: Author {
            name: &plugin.author,
        },
        repository: &plugin.repository,
        license: &plugin.license,
    };
    Ok(serde_json::to_string_pretty(&manifest).map_err(io::Error::other)? + "\n")
}

fn hooks(
    parts: &[Part],
    options: Option<&OptionsFile>,
    launcher: &str,
) -> io::Result<Option<String>> {
    let declares =
        |name| options.is_some_and(|file| file.declared.iter().any(|option| option.name == name));
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
    if declares("memory") {
        session_hooks.push(command("hook start".to_owned()));
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
    Ok(Some(
        serde_json::to_string_pretty(&hooks).map_err(io::Error::other)? + "\n",
    ))
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
