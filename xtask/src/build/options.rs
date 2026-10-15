use serde::Deserialize;

use crate::diagnostics::Diagnostic;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserOption {
    pub name: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: String,
}

impl UserOption {
    pub fn is_directory(&self) -> bool {
        self.kind == "directory"
    }
}

pub struct OptionsFile {
    pub path: String,
    pub text: String,
    pub declared: Vec<UserOption>,
}

pub fn read(path: &str, text: String, diagnostics: &mut Vec<Diagnostic>) -> OptionsFile {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Declarations {
        #[serde(default)]
        options: Vec<UserOption>,
    }
    let is_option_name = |name: &str| {
        name.len() <= 64
            && name.starts_with(|character: char| character.is_ascii_lowercase())
            && name.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
            })
    };
    let mut file = OptionsFile {
        path: path.to_owned(),
        text,
        declared: Vec::new(),
    };
    let declarations: Declarations = match toml::from_str(&file.text) {
        Ok(declarations) => declarations,
        Err(error) => {
            diagnostics.push(Diagnostic::in_file(path, error.message().to_owned()));
            return file;
        }
    };
    let mut problems = Vec::new();
    for (index, option) in declarations.options.iter().enumerate() {
        let name = &option.name;
        if !is_option_name(name) {
            problems.push(format!(
                "\"{name}\" is not an option name; use 1 to 64 lowercase letters, digits, and underscores, starting with a letter"
            ));
        }
        if !["boolean", "directory"].contains(&option.kind.as_str()) {
            problems.push(format!(
                "the type of \"{name}\" must be boolean or directory"
            ));
        }
        let texts = [
            ("title", &option.title, 40),
            ("description", &option.description, 120),
        ];
        for (key, text, limit) in texts {
            if !(1..=limit).contains(&text.chars().count()) {
                problems.push(format!(
                    "the {key} of \"{name}\" must be 1 to {limit} characters"
                ));
            }
        }
        let earlier_declarations = declarations.options[..index]
            .iter()
            .filter(|other| other.name == *name)
            .count();
        if earlier_declarations == 1 {
            problems.push(format!("the option \"{name}\" is declared twice"));
        }
    }
    if problems.is_empty() {
        file.declared = declarations.options;
        return file;
    }
    diagnostics.extend(
        problems
            .into_iter()
            .map(|problem| Diagnostic::in_file(path, problem)),
    );
    file
}
