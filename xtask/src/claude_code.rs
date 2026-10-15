use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::content::{Content, Kind, Piece, SupportingBody};
use crate::diagnostics::naming;
use crate::harness::Harness;
use crate::tokens::{self, Form};

pub fn render(root: &Path, harness: &Harness, content: &Content) -> io::Result<(PathBuf, usize)> {
    let staging = root.join("dist").join(format!(".{}.tmp", harness.name));
    remove_if_present(&staging)?;
    let mut written = vec![(
        staging.join(".claude-plugin/plugin.json"),
        manifest(root, harness)?.into_bytes(),
    )];
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

fn manifest(root: &Path, harness: &Harness) -> io::Result<String> {
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
    }
    let cargo_path = root.join("Cargo.toml");
    let cargo_text = fs::read_to_string(&cargo_path).map_err(naming(&cargo_path))?;
    let cargo: CargoManifest = toml::from_str(&cargo_text)
        .map_err(|error| io::Error::other(format!("Cargo.toml: {}", error.message())))?;
    let plugin = &harness.plugin;
    let manifest = PluginManifest {
        name: &plugin.name,
        version: cargo.workspace.package.version,
        description: &plugin.description,
        author: Author {
            name: &plugin.author,
        },
        repository: &plugin.repository,
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
    let mut unresolved = Vec::new();
    let resolved = tokens::replace(text, |token| {
        let replacement = match token.form {
            Form::Piece(kind, name) => Some(
                harness
                    .references
                    .template(kind)
                    .replace("{plugin}", &harness.plugin.name)
                    .replace("{name}", name),
            ),
            Form::Capability(name) => harness.capabilities.get(name).cloned(),
            Form::UnknownKind(_) | Form::Malformed => None,
        };
        replacement.unwrap_or_else(|| {
            unresolved.push(token.text.to_owned());
            token.text.to_owned()
        })
    });
    if unresolved.is_empty() {
        Ok(resolved)
    } else {
        Err(io::Error::other(format!(
            "{path}: rendered with unresolved tokens {}",
            unresolved.join(", ")
        )))
    }
}
