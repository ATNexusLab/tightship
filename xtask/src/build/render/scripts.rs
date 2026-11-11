//! Fills the tokens of an install script or a launcher for one harness and one shell.

use std::io;

use crate::build::harness::Harness;

/// The shell a script is written for.
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

/// `antigravity-cli` as `AntigravityCli`.
pub fn kebab_to_pascal(kebab: &str) -> String {
    kebab
        .split('-')
        .map(|word| {
            let (first, rest) = word.split_at(1);
            first.to_uppercase() + rest
        })
        .collect()
}

/// Fills each `{{path:…}}` and `{{install:…}}` token, indenting a multi-line value like its token.
///
/// # Errors
///
/// When a token has no value for this harness.
pub fn fill_template(
    text: &str,
    source: &str,
    harness: &Harness,
    shell: Shell,
) -> io::Result<String> {
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
        ("install", name) => harness.install.field(name).map(str::to_owned),
        _ => None,
    }
}
