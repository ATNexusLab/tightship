use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::binary::Binary;
use super::content::{Content, Kind};
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
        manifest(root, harness, content.options.as_ref())?.into_bytes(),
    )];
    if let Some(file) = &harness.dist.catalog {
        written.push((staging.join(file), render::catalog(harness)?.into_bytes()));
    }
    for piece in &content.pieces {
        let file = match piece.kind {
            Kind::Agent => staging.join("agents").join(format!("{}.md", piece.name)),
            Kind::Skill | Kind::Command => {
                staging.join("skills").join(&piece.name).join("SKILL.md")
            }
        };
        written.push((file, render::document(piece, harness, &[])?.into_bytes()));
    }
    written.extend(render::supporting_files(&staging, content, harness)?);
    for part in parts {
        let file = staging.join("contract").join(&part.file);
        written.push((file, part.text.clone().into_bytes()));
    }
    if let Some(hooks) = hooks(parts, content.options.as_ref())? {
        written.push((staging.join("hooks/hooks.json"), hooks.into_bytes()));
    }
    written.extend(render::harness_files(root, &staging, harness)?);
    written.extend(render::binary_files(&staging, binaries)?);
    staging::write_files(&staging, &harness.name, &written)?;
    Ok((staging, written.len()))
}

fn manifest(root: &Path, harness: &Harness, options: Option<&OptionsFile>) -> io::Result<String> {
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
        #[serde(skip_serializing_if = "Vec::is_empty")]
        dependencies: Vec<String>,
        #[serde(
            rename = "userConfig",
            skip_serializing_if = "serde_json::Map::is_empty"
        )]
        user_config: serde_json::Map<String, serde_json::Value>,
    }
    let version = render::workspace_version(root)?;
    let user_config = render::option_settings(options, harness)?;
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
        dependencies: harness
            .companions
            .iter()
            .map(|companion| {
                companion
                    .marketplace(&harness.name)
                    .map(|marketplace| format!("{}@{}", companion.plugin, marketplace))
            })
            .collect::<io::Result<_>>()?,
        user_config,
    };
    Ok(serde_json::to_string_pretty(&manifest).map_err(io::Error::other)? + "\n")
}

fn hooks(parts: &[Part], options: Option<&OptionsFile>) -> io::Result<Option<String>> {
    let declares =
        |name| options.is_some_and(|file| file.declared.iter().any(|option| option.name == name));
    let commands: Vec<serde_json::Value> = parts
        .iter()
        .map(|part| {
            let contract_option = declares("contract").then_some("contract");
            let tests: String = contract_option
                .into_iter()
                .chain(part.gate.as_deref())
                .map(|option| {
                    format!(
                        "[ \"$CLAUDE_PLUGIN_OPTION_{}\" = false ] || ",
                        option.to_uppercase()
                    )
                })
                .collect();
            serde_json::json!({
                "type": "command",
                "command": format!("{tests}cat \"${{CLAUDE_PLUGIN_ROOT}}/contract/{}\"", part.file),
            })
        })
        .collect();
    let mut session_start = Vec::new();
    if !commands.is_empty() {
        session_start.push(serde_json::json!({
            "matcher": "startup|clear|compact",
            "hooks": commands,
        }));
    }
    let memory_hook = |variables: &str, subcommand: &str| {
        serde_json::json!([{
            "type": "command",
            "command": format!(
                "[ \"$CLAUDE_PLUGIN_OPTION_MEMORY\" = false ] || TIGHTSHIP_VAULT=\"$CLAUDE_PLUGIN_OPTION_VAULT_FOLDER\" {variables}sh \"${{CLAUDE_PLUGIN_ROOT}}/bin/tightship\" hook {subcommand}"
            ),
        }])
    };
    if declares("memory") {
        let env_file = "TIGHTSHIP_ENV_FILE=\"$CLAUDE_ENV_FILE\" ";
        session_start.extend([
            serde_json::json!({
                "matcher": "startup|clear|compact",
                "hooks": memory_hook(env_file, "start"),
            }),
            serde_json::json!({
                "matcher": "resume|fork",
                "hooks": memory_hook(env_file, "export"),
            }),
            serde_json::json!({
                "matcher": "compact",
                "hooks": memory_hook("", "checkpoint"),
            }),
        ]);
    }
    if declares("status_line") {
        session_start.push(serde_json::json!({
            "matcher": "startup",
            "hooks": [{
                "type": "command",
                "command": "TIGHTSHIP_STATUS_LINE=\"$CLAUDE_PLUGIN_OPTION_STATUS_LINE\" TIGHTSHIP_SETTINGS=\"${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json\" TIGHTSHIP_DATA=\"$CLAUDE_PLUGIN_DATA\" TIGHTSHIP_ROOT=\"$CLAUDE_PLUGIN_ROOT\" sh \"${CLAUDE_PLUGIN_ROOT}/bin/tightship\" hook status-line",
            }],
        }));
    }
    if declares("auto_update") {
        session_start.push(serde_json::json!({
            "matcher": "startup",
            "hooks": [{
                "type": "command",
                "command": "TIGHTSHIP_AUTO_UPDATE=\"$CLAUDE_PLUGIN_OPTION_AUTO_UPDATE\" TIGHTSHIP_SETTINGS=\"${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json\" TIGHTSHIP_MANIFEST=\"${CLAUDE_PLUGIN_ROOT}/.claude-plugin/plugin.json\" sh \"${CLAUDE_PLUGIN_ROOT}/bin/tightship\" hook marketplace-auto-update",
            }],
        }));
    }
    if session_start.is_empty() {
        return Ok(None);
    }
    let mut events = serde_json::Map::new();
    events.insert("SessionStart".to_owned(), session_start.into());
    let hooks = serde_json::json!({ "hooks": events });
    Ok(Some(
        serde_json::to_string_pretty(&hooks).map_err(io::Error::other)? + "\n",
    ))
}
