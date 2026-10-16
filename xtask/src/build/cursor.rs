use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

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
    let mut written = vec![(
        staging.join(&harness.dist.manifest),
        manifest(root, harness, content.options.as_ref())?.into_bytes(),
    )];
    if let Some(file) = &harness.dist.catalog {
        written.push((staging.join(file), catalog(harness)?.into_bytes()));
    }
    for piece in &content.pieces {
        let file = match piece.kind {
            Kind::Skill => staging.join("skills").join(&piece.name).join("SKILL.md"),
            Kind::Command => staging.join("commands").join(format!("{}.md", piece.name)),
            Kind::Agent => staging.join("agents").join(format!("{}.md", piece.name)),
        };
        written.push((file, document(piece, harness)?.into_bytes()));
    }
    written.extend(render::supporting_files(&staging, content, harness)?);
    for part in parts {
        let file = staging.join("contract").join(&part.file);
        written.push((file, part.text.clone().into_bytes()));
    }
    if let Some(hooks) = hooks(parts)? {
        written.push((staging.join("hooks/hooks.json"), hooks.into_bytes()));
    }
    written.extend(render::harness_files(root, &staging, harness)?);
    written.extend(render::binary_files(&staging, binaries)?);
    staging::write_files(&staging, &written)?;
    Ok((staging, written.len()))
}

fn hooks(parts: &[Part]) -> io::Result<Option<String>> {
    if parts.is_empty() {
        return Ok(None);
    }
    let commands: Vec<serde_json::Value> = parts
        .iter()
        .map(|part| {
            json!({
                "command": format!(
                    "${{CURSOR_PLUGIN_ROOT}}/bin/tightship hook contract-part {}",
                    part.file
                ),
            })
        })
        .collect();
    let body = json!({
        "version": 1,
        "hooks": {
            "sessionStart": commands.clone(),
            "beforeSubmitPrompt": commands,
        },
    });
    Ok(Some(
        serde_json::to_string_pretty(&body).map_err(io::Error::other)? + "\n",
    ))
}

fn catalog(harness: &Harness) -> io::Result<String> {
    #[derive(Serialize)]
    struct CatalogEntry<'harness> {
        name: &'harness str,
        #[serde(rename = "pluginRoot")]
        plugin_root: String,
        #[serde(rename = "marketplace")]
        marketplace: String,
    }
    #[derive(Serialize)]
    struct Catalog<'harness> {
        name: &'harness str,
        owner: Author<'harness>,
        plugins: Vec<CatalogEntry<'harness>>,
    }
    #[derive(Serialize)]
    struct Author<'harness> {
        name: &'harness str,
    }
    let plugin = &harness.plugin;
    let catalog = Catalog {
        name: "tightship",
        owner: Author {
            name: &plugin.author,
        },
        plugins: vec![CatalogEntry {
            name: &plugin.name,
            plugin_root: "./".to_owned(),
            marketplace: plugin.repository.clone(),
        }],
    };
    Ok(serde_json::to_string_pretty(&catalog).map_err(io::Error::other)? + "\n")
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
        #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
        variables: serde_json::Map<String, serde_json::Value>,
    }
    let version = render::workspace_version(root)?;
    let mut variables = serde_json::Map::new();
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
            variables.insert(option.name.clone(), setting);
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
        variables,
    };
    Ok(serde_json::to_string_pretty(&manifest).map_err(io::Error::other)? + "\n")
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
        if front_matter.access.as_deref() == Some("read-only") {
            fields.push(("readonly", "true"));
        }
    }
    let mut document = String::from("---\n");
    for (key, value) in fields {
        document.push_str(&format!("{key}: {}\n", serde_json::Value::from(value)));
    }
    document.push_str("---\n");
    document.push_str(&render::resolve(&piece.body, &piece.path, harness)?);
    Ok(document)
}
