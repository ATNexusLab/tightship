//! `tightship docs check`: reports where the docs drifted from the repository.

use std::collections::{HashMap, HashSet};

use super::markdown::{self, Parsed};
use super::{Doc, Repository, glob};
use crate::failure::Failure;

const MARKERS: [&str; 3] = ["TODO", "FIXME", "TBD"];

/// One problem a doc has, on one line.
struct Finding {
    path: String,
    line: usize,
    problem: String,
}

/// What a file of code names, read once and shared by every doc that reads it.
struct Known {
    identifiers: HashSet<String>,
    words: HashSet<String>,
}

/// A reference to a path the repository may no longer hold, kept until the ignore rules are read.
struct PathReference {
    finding: Finding,
    path: String,
}

/// Prints every finding with its doc and line, then a summary.
///
/// # Errors
///
/// When the repository or a doc cannot be read, or `DocsCheckFailed` when anything was reported.
pub fn run() -> Result<(), Failure> {
    let repository = Repository::open()?;
    let docs = repository.docs()?;
    let parsed: HashMap<&str, Parsed> = docs
        .iter()
        .map(|doc| {
            (
                doc.path.as_str(),
                markdown::parse(&doc.text, doc.body_start),
            )
        })
        .collect();
    let mut outside: HashMap<String, Parsed> = HashMap::new();
    let mut known: HashMap<String, Known> = HashMap::new();
    let mut findings = Vec::new();
    let mut paths = Vec::new();
    let corpus = if docs.iter().any(|doc| !doc.covers.is_empty()) {
        corpus(&repository)?
    } else {
        String::new()
    };
    for doc in &docs {
        let own = &parsed[doc.path.as_str()];
        for link in &own.links {
            if let Some(problem) = check_link(&repository, doc, &link.text, &parsed, &mut outside)?
            {
                findings.push(found(doc, link.line, problem));
            }
        }
        if doc.path.starts_with("docs/work/") {
            continue;
        }
        markers(doc, own, &mut findings);
        coverage(&repository, doc, &mut findings);
        if !doc.covers.is_empty() {
            let context = Context {
                repository: &repository,
                corpus: &corpus,
            };
            references(&context, doc, own, &mut known, &mut findings, &mut paths)?;
        }
    }
    let candidates: Vec<String> = paths
        .iter()
        .map(|reference| reference.path.clone())
        .collect();
    let ignored: HashSet<String> = repository.ignored(&candidates)?.into_iter().collect();
    findings.extend(
        paths
            .into_iter()
            .filter(|reference| !ignored.contains(&reference.path))
            .map(|reference| reference.finding),
    );
    findings.sort_by(|left, right| (&left.path, left.line).cmp(&(&right.path, right.line)));
    for finding in &findings {
        println!("{}:{} — {}", finding.path, finding.line, finding.problem);
    }
    println!("{} docs, {} findings", docs.len(), findings.len());
    if findings.is_empty() {
        Ok(())
    } else {
        Err(Failure::DocsCheckFailed)
    }
}

fn found(doc: &Doc, line: usize, problem: String) -> Finding {
    Finding {
        path: doc.path.clone(),
        line,
        problem,
    }
}

/// The problem with a link, if its file or its anchor does not exist.
fn check_link(
    repository: &Repository,
    doc: &Doc,
    target: &str,
    parsed: &HashMap<&str, Parsed>,
    outside: &mut HashMap<String, Parsed>,
) -> Result<Option<String>, Failure> {
    if target.contains("://") || target.starts_with("mailto:") {
        return Ok(None);
    }
    let (path, anchor) = target.split_once('#').unwrap_or((target, ""));
    let path = path.replace("%20", " ");
    let resolved = if path.is_empty() {
        doc.path.clone()
    } else {
        let Some(resolved) = resolve(&doc.path, &path) else {
            return Ok(Some(format!("link to {target} leaves the repository")));
        };
        resolved
    };
    if !exists(repository, &resolved) {
        return Ok(Some(format!("link to {target}: no such file")));
    }
    if anchor.is_empty() || !resolved.ends_with(".md") {
        return Ok(None);
    }
    let anchors = match parsed.get(resolved.as_str()) {
        Some(parsed) => &parsed.anchors,
        None => {
            if !outside.contains_key(&resolved) {
                let text = repository.read(&resolved)?;
                outside.insert(resolved.clone(), markdown::parse(&text, 1));
            }
            &outside[&resolved].anchors
        }
    };
    Ok((!anchors.contains(anchor)).then(|| format!("link to {target}: no anchor #{anchor}")))
}

