use std::env;
use std::fs;
use std::io;
use std::path::Path;

use super::words::is_part_of_a_word;
use crate::diagnostics::{Diagnostic, naming};
use crate::git;

pub fn terms(root: &Path, diagnostics: &mut Vec<Diagnostic>) -> io::Result<Option<Vec<String>>> {
    let mut terms = Vec::new();
    let mut complete = true;
    for key in ["user.name", "user.email"] {
        let value = git::run(root, &["config", "--default", "", key])?;
        if value.trim().is_empty() {
            complete = false;
        } else {
            terms.push(value.trim().to_owned());
        }
    }
    if !complete {
        diagnostics.push(Diagnostic::general(
            "git config user.name or user.email is empty".to_owned(),
        ));
    }
    let user_name = env::var("USER").or_else(|_| env::var("USERNAME"));
    terms.extend(user_name.ok().filter(|name| !name.is_empty()));
    if let Some(home) = env::home_dir() {
        let home = home.to_string_lossy();
        terms.push(home.replace('\\', "/"));
        terms.push(home.replace('/', "\\"));
    }
    let listed = root.join(".identity-terms");
    match fs::read_to_string(&listed) {
        Ok(text) => terms.extend(
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned),
        ),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            complete = false;
            diagnostics.push(Diagnostic::general(
                ".identity-terms is missing; create it with one line per name, email address, user name, and handle of the maintainer".to_owned(),
            ));
        }
        Err(error) => return Err(naming(&listed)(error)),
    }
    let lowercase = terms.iter().map(|term| term.to_lowercase()).collect();
    Ok(complete.then_some(lowercase))
}

pub fn check_folder(
    folder: &Path,
    shown_as: &str,
    terms: &[String],
    diagnostics: &mut Vec<Diagnostic>,
) -> io::Result<()> {
    for entry in fs::read_dir(folder).map_err(naming(folder))? {
        let path = entry.map_err(naming(folder))?.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        if holds_a_term(&file_name.to_lowercase(), terms) {
            diagnostics.push(Diagnostic::in_file(
                shown_as,
                "holds a file or folder named with an identity term of the maintainer".to_owned(),
            ));
            continue;
        }
        let shown = format!("{shown_as}/{file_name}");
        if path.is_dir() {
            check_folder(&path, &shown, terms, diagnostics)?;
            continue;
        }
        let bytes = fs::read(&path).map_err(naming(&path))?;
        check_text(&String::from_utf8_lossy(&bytes), &shown, terms, diagnostics);
    }
    Ok(())
}

pub fn check_text(text: &str, shown_as: &str, terms: &[String], diagnostics: &mut Vec<Diagnostic>) {
    for (index, line) in text.to_lowercase().split('\n').enumerate() {
        if holds_a_term(line, terms) {
            diagnostics.push(Diagnostic::at_line(
                shown_as,
                index + 1,
                "holds an identity term of the maintainer".to_owned(),
            ));
        }
    }
}

fn holds_a_term(text: &str, terms: &[String]) -> bool {
    terms.iter().any(|term| {
        text.match_indices(term.as_str())
            .any(|(start, _)| !is_part_of_a_word(text, start, term))
    })
}
