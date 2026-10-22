use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

use super::content::Kind;
use crate::diagnostics::{Diagnostic, naming};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Harness {
    #[serde(skip)]
    pub name: String,
    pub plugin: Plugin,
    pub dist: Dist,
    pub install: Install,
    #[serde(default)]
    pub paths: BTreeMap<String, String>,
    pub references: References,
    pub access: Access,
    pub contract: Option<ContractTable>,
    pub capabilities: BTreeMap<String, String>,
    pub neutrality: Neutrality,
    #[serde(default)]
    pub companions: Vec<Companion>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub author: String,
    pub repository: String,
    pub license: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dist {
    pub manifest: String,
    pub catalog: Option<String>,
    #[serde(rename = "try-it")]
    pub try_it: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Install {
    pub label: String,
    pub command: String,
    #[serde(rename = "min-version")]
    pub min_version: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(rename = "companion-steps", default)]
    pub companion_steps: CompanionSteps,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CompanionSteps {
    pub plugin: Option<Steps>,
    pub skill: Option<Steps>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Steps {
    pub sh: String,
    pub ps1: String,
}

impl Install {
    pub fn field(&self, name: &str) -> Option<&str> {
        match name {
            "label" => Some(&self.label),
            "command" => Some(&self.command),
            "min-version" => Some(&self.min_version),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractTable {
    pub limit: usize,
    #[serde(rename = "compaction-notice")]
    pub compaction_notice: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Companion {
    pub plugin: String,
    pub marketplace: Option<String>,
    pub repository: String,
    #[serde(default)]
    pub kind: CompanionKind,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum CompanionKind {
    #[default]
    Plugin,
    Skill,
}

impl Companion {
    pub fn marketplace(&self, harness: &str) -> io::Result<&str> {
        self.marketplace.as_deref().ok_or_else(|| {
            io::Error::other(format!(
                "harnesses/{harness}/harness.toml: companion \"{}\" has no marketplace",
                self.plugin
            ))
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct References {
    skill: String,
    command: String,
    agent: String,
    option: String,
}

impl References {
    pub fn template(&self, kind: Kind) -> &str {
        match kind {
            Kind::Skill => &self.skill,
            Kind::Command => &self.command,
            Kind::Agent => &self.agent,
        }
    }

    pub fn option(&self, name: &str) -> String {
        self.option.replace("{name}", name)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Access {
    #[serde(rename = "read-only")]
    pub read_only: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Neutrality {
    names: Vec<String>,
    paths: Vec<String>,
    tools: Vec<String>,
}

impl Neutrality {
    pub fn terms(&self) -> impl Iterator<Item = &str> {
        self.names
            .iter()
            .chain(&self.paths)
            .chain(&self.tools)
            .map(String::as_str)
    }
}

pub fn read_all(root: &Path, diagnostics: &mut Vec<Diagnostic>) -> io::Result<Vec<Harness>> {
    let directory = root.join("harnesses");
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut folders = fs::read_dir(&directory)
        .map_err(naming(&directory))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    folders.sort();
    let mut harnesses = Vec::new();
    for folder in folders.iter().filter(|folder| folder.is_dir()) {
        let name = folder
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let path = format!("harnesses/{name}/harness.toml");
        let file = folder.join("harness.toml");
        if !file.is_file() {
            diagnostics.push(Diagnostic::general(format!("{path} is missing")));
            continue;
        }
        let text = fs::read_to_string(&file).map_err(naming(&file))?;
        match toml::from_str::<Harness>(&text) {
            Ok(harness) => {
                let limit = harness.contract.as_ref().map(|table| table.limit);
                if limit.is_some_and(|limit| !(1..=1_000_000).contains(&limit)) {
                    diagnostics.push(Diagnostic::in_file(
                        &path,
                        "[contract] limit must be 1 to 1000000".to_owned(),
                    ));
                }
                harnesses.push(Harness { name, ..harness });
            }
            Err(error) => diagnostics.push(Diagnostic::in_file(&path, error.message().to_owned())),
        }
    }
    Ok(harnesses)
}
