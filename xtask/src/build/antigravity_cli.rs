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
        staging.join("plugin.json"),
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
        let text = prepare_contract_part(part);
        written.push((file, text.into_bytes()));
    }

    if let Some(contract) = &content.contract {
        let skill_file = staging.join("skills/contract/SKILL.md");
        let skill_content = contract_skill(contract, harness)?;
        written.push((skill_file, skill_content.into_bytes()));
    }

    if let Some(hooks) = hooks(parts, content.options.as_ref())? {
        written.push((staging.join("hooks.json"), hooks.into_bytes()));
    }

    let launcher_posix = staging.join("bin/tightship");
    written.push((launcher_posix.clone(), launcher_sh().into_bytes()));

    let launcher_cmd = staging.join("bin/tightship.cmd");
    written.push((
        launcher_cmd.clone(),
        include_bytes!("../../../harnesses/antigravity-cli/launcher.cmd").to_vec(),
    ));

    let mut executables = vec![launcher_posix, launcher_cmd];
    let binary_copies = render::binary_files(&staging, binaries)?;
    executables.extend(binary_copies.iter().map(|(copy, _)| copy.clone()));
    written.extend(binary_copies);

    staging::write_files(&written, &executables)?;
    Ok((staging, written.len()))
}

fn launcher_sh() -> String {
    let neutral = include_str!("../../../tightship/launcher.sh");
    let (shebang, body) = neutral.split_once('\n').unwrap_or_default();
    let environment = include_str!("../../../harnesses/antigravity-cli/launcher-env.sh");
    format!("{shebang}\n{environment}{body}")
}

fn prepare_contract_part(part: &Part) -> String {
    if part.file == "part-1.md" {
        if let Some((heading, rest)) = part.text.split_once('\n') {
            format!(
                "{heading}\n\nAfter any context compaction (/compact), load the contract skill to restore this contract in full.\n\n{}",
                rest.trim_start_matches('\n')
            )
        } else {
            part.text.clone()
        }
    } else {
        part.text.clone()
    }
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

fn hooks(parts: &[Part], options: Option<&OptionsFile>) -> io::Result<Option<String>> {
    let declares =
        |name| options.is_some_and(|file| file.declared.iter().any(|option| option.name == name));
    let mut session_hooks = Vec::new();

    for part in parts {
        let command = match &part.gate {
            Some(gate) => format!(
                "\"${{PLUGIN_ROOT}}/bin/tightship\" hook contract --gate {gate} \"${{PLUGIN_ROOT}}/contract/{}\"",
                part.file
            ),
            None => format!(
                "\"${{PLUGIN_ROOT}}/bin/tightship\" hook contract \"${{PLUGIN_ROOT}}/contract/{}\"",
                part.file
            ),
        };
        session_hooks.push(serde_json::json!({
            "type": "command",
            "command": command,
        }));
    }

    if declares("memory") {
        session_hooks.push(serde_json::json!({
            "type": "command",
            "command": "\"${PLUGIN_ROOT}/bin/tightship\" hook start",
        }));
    }

    if declares("status_line") {
        session_hooks.push(serde_json::json!({
            "type": "command",
            "command": "\"${PLUGIN_ROOT}/bin/tightship\" hook status-line",
        }));
    }

    let mut events = serde_json::Map::new();
    if !session_hooks.is_empty() {
        events.insert(
            "SessionStart".to_owned(),
            serde_json::Value::Array(session_hooks),
        );
    }

    if declares("memory") {
        events.insert(
            "PreInvocation".to_owned(),
            serde_json::json!([{
                "type": "command",
                "command": "\"${PLUGIN_ROOT}/bin/tightship\" hook prompt",
            }]),
        );
    }

    if events.is_empty() {
        return Ok(None);
    }

    let mut root = serde_json::Map::new();
    root.insert("tightship".to_owned(), serde_json::Value::Object(events));
    let hooks = serde_json::Value::Object(root);
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
