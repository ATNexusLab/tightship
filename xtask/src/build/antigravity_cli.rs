use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::binary::Binary;
use super::content::{Content, Kind, Piece, SupportingBody, split_front_matter};
use super::contract::Part;
use super::harness::Harness;
use super::options::OptionsFile;
use super::staging;
use super::tokens;
use crate::diagnostics::naming;

const LAUNCHER_SH: &str = r#"#!/bin/sh
system=$(uname -s)
machine=$(uname -m)
case "$system $machine" in
Darwin\ *) platform=macos ;;
Linux\ x86_64) platform=linux-x64 ;;
Linux\ aarch64 | Linux\ arm64) platform=linux-arm64 ;;
MINGW*\ x86_64 | MSYS*\ x86_64 | CYGWIN*\ x86_64) platform=windows-x64 ;;
MINGW*\ aarch64 | MSYS*\ aarch64 | CYGWIN*\ aarch64) platform=windows-x64 ;;
MINGW*\ arm64 | MSYS*\ arm64 | CYGWIN*\ arm64) platform=windows-x64 ;;
*) platform=unknown ;;
esac

binary="$(dirname "$0")/$platform/tightship"
[ "$platform" = windows-x64 ] && binary="$binary.exe"
if [ ! -f "$binary" ]; then
  echo "error: tightship has no binary for $system $machine" >&2
  exit 1
fi
[ -x "$binary" ] || chmod +x "$binary" "$0"

plugin_dir="$(cd "$(dirname "$0")/.." && pwd)"
export TIGHTSHIP_ROOT="${TIGHTSHIP_ROOT:-${PLUGIN_ROOT:-$plugin_dir}}"
export TIGHTSHIP_DATA="${TIGHTSHIP_DATA:-${PLUGIN_DATA:-$HOME/.gemini/config/plugin_data/tightship}}"
export TIGHTSHIP_SETTINGS="${TIGHTSHIP_SETTINGS:-${AGY_CONFIG_DIR:-$HOME/.gemini/antigravity-cli}/settings.json}"
export TIGHTSHIP_HOOK_FORMAT="${TIGHTSHIP_HOOK_FORMAT:-inject-steps}"

exec "$binary" "$@"
"#;

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

    for supporting in &content.supporting {
        let bytes = match &supporting.body {
            SupportingBody::Markdown(text) => {
                resolve(text, &supporting.path, harness)?.into_bytes()
            }
            SupportingBody::Copied(bytes) => bytes.clone(),
        };
        let file = staging
            .join("skills")
            .join(&supporting.skill)
            .join(&supporting.relative);
        written.push((file, bytes));
    }

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
    written.push((launcher_posix.clone(), LAUNCHER_SH.as_bytes().to_vec()));

    let launcher_cmd = staging.join("bin/tightship.cmd");
    written.push((
        launcher_cmd.clone(),
        include_bytes!("../../../tightship/launcher.cmd").to_vec(),
    ));

    let mut executables = vec![launcher_posix, launcher_cmd];
    for binary in binaries {
        let file_name = binary.path.file_name().unwrap_or_default();
        let copy = staging.join("bin").join(binary.platform).join(file_name);
        let bytes = fs::read(&binary.path).map_err(naming(&binary.path))?;
        written.push((copy.clone(), bytes));
        executables.push(copy);
    }

    staging::write_files(&written, &executables)?;
    Ok((staging, written.len()))
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
    let resolved = resolve(body, &contract.path, harness)?;
    let mut document = String::from("---\n");
    document.push_str("name: \"contract\"\n");
    document.push_str("description: \"Reload the workflow's global working contract\"\n");
    document.push_str("---\n\n");
    document.push_str(resolved.trim_start_matches('\n'));
    Ok(document)
}

fn manifest(root: &Path, harness: &Harness) -> io::Result<String> {
    #[derive(Deserialize)]
    struct CargoManifest {
        workspace: Workspace,
    }
    #[derive(Deserialize)]
    struct Workspace {
        package: Package,
    }
    #[derive(Deserialize)]
    struct Package {
        version: String,
    }
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
    let cargo_path = root.join("Cargo.toml");
    let cargo_text = fs::read_to_string(&cargo_path).map_err(naming(&cargo_path))?;
    let cargo: CargoManifest = toml::from_str(&cargo_text)
        .map_err(|error| io::Error::other(format!("Cargo.toml: {}", error.message())))?;
    let plugin = &harness.plugin;
    let manifest = PluginManifest {
        name: &plugin.name,
        version: cargo.workspace.package.version,
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
    document.push_str(&resolve(&piece.body, &piece.path, harness)?);
    Ok(document)
}

fn resolve(text: &str, path: &str, harness: &Harness) -> io::Result<String> {
    let (rendered, unresolved) = tokens::render(text, harness);
    if unresolved.is_empty() {
        Ok(rendered)
    } else {
        Err(io::Error::other(format!(
            "{path}: rendered with unresolved tokens {}",
            unresolved.join(", ")
        )))
    }
}
