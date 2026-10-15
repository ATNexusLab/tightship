use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::content::{Content, Kind, Piece, SupportingBody};
use crate::contract::Part;
use crate::diagnostics::naming;
use crate::harness::Harness;
use crate::options::OptionsFile;
use crate::tokens;

pub fn render(
    root: &Path,
    harness: &Harness,
    content: &Content,
    parts: &[Part],
) -> io::Result<(PathBuf, usize)> {
    let staging = root.join("dist").join(format!(".{}.tmp", harness.name));
    remove_if_present(&staging)?;
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
        written.push((file, part.text.clone().into_bytes()));
    }
    if !parts.is_empty() {
        written.push((
            staging.join("hooks/hooks.json"),
            hooks(parts, content.options.as_ref())?.into_bytes(),
        ));
    }
    for (file, bytes) in &written {
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder).map_err(naming(folder))?;
        }
        fs::write(file, bytes).map_err(naming(file))?;
    }
    Ok((staging, written.len()))
}

pub fn publish(root: &Path, staging: &Path, harness_name: &str) -> io::Result<()> {
    let target = root.join("dist").join(harness_name);
    remove_if_present(&target)?;
    fs::rename(staging, &target).map_err(naming(&target))
}

pub fn discard(root: &Path, harness_name: &str) -> io::Result<()> {
    let dist = root.join("dist");
    remove_if_present(&dist.join(format!(".{harness_name}.tmp")))?;
    remove_if_present(&dist.join(harness_name))
}

fn remove_if_present(folder: &Path) -> io::Result<()> {
    match fs::remove_dir_all(folder) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(naming(folder)(error)),
        _ => Ok(()),
    }
}

fn manifest(root: &Path, harness: &Harness, options: Option<&OptionsFile>) -> io::Result<String> {
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
        #[serde(skip_serializing_if = "Vec::is_empty")]
        dependencies: Vec<String>,
        #[serde(
            rename = "userConfig",
            skip_serializing_if = "serde_json::Map::is_empty"
        )]
        user_config: serde_json::Map<String, serde_json::Value>,
    }
    let cargo_path = root.join("Cargo.toml");
    let cargo_text = fs::read_to_string(&cargo_path).map_err(naming(&cargo_path))?;
    let cargo: CargoManifest = toml::from_str(&cargo_text)
        .map_err(|error| io::Error::other(format!("Cargo.toml: {}", error.message())))?;
    let mut user_config = serde_json::Map::new();
    if let Some(file) = options {
        for option in &file.declared {
            user_config.insert(
                option.name.clone(),
                serde_json::json!({
                    "type": "boolean",
                    "title": resolve(&option.title, &file.path, harness)?,
                    "description": resolve(&option.description, &file.path, harness)?,
                    "default": true,
                }),
            );
        }
    }
    let plugin = &harness.plugin;
    let manifest = PluginManifest {
        name: &plugin.name,
        version: cargo.workspace.package.version,
        description: &plugin.description,
        author: Author {
            name: &plugin.author,
        },
        repository: &plugin.repository,
        dependencies: harness
            .companions
            .iter()
            .map(|companion| format!("{}@{}", companion.plugin, companion.marketplace))
            .collect(),
        user_config,
    };
    Ok(serde_json::to_string_pretty(&manifest).map_err(io::Error::other)? + "\n")
}

fn hooks(parts: &[Part], options: Option<&OptionsFile>) -> io::Result<String> {
    let declares_contract =
        options.is_some_and(|file| file.declared.iter().any(|option| option.name == "contract"));
    let commands: Vec<serde_json::Value> = parts
        .iter()
        .map(|part| {
            let contract_option = declares_contract.then_some("contract");
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
    let hooks = serde_json::json!({
        "hooks": {
            "SessionStart": [{
                "matcher": "startup|clear|compact",
                "hooks": commands,
            }],
        },
    });
    Ok(serde_json::to_string_pretty(&hooks).map_err(io::Error::other)? + "\n")
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