/// The path from the root that a link written in `doc` points at, or `None` when it leaves the root.
fn resolve(doc: &str, link: &str) -> Option<String> {
    let mut parts: Vec<&str> = match link.strip_prefix('/') {
        Some(_) => Vec::new(),
        None => doc.split('/').collect(),
    };
    if !link.starts_with('/') {
        parts.pop();
    }
    for part in link.trim_start_matches('/').split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            part => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

/// Whether the repository holds the path as a file or as a folder of files.
fn exists(repository: &Repository, path: &str) -> bool {
    let folder = format!("{}/", path.trim_end_matches('/'));
    repository
        .files
        .iter()
        .any(|file| file == path || file.starts_with(&folder) || path.is_empty())
}

fn markers(doc: &Doc, parsed: &Parsed, findings: &mut Vec<Finding>) {
    for item in &parsed.prose {
        for word in item.text.split(|c: char| !c.is_ascii_alphanumeric()) {
            if MARKERS.contains(&word) {
                findings.push(found(
                    doc,
                    item.line,
                    format!("{word} marker left in the doc"),
                ));
            }
        }
    }
}

fn coverage(repository: &Repository, doc: &Doc, findings: &mut Vec<Finding>) {
    for cover in &doc.covers {
        if !repository
            .files
            .iter()
            .any(|file| glob::matches(&cover.pattern, file))
        {
            findings.push(found(
                doc,
                cover.line,
                format!("covers `{}`, which matches no file", cover.pattern),
            ));
        }
    }
    let is_index = doc.path.ends_with("/README.md");
    if doc.covers.is_empty() && doc.path.starts_with("docs/system/") && !is_index {
        findings.push(found(
            doc,
            1,
            "covers no code; list the paths it describes under `covers` in its front matter"
                .to_owned(),
        ));
    }
}

/// What every doc's references are checked against: the repository and the text of its code.
struct Context<'a> {
    repository: &'a Repository,
    corpus: &'a str,
}

/// The text of every file outside the docs, which a path a doc names must still appear in or be.
fn corpus(repository: &Repository) -> Result<String, Failure> {
    let mut corpus = String::new();
    for file in &repository.files {
        if super::FOLDERS.iter().any(|folder| file.starts_with(folder)) {
            continue;
        }
        let text = repository.read(file)?;
        // A binary file names no path a reader follows, and its bytes would only slow the search.
        if !text.contains('\0') {
            corpus.push_str(&text);
            corpus.push('\n');
        }
    }
    Ok(corpus)
}

/// Checks each code span of a doc: a path against the repository, a command against the code of the
/// program it runs, and a symbol against the code the doc covers.
fn references(
    context: &Context<'_>,
    doc: &Doc,
    parsed: &Parsed,
    known: &mut HashMap<String, Known>,
    findings: &mut Vec<Finding>,
    paths: &mut Vec<PathReference>,
) -> Result<(), Failure> {
    let repository = context.repository;
    let anchors: HashSet<&str> = doc
        .covers
        .iter()
        .filter_map(|cover| cover.pattern.split('/').next())
        .collect();
    let covered = repository.covered(&doc.covers);
    let programs: Vec<&str> = repository
        .files
        .iter()
        .map(String::as_str)
        .filter(|file| {
            file.split_once('/')
                .is_some_and(|(top, _)| anchors.contains(top))
        })
        .collect();
    for file in covered.iter().chain(&programs) {
        if !known.contains_key(*file) {
            known.insert((*file).to_owned(), read_known(repository, file)?);
        }
    }
    let sources: Vec<&Known> = covered.iter().map(|file| &known[*file]).collect();
    for span in &parsed.spans {
        let text = span.text.trim();
        if is_path(text) {
            if !path_holds(repository, doc, text, context.corpus) {
                paths.push(PathReference {
                    finding: found(
                        doc,
                        span.line,
                        format!("`{text}` names no file in the repository"),
                    ),
                    path: text.trim_end_matches('/').to_owned(),
                });
            }
        } else if text.contains(char::is_whitespace) {
            if let Some((program, word)) = missing_command_word(text, &anchors, &programs, known) {
                findings.push(found(
                    doc,
                    span.line,
                    format!("`{text}`: `{word}` is in none of the code under `{program}/`"),
                ));
            }
        } else if let Some(name) = identifier(text)
            && !sources
                .iter()
                .any(|source| source.identifiers.contains(name))
        {
            findings.push(found(
                doc,
                span.line,
                format!("`{text}` is in none of the code the doc covers"),
            ));
        }
    }
    Ok(())
}

fn read_known(repository: &Repository, file: &str) -> Result<Known, Failure> {
    let text = repository.read(file)?;
    let identifiers = text
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect();
    let mut words: HashSet<String> = text
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .map(normalize)
        .filter(|word| !word.is_empty())
        .collect();
    for part in file.split(['/', '.']) {
        words.insert(normalize(part));
    }
    Ok(Known { identifiers, words })
}

