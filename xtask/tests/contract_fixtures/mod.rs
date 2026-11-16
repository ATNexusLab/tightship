use std::fs;

use super::common::Repository;

pub fn contract(front_matter: &str, body: &str) -> (&'static str, String) {
    let text = if front_matter.is_empty() {
        body.to_owned()
    } else {
        format!("+++\n{front_matter}+++\n\n{body}")
    };
    ("content/contract.md", text)
}

pub fn options(names: &[&str]) -> (&'static str, String) {
    let text = names
        .iter()
        .map(|name| {
            format!("[[options]]\nname = \"{name}\"\ntitle = \"An option\"\ndescription = \"An option\"\ntype = \"boolean\"\n")
        })
        .collect::<Vec<String>>()
        .join("\n");
    ("content/options.toml", text)
}

pub fn read(repository: &Repository, path: &str) -> String {
    fs::read_to_string(repository.path(path)).expect("a built file")
}
