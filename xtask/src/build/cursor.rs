use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::binary::Binary;
use super::claude_code::remove_if_present;
use super::content::{Content, Kind, Piece, SupportingBody};
use super::contract::Part;
use super::harness::Harness;
use super::options::OptionsFile;
use serde_json::json;
use super::tokens;
use crate::diagnostics::naming;

#[cfg(unix)]
use super::claude_code::make_executable;

pub fn render(
    root: &Path,
    harness: &Harness,
    content: &Content,
    parts: &[Part],
    binaries: &[Binary],
) -> io::Result<(PathBuf, usize)> {
    let staging = root.join("dist").join(format!(".{}.tmp", harness.name));
    remove_if_present(&staging)?;
    let mut written = vec![(
        staging.join(".cursor-plugin/plugin.json"),
        manifest(root, harness, content.options.as_ref())?.into_bytes(),
    )];
    for piece in &content.pieces {
        let file = match piece.kind {
            Kind::Skill => staging.join("skills").join(&piece.name).join("SKILL.md"),
            Kind::Command => staging.join("commands").join(format!("{}.md", piece.name)),
            Kind::Agent => staging.join("agents").join(format!("{}.md", piece.name)),
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
    if let Some(hooks) = hooks(parts)? {
        written.push((staging.join("hooks/hooks.json"), hooks.into_bytes()));
    }
    let launcher = staging.join("bin/tightship");
    written.push((
        launcher.clone(),
        include_bytes!("../../../tightship/launcher.sh").to_vec(),
    ));
    written.push((
        staging.join("bin/tightship.cmd"),
        include_bytes!("../../../tightship/launcher.cmd").to_vec(),
    ));
    let mut executables = vec![launcher, staging.join("bin/tightship.cmd")];
    for binary in binaries {
        let file_name = binary.path.file_name().unwrap_or_default();
        let copy = staging.join("bin").join(binary.platform).join(file_name);
        let bytes = fs::read(&binary.path).map_err(naming(&binary.path))?;
        written.push((copy.clone(), bytes));
        executables.push(copy);
    }
    for (file, bytes) in &written {
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder).map_err(naming(folder))?;
        }
        fs::write(file, bytes).map_err(naming(file))?;
    }
    #[cfg(unix)]
    for file in &executables {
        make_executable(file)?;
    }
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
        license: &'harness str,
        #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
        variables: serde_json::Map<String, serde_json::Value>,
    }
    let cargo_path = root.join("Cargo.toml");
    let cargo_text = fs::read_to_string(&cargo_path).map_err(naming(&cargo_path))?;
    let cargo: CargoManifest = toml::from_str(&cargo_text)
        .map_err(|error| io::Error::other(format!("Cargo.toml: {}", error.message())))?;
    let mut variables = serde_json::Map::new();
    if let Some(file) = options {
        for option in &file.declared {
            let mut setting = serde_json::json!({
                "type": option.kind,
                "title": resolve(&option.title, &file.path, harness)?,
                "description": resolve(&option.description, &file.path, harness)?,
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
        version: cargo.workspace.package.version,
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
