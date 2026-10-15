use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::contract::{self, Contract};
use super::options::{self, OptionsFile};
use crate::diagnostics::{Diagnostic, naming};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Skill,
    Command,
    Agent,
}

impl Kind {
    pub fn noun(self) -> &'static str {
        match self {
            Self::Skill => "skill",
            Self::Command => "command",
            Self::Agent => "agent",
        }
    }
}

#[derive(Deserialize)]
pub struct FrontMatter {
    pub description: String,
    #[serde(rename = "argument-hint")]
    pub argument_hint: Option<String>,
}

pub struct Piece {
    pub kind: Kind,
    pub name: String,
    pub path: String,
    pub front_matter: Option<FrontMatter>,
    pub body: String,
    pub body_line: usize,
}

pub enum SupportingBody {
    Markdown(String),
    Copied(Vec<u8>),
}

pub struct Supporting {
    pub skill: String,
    pub relative: String,
    pub path: String,
    pub body: SupportingBody,
}

pub struct SourceFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct Content {
    pub files: Vec<SourceFile>,
    pub pieces: Vec<Piece>,
    pub supporting: Vec<Supporting>,
    pub options: Option<OptionsFile>,
    pub contract: Option<Contract>,
    pub vault_template: Option<String>,
}

pub fn read(root: &Path, diagnostics: &mut Vec<Diagnostic>) -> io::Result<Content> {
    let mut content = Content::default();
    let directory = root.join("content");
    if !directory.is_dir() {
        diagnostics.push(Diagnostic::general("content/ is missing".to_owned()));
        return Ok(content);
    }
    for file in files_under(&directory)? {
        let bytes = fs::read(&file).map_err(naming(&file))?;
        let relative: Vec<String> = file
            .strip_prefix(root)
            .map_err(io::Error::other)?
            .components()
            .map(|component| component.as_os_str().to_string_lossy().into_owned())
            .collect();
        let path = relative.join("/");
        classify(&mut content, &relative[1..], &path, &bytes, diagnostics);
        content.files.push(SourceFile { path, bytes });
    }
    let contract_file = content
        .files
        .iter()
        .find(|file| file.path == "content/contract.md");
    if let Some(file) = contract_file
        && let Some(text) = utf8(&file.path, &file.bytes, diagnostics)
    {
        let contract = contract::read(&file.path, text, content.options.as_ref(), diagnostics);
        content.contract = Some(contract);
    }
    for supporting in &content.supporting {
        let has_skill = content
            .pieces
            .iter()
            .any(|piece| piece.kind == Kind::Skill && piece.name == supporting.skill);
        if !has_skill {
            diagnostics.push(Diagnostic::in_file(
                &supporting.path,
                format!("content/skills/{}/ has no SKILL.md", supporting.skill),
            ));
        }
    }
    check_names_are_unique(&content.pieces, diagnostics);
    Ok(content)
}

pub fn is_valid_name(text: &str) -> bool {
    text.len() <= 64
        && text.starts_with(|character: char| character.is_ascii_lowercase())
        && !text.ends_with('-')
        && !text.contains("--")
        && text.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

fn files_under(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(directory)
        .map_err(naming(directory))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<PathBuf>>>()?;
    entries.sort();
    let mut files = Vec::new();
    for entry in entries {
        let file_type = fs::symlink_metadata(&entry)
            .map_err(naming(&entry))?
            .file_type();
        if file_type.is_symlink() {
            return Err(io::Error::other(format!(
                "{} is a symbolic link; content/ holds only real files",
                entry.display()
            )));
        }
        if file_type.is_dir() {
            files.extend(files_under(&entry)?);
        } else {
            files.push(entry);
        }
    }
    Ok(files)
}

fn classify(
    content: &mut Content,
    segments: &[String],
    path: &str,
    bytes: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let segments: Vec<&str> = segments.iter().map(String::as_str).collect();
    let piece = match segments.as_slice() {
        ["options.toml"] => {
            if let Some(text) = utf8(path, bytes, diagnostics) {
                content.options = Some(options::read(path, text, diagnostics));
            }
            None
        }
        ["contract.md"] => None,
        ["vault-template", "AGENTS.md"] => {
            content.vault_template = utf8(path, bytes, diagnostics);
            None
        }
        ["skills", skill, "SKILL.md"] => Some((Kind::Skill, (*skill).to_owned())),
        ["commands", file] if file.ends_with(".md") => {
            Some((Kind::Command, file.trim_end_matches(".md").to_owned()))
        }
        ["agents", file] if file.ends_with(".md") => {
            Some((Kind::Agent, file.trim_end_matches(".md").to_owned()))
        }
        ["skills", skill, rest @ ..] if !rest.is_empty() => {
            let relative = rest.join("/");
            let body = if relative.ends_with(".md") {
                match utf8(path, bytes, diagnostics) {
                    Some(text) => SupportingBody::Markdown(text),
                    None => return,
                }
            } else {
                SupportingBody::Copied(bytes.to_vec())
            };
            content.supporting.push(Supporting {
                skill: (*skill).to_owned(),
                relative,
                path: path.to_owned(),
                body,
            });
            None
        }
        _ => {
            diagnostics.push(Diagnostic::in_file(
                path,
                "not a piece; content/ holds skills/<name>/, commands/<name>.md, agents/<name>.md, options.toml, contract.md and vault-template/AGENTS.md"
                    .to_owned(),
            ));
            None
        }
    };
    let Some((kind, name)) = piece else {
        return;
    };
    let Some(text) = utf8(path, bytes, diagnostics) else {
        return;
    };
    content
        .pieces
        .push(parse_piece(kind, name, path, &text, diagnostics));
}

fn utf8(path: &str, bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => Some(text),
        Err(_) => {
            diagnostics.push(Diagnostic::in_file(path, "is not UTF-8".to_owned()));
            None
        }
    }
}

