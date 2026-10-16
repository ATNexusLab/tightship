use std::io;

use super::harness::{CompanionKind, Harness};

#[derive(Clone, Copy)]
pub enum Shell {
    Sh,
    Pwsh,
    Cmd,
}

impl Shell {
    fn home(self, relative: &str) -> String {
        match self {
            Shell::Sh => format!("$HOME/{relative}"),
            Shell::Pwsh => format!("$userProfile/{relative}"),
            Shell::Cmd => format!("%USERPROFILE%\\{}", relative.replace('/', "\\")),
        }
    }
}

pub fn pascal(kebab: &str) -> String {
    kebab
        .split('-')
        .map(|word| {
            let (first, rest) = word.split_at(1);
            first.to_uppercase() + rest
        })
        .collect()
}

pub fn fill(text: &str, source: &str, harness: &Harness, shell: Shell) -> io::Result<String> {
    let mut filled = String::new();
    let mut unresolved = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{")
        && let Some(length) = rest[start..].find("}}")
    {
        let token = &rest[start + 2..start + length];
        filled.push_str(&rest[..start]);
        let line_start = filled.rfind('\n').map_or(0, |newline| newline + 1);
        let indent = &filled[line_start..];
        let indent = if indent.trim().is_empty() {
            indent.to_owned()
        } else {
            String::new()
        };
        match value(token, harness, shell) {
            Some(value) => filled.push_str(&value.replace('\n', &format!("\n{indent}"))),
            None => unresolved.push(token),
        }
        rest = &rest[start + length + 2..];
    }
    filled.push_str(rest);
    if unresolved.is_empty() {
        Ok(filled)
    } else {
        Err(io::Error::other(format!(
            "{source}: unresolved tokens {}",
            unresolved.join(", ")
        )))
    }
}

fn value(token: &str, harness: &Harness, shell: Shell) -> Option<String> {
    match token.split_once(':')? {
        ("path", name) => harness.paths.get(name).map(|relative| shell.home(relative)),
        ("install", "companions") => companion_steps(harness, shell),
        ("install", name) => harness.install.field(name).map(str::to_owned),
        _ => None,
    }
}

fn companion_steps(harness: &Harness, shell: Shell) -> Option<String> {
    let steps = &harness.install.companion_steps;
    let lines = harness.companions.iter().map(|companion| {
        let kind = match companion.kind {
            CompanionKind::Plugin => steps.plugin.as_ref(),
            CompanionKind::Skill => steps.skill.as_ref(),
        }?;
        let template = match shell {
            Shell::Sh => &kind.sh,
            Shell::Pwsh => &kind.ps1,
            Shell::Cmd => return None,
        };
        Some(
            template
                .replace("{plugin}", &companion.plugin)
                .replace("{repository}", &companion.repository),
        )
    });
    lines
        .collect::<Option<Vec<String>>>()
        .map(|lines| lines.join("\n"))
}
