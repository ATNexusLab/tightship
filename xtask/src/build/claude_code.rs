use std::io;
use std::path::{Path, PathBuf};

use super::binary::Binary;
use super::content::Content;
use super::contract::Part;
use super::harness::Harness;
use super::options::{self, OptionsFile};
use super::render;

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

fn hooks(parts: &[Part], options: Option<&OptionsFile>, clean: bool) -> io::Result<Option<String>> {
    let declares = |name| options::declared(options, name);
    let mut commands: Vec<serde_json::Value> = parts
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
    if !parts.is_empty() && declares("contract") {
        commands.push(serde_json::json!({
            "type": "command",
            "command": "[ \"$CLAUDE_PLUGIN_OPTION_CONTRACT\" = false ] || sh \"${CLAUDE_PLUGIN_ROOT}/bin/tightship\" hook map",
        }));
    }
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
    let settings = "TIGHTSHIP_SETTINGS=\"${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json\" TIGHTSHIP_DATA=\"$CLAUDE_PLUGIN_DATA\" TIGHTSHIP_ROOT=\"$CLAUDE_PLUGIN_ROOT\"";
    let binary = "sh \"${CLAUDE_PLUGIN_ROOT}/bin/tightship\"";
    let startup: Vec<String> = [
        clean.then(|| format!("{settings} {binary} hook clean")),
        declares("status_line").then(|| {
            format!("TIGHTSHIP_STATUS_LINE=\"$CLAUDE_PLUGIN_OPTION_STATUS_LINE\" {settings} {binary} hook status-line")
        }),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !startup.is_empty() {
        session_start.push(serde_json::json!({
            "matcher": "startup",
            "hooks": [{ "type": "command", "command": startup.join("; ") }],
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
    render::pretty_json(&hooks).map(Some)
}
