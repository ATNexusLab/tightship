use std::io;

use super::harness::Harness;

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
        match value(token, harness, shell) {
            Some(value) => filled.push_str(&value),
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
