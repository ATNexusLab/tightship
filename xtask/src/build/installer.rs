use std::fs;
use std::io;
use std::path::Path;

use super::harness::Harness;
use super::scripts::{self, Shell};
use crate::diagnostics::naming;

type Registry = fn(&[Harness]) -> String;

const FILES: [(&str, Shell, Registry); 2] = [
    ("install.sh", Shell::Sh, registry_sh),
    ("install.ps1", Shell::Pwsh, registry_ps1),
];

pub fn assemble(root: &Path, harnesses: &[Harness]) -> io::Result<Vec<(&'static str, String)>> {
    let repository = harnesses
        .first()
        .map(|harness| harness.plugin.repository.as_str())
        .unwrap_or_default();
    let mut installers = Vec::new();
    for (file, shell, registry) in FILES {
        let core_path = root.join("installer").join(file);
        let core = fs::read_to_string(&core_path).map_err(naming(&core_path))?;
        let mut fragments = Vec::new();
        for harness in harnesses {
            let path = root.join("harnesses").join(&harness.name).join(file);
            let text = fs::read_to_string(&path).map_err(naming(&path))?;
            let source = format!("harnesses/{}/{file}", harness.name);
            fragments.push(scripts::fill(&text, &source, harness, shell)?);
        }
        let unresolved = || io::Error::other(format!("installer/{file}: unresolved tokens"));
        let mut parts = Vec::new();
        for part in core.split("{{fragments}}") {
            let filled = part
                .replace("{{repository}}", repository)
                .replace("{{harness-choices}}", &choices(harnesses))
                .replace("{{harnesses}}", &registry(harnesses));
            if filled.contains("{{") {
                return Err(unresolved());
            }
            parts.push(filled);
        }
        if parts.len() != 2 {
            return Err(unresolved());
        }
        installers.push((file, parts.join(&fragments.join("\n"))));
    }
    Ok(installers)
}

fn choices(harnesses: &[Harness]) -> String {
    let names: Vec<&str> = harnesses
        .iter()
        .map(|harness| harness.name.as_str())
        .collect();
    match names.as_slice() {
        [only] => format!("{only} or all"),
        _ => format!("{}, or all", names.join(", ")),
    }
}

fn registry_sh(harnesses: &[Harness]) -> String {
    let quoted = |value: &str| format!("'{}'", value.replace('\'', r"'\''"));
    let mut named = String::new();
    let mut fields = String::new();
    for harness in harnesses {
        let name = &harness.name;
        let install = &harness.install;
        let mut patterns = vec![name.as_str()];
        patterns.extend(install.aliases.iter().map(String::as_str));
        named.push_str(&format!("    {}) echo {name} ;;\n", patterns.join("|")));
        let id = name.replace('-', "_");
        for (field, value) in [
            ("id", id.as_str()),
            ("label", &install.label),
            ("command", &install.command),
            ("min-version", &install.min_version),
        ] {
            fields.push_str(&format!(
                "    \"{name} {field}\") echo {} ;;\n",
                quoted(value)
            ));
        }
    }
    let names: Vec<&str> = harnesses
        .iter()
        .map(|harness| harness.name.as_str())
        .collect();
    format!(
        "HARNESSES=\"{}\"\n\nharness_named() {{\n  case \"$1\" in\n{named}    *) return 1 ;;\n  esac\n}}\n\nharness_field() {{\n  case \"$1 $2\" in\n{fields}  esac\n}}",
        names.join(" ")
    )
}

fn registry_ps1(harnesses: &[Harness]) -> String {
    let quoted = |value: &str| format!("'{}'", value.replace('\'', "''"));
    let entries: Vec<String> = harnesses
        .iter()
        .map(|harness| {
            let install = &harness.install;
            let aliases: Vec<String> = install.aliases.iter().map(|alias| quoted(alias)).collect();
            format!(
                "    {} = @{{ Id = {}; Label = {}; Command = {}; MinVersion = {}; Aliases = @({}) }}",
                quoted(&harness.name),
                quoted(&scripts::pascal(&harness.name)),
                quoted(&install.label),
                quoted(&install.command),
                quoted(&install.min_version),
                aliases.join(", ")
            )
        })
        .collect();
    format!("$Harnesses = [ordered]@{{\n{}\n}}", entries.join("\n"))
}
