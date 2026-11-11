use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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
    pub clean: Option<Clean>,
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

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Clean {
    pub names: Vec<String>,
    #[serde(default)]
    pub previous_versions: bool,
    #[serde(default)]
    pub commands: Vec<CleanCommand>,
    #[serde(default)]
    pub paths: Vec<CleanPath>,
    #[serde(default)]
    pub json: Vec<CleanJson>,
    #[serde(default)]
    pub status_line: Vec<CleanStatusLine>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CleanCommand {
    pub when: String,
    pub run: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CleanPath {
    pub path: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CleanJson {
    pub file: String,
    pub remove: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CleanStatusLine {
    pub contains: String,
}

const COMPANIONS: [&str; 3] = ["ponytail", "i-have-adhd", "humanizer"];

impl Clean {
    fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let well_named = |name: &String| {
            (1..=64).contains(&name.len())
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        };
        if !(1..=10).contains(&self.names.len()) || !self.names.iter().all(well_named) {
            problems.push("[clean] names must be 1 to 10 names of [a-z0-9-]{1,64}".to_owned());
        }
        let paths = self
            .commands
            .iter()
            .map(|command| &command.when)
            .chain(self.paths.iter().map(|path| &path.path))
            .chain(self.json.iter().map(|json| &json.file));
        for path in paths {
            let in_home = path.starts_with("~/") || path.starts_with("{config}/");
            if !in_home || path.split('/').any(|part| part == "..") {
                problems.push(format!("[clean] path {path} is outside the home folder"));
            }
        }
        problems
    }
}

fn companions_outside_clean(text: &str) -> Vec<&'static str> {
    let Ok(mut table) = text.parse::<toml::Table>() else {
        return Vec::new();
    };
    table.remove("clean");
    let rest = table.to_string();
    COMPANIONS
        .into_iter()
        .filter(|name| rest.contains(name))
        .collect()
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

pub fn folders(root: &Path) -> io::Result<Vec<PathBuf>> {
    let directory = root.join("harnesses");
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut folders = fs::read_dir(&directory)
        .map_err(naming(&directory))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    folders.retain(|folder| folder.is_dir());
    folders.sort();
    Ok(folders)
}

pub fn read_all(root: &Path, diagnostics: &mut Vec<Diagnostic>) -> io::Result<Vec<Harness>> {
    let mut harnesses = Vec::new();
    for folder in &folders(root)? {
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
                let problems = harness
                    .clean
                    .as_ref()
                    .map(Clean::problems)
                    .unwrap_or_default();
                for problem in problems {
                    diagnostics.push(Diagnostic::in_file(&path, problem));
                }
                for companion in companions_outside_clean(&text) {
                    diagnostics.push(Diagnostic::in_file(
                        &path,
                        format!(
                            "{companion} is named outside [clean]; tightship depends on no other plugin"
                        ),
                    ));
                }
                harnesses.push(Harness { name, ..harness });
            }
            Err(error) => diagnostics.push(Diagnostic::in_file(&path, error.message().to_owned())),
        }
    }
    if harnesses
        .windows(2)
        .any(|pair| pair[0].plugin.repository != pair[1].plugin.repository)
    {
        let named: Vec<String> = harnesses
            .iter()
            .map(|harness| format!("{} \"{}\"", harness.name, harness.plugin.repository))
            .collect();
        diagnostics.push(Diagnostic::general(format!(
            "the harnesses name different repositories: {}; use one",
            named.join(", ")
        )));
    }
    Ok(harnesses)
}
