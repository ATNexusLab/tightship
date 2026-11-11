//! What the Claude Code plugin adds to the shared layout: its manifest options and its hooks.

use std::io;
use std::path::{Path, PathBuf};

use crate::build::check::content::Content;
use crate::build::check::options::{self, OptionsFile};
use crate::build::harness::Harness;
use crate::build::render;
use crate::build::render::binary::Binary;
use crate::build::render::contract::Part;

/// Renders the Claude Code plugin into its staging folder and returns it with the number of files.
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
        manifest: render::manifest(root, harness, Some("userConfig"), options)?,
        commands_as_files: false,
        document: |piece, harness| render::document(piece, harness, &[]),
        extra,
    };
    render::assemble(root, harness, content, parts, binaries, layout)
}

const LAUNCHER: &str = "sh \"${CLAUDE_PLUGIN_ROOT}/bin/tightship\"";

fn hooks(parts: &[Part], options: Option<&OptionsFile>, clean: bool) -> io::Result<Option<String>> {
    let declares = |name| options::declared(options, name);
    let mut session_start = Vec::new();
    session_start.extend(contract_hooks(parts, declares("contract")));
    if declares("memory") {
        session_start.extend(memory_hooks());
    }
    session_start.extend(startup_hook(clean, declares("status_line")));
    if declares("auto_update") {
        session_start.push(serde_json::json!({
            "matcher": "startup",
            "hooks": [{
                "type": "command",
                "command": format!("TIGHTSHIP_AUTO_UPDATE=\"$CLAUDE_PLUGIN_OPTION_AUTO_UPDATE\" TIGHTSHIP_SETTINGS=\"${{CLAUDE_CONFIG_DIR:-$HOME/.claude}}/settings.json\" TIGHTSHIP_MANIFEST=\"${{CLAUDE_PLUGIN_ROOT}}/.claude-plugin/plugin.json\" {LAUNCHER} hook marketplace-auto-update"),
            }],
        }));
    }
    if session_start.is_empty() {
        return Ok(None);
    }
    let mut events = serde_json::Map::new();
    events.insert("SessionStart".to_owned(), session_start.into());
    render::pretty_json(&serde_json::json!({ "hooks": events })).map(Some)
}

fn contract_hooks(parts: &[Part], has_contract_option: bool) -> Option<serde_json::Value> {
    let contract_option = has_contract_option.then_some("contract");
    let mut commands: Vec<serde_json::Value> = parts
        .iter()
        .map(|part| {
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
    if !parts.is_empty() && has_contract_option {
        commands.push(serde_json::json!({
            "type": "command",
            "command": format!("[ \"$CLAUDE_PLUGIN_OPTION_CONTRACT\" = false ] || {LAUNCHER} hook map"),
        }));
    }
    (!commands.is_empty()).then(|| {
        serde_json::json!({
            "matcher": "startup|clear|compact",
            "hooks": commands,
        })
    })
}

fn memory_hooks() -> [serde_json::Value; 3] {
    let hook = |matcher: &str, variables: &str, subcommand: &str| {
        serde_json::json!({
            "matcher": matcher,
            "hooks": [{
                "type": "command",
                "command": format!(
                    "[ \"$CLAUDE_PLUGIN_OPTION_MEMORY\" = false ] || TIGHTSHIP_VAULT=\"$CLAUDE_PLUGIN_OPTION_VAULT_FOLDER\" {variables}{LAUNCHER} hook {subcommand}"
                ),
            }],
        })
    };
    let env_file = "TIGHTSHIP_ENV_FILE=\"$CLAUDE_ENV_FILE\" ";
    [
        hook("startup|clear|compact", env_file, "start"),
        hook("resume|fork", env_file, "export"),
        hook("compact", "", "checkpoint"),
    ]
}

fn startup_hook(clean: bool, has_status_line_option: bool) -> Option<serde_json::Value> {
    let settings = "TIGHTSHIP_SETTINGS=\"${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json\" TIGHTSHIP_DATA=\"$CLAUDE_PLUGIN_DATA\" TIGHTSHIP_ROOT=\"$CLAUDE_PLUGIN_ROOT\"";
    let commands: Vec<String> = [
        clean.then(|| format!("{settings} {LAUNCHER} hook clean")),
        has_status_line_option.then(|| {
            format!("TIGHTSHIP_STATUS_LINE=\"$CLAUDE_PLUGIN_OPTION_STATUS_LINE\" {settings} {LAUNCHER} hook status-line")
        }),
    ]
    .into_iter()
    .flatten()
    .collect();
    (!commands.is_empty()).then(|| {
        serde_json::json!({
            "matcher": "startup",
            "hooks": [{ "type": "command", "command": commands.join("; ") }],
        })
    })
}