fn parse_piece(
    kind: Kind,
    name: String,
    path: &str,
    text: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Piece {
    let Some((front_matter, body, body_line)) = split_front_matter(text) else {
        diagnostics.push(Diagnostic::in_file(
            path,
            "front matter: no +++ block".to_owned(),
        ));
        return Piece {
            kind,
            name,
            path: path.to_owned(),
            front_matter: None,
            body: text.to_owned(),
            body_line: 1,
        };
    };
    Piece {
        kind,
        front_matter: validate(kind, &name, path, front_matter, diagnostics),
        name,
        path: path.to_owned(),
        body: body.to_owned(),
        body_line,
    }
}

pub fn split_front_matter(text: &str) -> Option<(&str, &str, usize)> {
    let after_opening = text.strip_prefix("+++\n")?;
    let mut offset = 0;
    for (index, line) in after_opening.split_inclusive('\n').enumerate() {
        if line.trim_end_matches('\n') == "+++" {
            let body = &after_opening[offset + line.len()..];
            return Some((&after_opening[..offset], body, index + 3));
        }
        offset += line.len();
    }
    None
}

fn validate(
    kind: Kind,
    file_name: &str,
    path: &str,
    front_matter: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<FrontMatter> {
    let table: toml::Table = match front_matter.parse() {
        Ok(table) => table,
        Err(error) => {
            diagnostics.push(Diagnostic::in_file(
                path,
                format!("front matter: {}", error.message()),
            ));
            return None;
        }
    };
    let (allowed, required): (&[&str], &[&str]) = match kind {
        Kind::Skill => (&["name", "description"], &["name", "description"]),
        Kind::Command => (
            &["name", "description", "argument-hint"],
            &["name", "description"],
        ),
        Kind::Agent => (
            &["name", "description", "access"],
            &["name", "description", "access"],
        ),
    };
    let found_before = diagnostics.len();
    let mut reject = |reason: String| {
        diagnostics.push(Diagnostic::in_file(path, format!("front matter: {reason}")));
    };
    for (key, value) in &table {
        if !allowed.contains(&key.as_str()) {
            reject(format!("unknown key \"{key}\""));
        } else if !is_in_domain(key, value) {
            reject(format!("\"{key}\" is outside its domain"));
        }
    }
    for key in required {
        if !table.contains_key(*key) {
            reject(format!("missing key \"{key}\""));
        }
    }
    let declared_name = table.get("name").and_then(toml::Value::as_str);
    if let Some(declared_name) = declared_name
        && is_valid_name(declared_name)
        && declared_name != file_name
    {
        diagnostics.push(Diagnostic::in_file(
            path,
            format!("name \"{declared_name}\" differs from its file name \"{file_name}\""),
        ));
    }
    if diagnostics.len() > found_before {
        return None;
    }
    match table.try_into() {
        Ok(front_matter) => Some(front_matter),
        Err(error) => {
            diagnostics.push(Diagnostic::in_file(
                path,
                format!("front matter: {}", error.message()),
            ));
            None
        }
    }
}

fn is_in_domain(key: &str, value: &toml::Value) -> bool {
    let fits_one_line = |text: &str, maximum: usize| {
        !text.is_empty() && text.chars().count() <= maximum && !text.contains(['\n', '\r'])
    };
    let Some(text) = value.as_str() else {
        return false;
    };
    match key {
        "name" => is_valid_name(text),
        "description" => fits_one_line(text, 1024),
        "argument-hint" => fits_one_line(text, 200),
        "access" => text == "read-only",
        _ => false,
    }
}

fn check_names_are_unique(pieces: &[Piece], diagnostics: &mut Vec<Diagnostic>) {
    for (index, piece) in pieces.iter().enumerate() {
        let owner = pieces[..index]
            .iter()
            .find(|other| other.name == piece.name);
        if let Some(owner) = owner {
            diagnostics.push(Diagnostic::in_file(
                &piece.path,
                format!(
                    "the name \"{}\" already belongs to {}",
                    piece.name, owner.path
                ),
            ));
        }
    }
}
