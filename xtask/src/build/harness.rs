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
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractTable {
    pub limit: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Companion {
    pub plugin: String,
    pub marketplace: String,
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
