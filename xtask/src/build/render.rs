use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::binary::Binary;
use super::content::{self, Content, Kind, Piece, SupportingBody};
use super::contract::Part;
use super::harness::Harness;
use super::options::OptionsFile;
use super::scripts::{self, Shell};
use super::staging;
use super::tokens;
use crate::diagnostics::naming;

pub struct Layout {
    pub manifest: String,
    pub commands_as_files: bool,
    pub document: fn(&Piece, &Harness) -> io::Result<String>,
    pub extra: Vec<(String, String)>,
}

pub fn assemble(
    root: &Path,
    harness: &Harness,
    content: &Content,
    parts: &[Part],
    binaries: &[Binary],
    layout: Layout,
) -> io::Result<(PathBuf, usize)> {
    let staging = staging::staging_dir(root, &harness.name);
    staging::remove_if_present(&staging)?;
    let mut written = vec![(
        staging.join(&harness.dist.manifest),
        layout.manifest.into_bytes(),
    )];
    if let Some(file) = &harness.dist.catalog {
        written.push((staging.join(file), catalog(harness)?.into_bytes()));
    }
    for piece in &content.pieces {
        let file = match piece.kind {
            Kind::Agent => staging.join("agents").join(format!("{}.md", piece.name)),
            Kind::Command if layout.commands_as_files => {
                staging.join("commands").join(format!("{}.md", piece.name))
            }
            Kind::Skill | Kind::Command => {
                staging.join("skills").join(&piece.name).join("SKILL.md")
            }
        };
        written.push((file, (layout.document)(piece, harness)?.into_bytes()));
    }
    written.extend(supporting_files(&staging, content, harness)?);
    for part in parts {
        let file = staging.join("contract").join(&part.file);
        written.push((file, part.text.clone().into_bytes()));
    }
    for (file, text) in layout.extra {
        written.push((staging.join(file), text.into_bytes()));
    }
    written.extend(harness_files(root, &staging, harness)?);
    written.extend(binary_files(&staging, binaries)?);
    staging::write_files(&staging, &harness.name, &written)?;
    Ok((staging, written.len()))
}

pub fn pretty_json(value: &impl Serialize) -> io::Result<String> {
    Ok(serde_json::to_string_pretty(value).map_err(io::Error::other)? + "\n")
}

pub fn manifest(
    root: &Path,
    harness: &Harness,
    options_key: Option<&str>,
    options: Option<&OptionsFile>,
) -> io::Result<String> {
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
        #[serde(flatten)]
        options: serde_json::Map<String, serde_json::Value>,
    }
    let mut flattened = serde_json::Map::new();
    if let Some(key) = options_key {
        let settings = option_settings(options, harness)?;
        if !settings.is_empty() {
            flattened.insert(key.to_owned(), settings.into());
        }
    }
    let plugin = &harness.plugin;
    pretty_json(&PluginManifest {
        name: &plugin.name,
        version: workspace_version(root)?,
        description: &plugin.description,
        author: Author {
            name: &plugin.author,
        },
        repository: &plugin.repository,
        license: &plugin.license,
        options: flattened,
    })
}

pub fn resolve(text: &str, path: &str, harness: &Harness) -> io::Result<String> {
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

fn workspace_version(root: &Path) -> io::Result<String> {
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
    let cargo_path = root.join("Cargo.toml");
    let cargo_text = fs::read_to_string(&cargo_path).map_err(naming(&cargo_path))?;
    let cargo: CargoManifest = toml::from_str(&cargo_text)
        .map_err(|error| io::Error::other(format!("Cargo.toml: {}", error.message())))?;
    Ok(cargo.workspace.package.version)
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
        plugins: [Entry<'harness>; 1],
    }
    let plugin = &harness.plugin;
    let catalog = Catalog {
        name: &plugin.name,
        owner: Owner {
            name: &plugin.author,
        },
        description: &plugin.description,
        plugins: [Entry {
            name: &plugin.name,
            source: "./",
            description: &plugin.description,
        }],
    };
    pretty_json(&catalog)
}

fn option_settings(
    options: Option<&OptionsFile>,
    harness: &Harness,
) -> io::Result<serde_json::Map<String, serde_json::Value>> {
    let mut settings = serde_json::Map::new();
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
            settings.insert(option.name.clone(), setting);
        }
    }
    Ok(settings)
}

