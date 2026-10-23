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
        written.push((staging.join(file), render::catalog(harness)?.into_bytes()));
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
    if let Some(hooks) = hooks(parts, content.options.as_ref())? {
        written.push((staging.join("hooks/hooks.json"), hooks.into_bytes()));
    }
    written.extend(render::harness_files(root, &staging, harness)?);
    written.extend(render::binary_files(&staging, binaries)?);
    staging::write_files(&staging, &harness.name, &written)?;
    Ok((staging, written.len()))
}

fn hooks(parts: &[Part], options: Option<&OptionsFile>) -> io::Result<Option<String>> {
    let declares =
        |name| options.is_some_and(|file| file.declared.iter().any(|option| option.name == name));
    let binary = "${CURSOR_PLUGIN_ROOT}/bin/tightship";
    let mut session_start: Vec<serde_json::Value> = parts
        .iter()
        .map(|part| json!({ "command": format!("{binary} hook contract-part {}", part.file) }))
        .collect();
    let before_submit = session_start.clone();
    if declares("status_line") {
        session_start.push(json!({ "command": format!("{binary} hook status-line") }));
    }
    let memory = declares("memory");
    if memory {
        session_start.push(json!({ "command": format!("{binary} hook start") }));
    }
    if session_start.is_empty() {
        return Ok(None);
    }
    let mut events = serde_json::Map::new();
    events.insert("sessionStart".to_owned(), session_start.into());
    if !before_submit.is_empty() {
        events.insert("beforeSubmitPrompt".to_owned(), before_submit.into());
    }
    if memory {
        events.insert(
            "preCompact".to_owned(),
            json!([{ "command": format!("{binary} hook checkpoint") }]),
        );
    }
    let body = json!({ "version": 1, "hooks": events });
    Ok(Some(
        serde_json::to_string_pretty(&body).map_err(io::Error::other)? + "\n",
    ))
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
    let variables = render::option_settings(options, harness)?;
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
    let read_only = piece
        .front_matter
        .as_ref()
        .and_then(|front_matter| front_matter.access.as_deref())
        == Some("read-only");
    let agent_fields: &[(&str, serde_json::Value)] = if read_only {
        &[("readonly", serde_json::Value::Bool(true))]
    } else {
        &[]
    };
    render::document(piece, harness, agent_fields)
}
