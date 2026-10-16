use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::binary::Binary;
use super::content::{Content, Kind, Piece};
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
    let mut written = vec![
        (
            staging.join(".claude-plugin/plugin.json"),
            manifest(root, harness, content.options.as_ref())?.into_bytes(),
        ),
        (
            staging.join(".claude-plugin/marketplace.json"),
            catalog(harness)?.into_bytes(),
        ),
    ];
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
    if let Some(hooks) = hooks(parts, content.options.as_ref())? {
        written.push((staging.join("hooks/hooks.json"), hooks.into_bytes()));
    }
    let launcher = staging.join("bin/tightship");
    written.push((
        launcher.clone(),
        include_bytes!("../../../tightship/launcher.sh").to_vec(),
    ));
    let mut executables = vec![launcher];
    let binary_copies = render::binary_files(&staging, binaries)?;
    executables.extend(binary_copies.iter().map(|(copy, _)| copy.clone()));
    written.extend(binary_copies);
    staging::write_files(&written, &executables)?;
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
    let mut user_config = serde_json::Map::new();
    if let Some(file) = options {
        for option in &file.declared {
            let mut setting = serde_json::json!({
                "type": option.kind,
                "title": render::resolve(&option.title, &file.path, harness)?,
                "description": render::resolve(&option.description, &file.path, harness)?,
            });
            if !option.is_directory() {
                setting["default"] = serde_json::Value::Bool(true);
            }
            user_config.insert(option.name.clone(), setting);
        }
    }
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
            .map(|companion| format!("{}@{}", companion.plugin, companion.marketplace))
            .collect(),
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
    let memory = declares("memory");
    if memory {
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
    if session_start.is_empty() {
        return Ok(None);
    }
    let mut events = serde_json::Map::new();
    events.insert("SessionStart".to_owned(), session_start.into());
    if memory {
        events.insert(
            "UserPromptSubmit".to_owned(),
            serde_json::json!([{
                "hooks": memory_hook("TIGHTSHIP_DATA=\"$CLAUDE_PLUGIN_DATA\" ", "prompt"),
            }]),
        );
    }
    let hooks = serde_json::json!({ "hooks": events });
    Ok(Some(
        serde_json::to_string_pretty(&hooks).map_err(io::Error::other)? + "\n",
    ))
}

fn catalog(harness: &Harness) -> io::Result<String> {
    #[derive(Serialize)]
    struct Owner<'harness> {
        name: &'harness str,
    }
    #[derive(Serialize)]
    struct Entry<'harness> {
        name: &'harness str,
        source: &'static str,
        description: &'harness str,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Catalog<'harness> {
        name: &'harness str,
        owner: Owner<'harness>,
        description: &'harness str,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        allow_cross_marketplace_dependencies_on: Vec<&'harness str>,
        plugins: [Entry<'harness>; 1],
    }
    let plugin = &harness.plugin;
    let mut marketplaces = Vec::new();
    for companion in &harness.companions {
        if !marketplaces.contains(&companion.marketplace.as_str()) {
            marketplaces.push(companion.marketplace.as_str());
        }
    }
    let catalog = Catalog {
        name: &plugin.name,
        owner: Owner {
            name: &plugin.author,
        },
        description: &plugin.description,
        allow_cross_marketplace_dependencies_on: marketplaces,
        plugins: [Entry {
            name: &plugin.name,
            source: "./",
            description: &plugin.description,
        }],
    };
    Ok(serde_json::to_string_pretty(&catalog).map_err(io::Error::other)? + "\n")
}

fn document(piece: &Piece, harness: &Harness) -> io::Result<String> {
    let front_matter = piece.front_matter.as_ref().ok_or_else(|| {
        io::Error::other(format!(
            "{}: rendered with invalid front matter",
            piece.path
        ))
    })?;
    let mut fields = vec![
        ("name", piece.name.as_str()),
        ("description", front_matter.description.as_str()),
    ];
    if let Some(hint) = &front_matter.argument_hint {
        fields.push(("argument-hint", hint));
    }
    if piece.kind == Kind::Agent {
        fields.push(("tools", &harness.access.read_only));
    }
    let mut document = String::from("---\n");
    for (key, value) in fields {
        document.push_str(&format!("{key}: {}\n", serde_json::Value::from(value)));
    }
    document.push_str("---\n");
    document.push_str(&render::resolve(&piece.body, &piece.path, harness)?);
    Ok(document)
}
