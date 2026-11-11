//! The names a source file declares at its top level, read line by line for each language.

/// The functions, types, and classes the text declares, for the language its extension names.
pub fn names(extension: &str, text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in text.lines() {
        let declared = match extension {
            "py" => python(line),
            "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" => javascript(line),
            "go" => go(line),
            "rs" => rust(line),
            "rb" => ruby(line),
            "java" | "kt" | "swift" | "cs" | "php" => class_like(line),
            _ => Vec::new(),
        };
        for name in declared {
            if !name.starts_with('_') && !found.contains(&name) {
                found.push(name);
            }
        }
    }
    found
}

fn identifier(text: &str) -> Option<String> {
    let name: String = text
        .chars()
        .take_while(|character| {
            character.is_alphanumeric() || *character == '_' || *character == '$'
        })
        .collect();
    let first = name.chars().next()?;
    (!first.is_numeric()).then_some(name)
}

fn after_word<'a>(line: &'a str, word: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(word)?;
    rest.starts_with(char::is_whitespace)
        .then(|| rest.trim_start())
}

fn python(line: &str) -> Vec<String> {
    let declaration = after_word(line, "def")
        .or_else(|| after_word(line, "async").and_then(|rest| after_word(rest, "def")))
        .or_else(|| after_word(line, "class"));
    declaration.and_then(identifier).into_iter().collect()
}

fn javascript(line: &str) -> Vec<String> {
    const KEYWORDS: [&str; 10] = [
        "default",
        "async",
        "function",
        "class",
        "const",
        "let",
        "var",
        "interface",
        "type",
        "enum",
    ];
    let Some(rest) = line.strip_prefix("export") else {
        return Vec::new();
    };
    let rest = rest.trim_start();
    if let Some(list) = rest.strip_prefix('{') {
        let list = list.split('}').next().unwrap_or(list);
        return list
            .split(',')
            .filter_map(|item| {
                let words: Vec<&str> = item.split_whitespace().collect();
                match words.as_slice() {
                    [.., "as", alias] => identifier(alias),
                    [name, ..] => identifier(name),
                    [] => None,
                }
            })
            .collect();
    }
    let separator = |character: char| character.is_whitespace() || character == '*';
    let mut rest = rest;
    let mut saw_keyword = false;
    loop {
        let word = rest.split(separator).next().unwrap_or("");
        if !KEYWORDS.contains(&word) {
            break;
        }
        saw_keyword = true;
        rest = rest[word.len()..].trim_start_matches(separator);
    }
    if !saw_keyword {
        return Vec::new();
    }
    identifier(rest).into_iter().collect()
}

fn go(line: &str) -> Vec<String> {
    let capitalized = |name: &String| name.starts_with(char::is_uppercase);
    let name = if let Some(rest) = line.strip_prefix("func") {
        let rest = rest.trim_start();
        match rest.strip_prefix('(') {
            Some(receiver) => receiver
                .split_once(')')
                .and_then(|(_, after)| identifier(after.trim_start())),
            None if line.starts_with("func ") => identifier(rest),
            None => None,
        }
    } else {
        after_word(line, "type").and_then(identifier)
    };
    name.filter(capitalized).into_iter().collect()
}

fn rust(line: &str) -> Vec<String> {
    let Some(rest) = after_word(line, "pub") else {
        return Vec::new();
    };
    let rest = after_word(rest, "async")
        .and_then(|rest| after_word(rest, "fn"))
        .or_else(|| {
            ["fn", "struct", "enum", "trait", "type"]
                .iter()
                .find_map(|word| after_word(rest, word))
        });
    rest.and_then(identifier).into_iter().collect()
}

fn ruby(line: &str) -> Vec<String> {
    let line = line.trim_start();
    if let Some(rest) = after_word(line, "def") {
        let rest = rest.strip_prefix("self.").unwrap_or(rest);
        return identifier(rest).into_iter().collect();
    }
    after_word(line, "class")
        .or_else(|| after_word(line, "module"))
        .and_then(identifier)
        .into_iter()
        .collect()
}

fn class_like(line: &str) -> Vec<String> {
    let mut rest = line.trim_start();
    while let Some(after) = ["public", "static", "final"]
        .iter()
        .find_map(|word| after_word(rest, word))
    {
        rest = after;
    }
    ["class", "interface", "enum", "record"]
        .iter()
        .find_map(|word| after_word(rest, word))
        .and_then(identifier)
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn reads_python_names() {
        let text = "def load():\nasync def fetch():\nclass Store(Base):\n    def inner(self):\ndef _hidden():\n";
        assert_eq!(names("py", text), owned(&["load", "fetch", "Store"]));
    }

    #[test]
    fn reads_javascript_names() {
        let text = "\
export default function render() {}
export async function load() {}
export const LIMIT = 3;
export interface Props {}
export type Id = string;
export enum Kind {}
export class Store {}
export { alpha, beta as gamma };
export default {};
export * from './x';
const local = 1;
export function* walk() {}
";
        assert_eq!(
            names("ts", text),
            owned(&[
                "render", "load", "LIMIT", "Props", "Id", "Kind", "Store", "alpha", "gamma", "walk"
            ])
        );
    }

    #[test]
    fn reads_go_names() {
        let text = "\
func Run() {}
func helper() {}
func (s *Server) Serve() {}
func (s *Server) stop() {}
type Config struct {}
type internal int
";
        assert_eq!(names("go", text), owned(&["Run", "Serve", "Config"]));
    }

    #[test]
    fn reads_rust_names() {
        let text = "\
pub fn run() {}
pub async fn fetch() {}
pub struct Note;
pub enum Kind {}
pub trait Store {}
pub type Id = u32;
fn private() {}
    pub fn method(&self) {}
pub mod nested;
pub fn run() {}
";
        assert_eq!(
            names("rs", text),
            owned(&["run", "fetch", "Note", "Kind", "Store", "Id"])
        );
    }

    #[test]
    fn reads_ruby_and_class_like_names() {
        let ruby_text =
            "module Billing\n  class Invoice < Base\n    def self.build\n    def total\n";
        assert_eq!(
            names("rb", ruby_text),
            owned(&["Billing", "Invoice", "build", "total"])
        );
        let java_text = "public final class Store {\n  public static enum Kind {}\n  record Point() {}\n  void run() {}\n";
        assert_eq!(names("java", java_text), owned(&["Store", "Kind", "Point"]));
    }
}