pub fn document(
    piece: &Piece,
    harness: &Harness,
    agent_fields: &[(&str, serde_json::Value)],
) -> io::Result<String> {
    let front_matter = piece.front_matter.as_ref().ok_or_else(|| {
        io::Error::other(format!(
            "{}: rendered with invalid front matter",
            piece.path
        ))
    })?;
    let mut fields = vec![
        ("name", serde_json::Value::from(piece.name.as_str())),
        (
            "description",
            serde_json::Value::from(front_matter.description.as_str()),
        ),
    ];
    if let Some(hint) = &front_matter.argument_hint {
        fields.push(("argument-hint", serde_json::Value::from(hint.as_str())));
    }
    if piece.kind == Kind::Agent {
        fields.push((
            "tools",
            serde_json::Value::from(harness.access.read_only.as_str()),
        ));
        fields.extend_from_slice(agent_fields);
    }
    let mut document = String::from("---\n");
    for (key, value) in fields {
        document.push_str(&format!("{key}: {value}\n"));
    }
    document.push_str("---\n");
    document.push_str(&resolve(&piece.body, &piece.path, harness)?);
    Ok(document)
}

fn supporting_files(
    staging: &Path,
    content: &Content,
    harness: &Harness,
) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
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
        files.push((file, bytes));
    }
    Ok(files)
}

fn harness_files(
    root: &Path,
    staging: &Path,
    harness: &Harness,
) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let folder = root.join("harnesses").join(&harness.name);
    let launcher = |neutral: &str, environment: &str, shell| -> io::Result<Option<String>> {
        let environment_file = folder.join(environment);
        match fs::read_to_string(&environment_file) {
            Ok(text) => {
                let source = format!("harnesses/{}/{environment}", harness.name);
                let filled = scripts::fill(&text, &source, harness, shell)?;
                let (header, body) = neutral.split_once('\n').unwrap_or_default();
                Ok(Some(format!("{header}\n{filled}{body}")))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(naming(&environment_file)(error)),
        }
    };
    let neutral_sh = include_str!("../../../tightship/launcher.sh");
    let sh = launcher(neutral_sh, "launcher-env.sh", Shell::Sh)?;
    let mut files = vec![(
        staging.join("bin/tightship"),
        sh.unwrap_or_else(|| neutral_sh.to_owned()).into_bytes(),
    )];
    let neutral_cmd = include_str!("../../../tightship/launcher.cmd");
    if let Some(cmd) = launcher(neutral_cmd, "launcher-env.cmd", Shell::Cmd)? {
        files.push((staging.join("bin/tightship.cmd"), cmd.into_bytes()));
    }
    if let Some(clean) = &harness.clean {
        let mut data = serde_json::to_value(clean).map_err(io::Error::other)?;
        if let Some(paths) = data
            .get_mut("paths")
            .and_then(serde_json::Value::as_array_mut)
        {
            paths.push(serde_json::json!({ "path": "{appdata}/ponytail/" }));
        }
        files.push((staging.join("clean.json"), pretty_json(&data)?.into_bytes()));
    }
    let plugin = folder.join("plugin");
    if plugin.is_dir() {
        for file in content::files_under(&plugin)? {
            let relative = file.strip_prefix(&plugin).map_err(io::Error::other)?;
            let bytes = fs::read(&file).map_err(naming(&file))?;
            files.push((staging.join(relative), bytes));
        }
    }
    Ok(files)
}

fn binary_files(staging: &Path, binaries: &[Binary]) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
    for binary in binaries {
        let file_name = binary.path.file_name().unwrap_or_default();
        let copy = staging.join("bin").join(binary.platform).join(file_name);
        let bytes = fs::read(&binary.path).map_err(naming(&binary.path))?;
        files.push((copy, bytes));
    }
    Ok(files)
}