/// A word as a subcommand compares to its code: lower case, with no `-` or `_`.
fn normalize(word: &str) -> String {
    word.chars()
        .filter(|c| *c != '-' && *c != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether a code span reads as a path: one token with a `/` or a file extension.
fn is_path(text: &str) -> bool {
    if text.is_empty()
        || text.contains(char::is_whitespace)
        || text.contains("://")
        || text.contains("::")
        || text.starts_with(['~', '/', '-'])
        || text.contains([
            '<', '>', '{', '}', '$', '%', '@', '=', '(', ')', '"', '\'', '|', '\\', ',',
        ])
    {
        return false;
    }
    if text.contains('/') {
        return true;
    }
    let Some((stem, extension)) = text.rsplit_once('.') else {
        return false;
    };
    let extension_reads = extension.len() <= 8
        && extension.starts_with(|c: char| c.is_ascii_lowercase())
        && extension
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    extension_reads && (!stem.is_empty() || text.starts_with('.'))
}

/// Whether the path exists in the repository or under the doc's folder, or the code outside the docs
/// still names it, as it names a file it generates.
fn path_holds(repository: &Repository, doc: &Doc, path: &str, corpus: &str) -> bool {
    let trimmed = path.trim_start_matches("./").trim_end_matches('/');
    let suffix = format!("/{trimmed}");
    let folder = format!("{trimmed}/");
    let inside = format!("/{trimmed}/");
    let from_doc = resolve(&doc.path, trimmed);
    repository.files.iter().any(|file| {
        (trimmed.contains('*') && glob::matches(trimmed, file))
            || file == trimmed
            || file.ends_with(&suffix)
            || file.starts_with(&folder)
            || file.contains(&inside)
            || from_doc.as_deref() == Some(file.as_str())
    }) || corpus.contains(trimmed)
}

/// The program a command runs, when it is the top folder of a path the doc covers, and the first word
/// after it that none of that program's code names.
fn missing_command_word<'a>(
    text: &'a str,
    anchors: &HashSet<&str>,
    programs: &[&str],
    known: &HashMap<String, Known>,
) -> Option<(&'a str, String)> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let start = tokens
        .iter()
        .take(2)
        .position(|token| anchors.contains(token))?;
    let program = tokens[start];
    let folder = format!("{program}/");
    let sources: Vec<&Known> = programs
        .iter()
        .filter(|file| file.starts_with(&folder))
        .map(|file| &known[*file])
        .collect();
    for token in &tokens[start + 1..] {
        let plain = token.replace('\\', "");
        let reads = !plain.is_empty()
            && plain
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '|');
        if !reads || plain.starts_with('-') {
            break;
        }
        for alternative in plain.split('|') {
            let word = normalize(alternative);
            if !word.is_empty() && !sources.iter().any(|source| source.words.contains(&word)) {
                return Some((program, alternative.to_owned()));
            }
        }
    }
    None
}

/// The name a code span gives a symbol, when it reads as one: `snake_case`, `CamelCase`, or `a::b`.
fn identifier(text: &str) -> Option<&str> {
    let text = text.strip_suffix("()").unwrap_or(text);
    let name = text.rsplit("::").next()?;
    let reads = |part: &str| {
        part.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    if name.len() < 3 || !text.split("::").all(reads) {
        return None;
    }
    let camel = name
        .as_bytes()
        .windows(2)
        .any(|pair| pair[0].is_ascii_lowercase() && pair[1].is_ascii_uppercase());
    let symbol = text.contains("::") || camel || name.trim_matches('_').contains('_');
    symbol.then_some(name)
}

#[cfg(test)]
mod tests {
    use super::{identifier, is_path, resolve};

    #[test]
    fn reads_paths_and_symbols() {
        assert!(is_path("tightship/src/docs.rs"));
        assert!(is_path("Cargo.toml"));
        assert!(is_path(".env"));
        assert!(!is_path("1.2.3"));
        assert!(!is_path("${TIGHTSHIP_DATA}/options.toml"));
        assert_eq!(
            identifier("Failure::DocsCheckFailed"),
            Some("DocsCheckFailed")
        );
        assert_eq!(identifier("repository_root()"), Some("repository_root"));
        assert_eq!(identifier("injectSteps"), Some("injectSteps"));
        assert_eq!(identifier("vault"), None);
        assert_eq!(identifier("COLUMNS"), None);
    }

    #[test]
    fn resolves_links_from_the_doc() {
        assert_eq!(
            resolve("docs/a/b.md", "../c.md").as_deref(),
            Some("docs/c.md")
        );
        assert_eq!(
            resolve("docs/a/b.md", "/README.md").as_deref(),
            Some("README.md")
        );
        assert_eq!(resolve("docs/b.md", "../../x.md"), None);
    }
}
