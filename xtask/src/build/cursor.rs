//! What the Cursor plugin adds to the shared layout: commands as files, its hooks, and read-only agents.

use std::io;
use std::path::{Path, PathBuf};

use serde_json::json;

use super::binary::Binary;
use super::content::{Content, Piece};
use super::contract::Part;
use super::harness::Harness;
use super::options::{self, OptionsFile};
use super::render;

/// Renders the Cursor plugin into its staging folder and returns it with the number of files.
///
/// # Errors
///
/// When a piece does not render or a file cannot be written.
pub fn render(
    root: &Path,
    harness: &Harness,
    content: &Content,
    parts: &[Part],
    binaries: &[Binary],
) -> io::Result<(PathBuf, usize)> {
    let options = content.options.as_ref();
    let mut extra = Vec::new();
    if let Some(hooks) = hooks(parts, options, harness.clean.is_some())? {
        extra.push(("hooks/hooks.json".to_owned(), hooks));
    }
    let layout = render::Layout {
        manifest: render::manifest(root, harness, Some("variables"), options)?,
        commands_as_files: true,
        document,
        extra,
    };
    render::assemble(root, harness, content, parts, binaries, layout)
}

fn hooks(parts: &[Part], options: Option<&OptionsFile>, clean: bool) -> io::Result<Option<String>> {
    let declares = |name| options::declared(options, name);
    let binary = "${CURSOR_PLUGIN_ROOT}/bin/tightship";
    let mut session_start: Vec<serde_json::Value> = parts
        .iter()
        .map(|part| json!({ "command": format!("{binary} hook contract-part {}", part.file) }))
        .collect();
    let before_submit = session_start.clone();
    if !parts.is_empty() && declares("contract") {
        session_start.push(json!({ "command": format!("{binary} hook map") }));
    }
    if clean {
        session_start.push(json!({ "command": format!("{binary} hook clean") }));
    }
    if declares("status_line") {
        session_start.push(json!({ "command": format!("{binary} hook status-line") }));
    }
    let memory = declares("memory");
    if memory {
        session_start.push(json!({ "command": format!("{binary} hook start") }));
    }
    if declares("auto_update") {
        session_start.push(json!({ "command": format!("{binary} hook update") }));
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
    render::pretty_json(&body).map(Some)
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
