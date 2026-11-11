//! The map hook: lists the names each folder of the project declares, so the agent reuses them.

mod names;

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::failure::Failure;
use names::names;

const GIT_LIMIT: Duration = Duration::from_secs(3);
const MAXIMUM_PATHS: usize = 20_000;
const MAXIMUM_WALK_DEPTH: usize = 6;
const MAXIMUM_KEPT: usize = 4_000;
const MAXIMUM_BYTES: u64 = 65_536;
const MAXIMUM_NAMES: usize = 30;
const BUDGET: usize = 2_000;

const EXTENSIONS: [&str; 15] = [
    "py", "js", "jsx", "mjs", "cjs", "ts", "tsx", "go", "rs", "rb", "java", "kt", "php", "swift",
    "cs",
];
const SKIPPED_FOLDERS: [&str; 19] = [
    "node_modules",
    ".git",
    "dist",
    "build",
    "out",
    "vendor",
    "venv",
    ".venv",
    "__pycache__",
    "coverage",
    ".next",
    "target",
    "migrations",
    "versions",
    "fixtures",
    "test",
    "tests",
    "__tests__",
    "spec",
];
const SKIPPED_SUFFIXES: [&str; 6] = ["_test", ".test", ".spec", ".gen", ".min", ".d"];
const PRIORITY_WORDS: [&str; 12] = [
    "util",
    "helper",
    "lib",
    "common",
    "shared",
    "service",
    "component",
    "hook",
    "core",
    "model",
    "schema",
    "api",
];

/// Prints the codebase map of the working folder, from the files git tracks or else a bounded walk.
///
/// # Errors
///
/// When the options are invalid or the map cannot be emitted.
pub fn run() -> Result<(), Failure> {
    if !crate::options::load()?.contract {
        return Ok(());
    }
    let Ok(folder) = env::current_dir() else {
        return Ok(());
    };
    let paths = match tracked(&folder) {
        Some(paths) if !paths.is_empty() => paths,
        _ => {
            let mut paths = Vec::new();
            walk(&folder, "", 0, &mut paths);
            paths
        }
    };
    let files: Vec<(String, Vec<String>)> = ordered(paths)
        .into_iter()
        .filter_map(|path| {
            let text = read_head(&folder.join(&path))?;
            let names = names(extension(&path), &text);
            (!names.is_empty()).then_some((path, names))
        })
        .collect();
    super::emit(&render(&files))
}

fn tracked(folder: &Path) -> Option<Vec<String>> {
    let mut child = crate::git::command(folder)
        .arg("ls-files")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut output = Vec::new();
        let read = stdout.read_to_end(&mut output).map(|_| output);
        // The receiver is gone once the time limit passed, and nothing waits for the output.
        let _ = sender.send(read);
    });
    let Ok(Ok(output)) = receiver.recv_timeout(GIT_LIMIT) else {
        // A git that does not answer in time falls back to walking the folder.
        let _ = child.kill();
        let _ = child.wait();
        return None;
    };
    if !child.wait().ok()?.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output)
            .lines()
            .take(MAXIMUM_PATHS)
            .map(str::to_owned)
            .collect(),
    )
}

fn walk(folder: &Path, prefix: &str, depth: usize, paths: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        if paths.len() >= MAXIMUM_PATHS {
            return;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = format!("{prefix}{name}");
        if kind.is_dir() {
            if depth + 1 < MAXIMUM_WALK_DEPTH && !SKIPPED_FOLDERS.contains(&name.as_str()) {
                walk(&entry.path(), &format!("{path}/"), depth + 1, paths);
            }
        } else if kind.is_file() {
            paths.push(path);
        }
    }
}

fn read_head(file: &Path) -> Option<String> {
    if !fs::symlink_metadata(file).ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    File::open(file)
        .ok()?
        .take(MAXIMUM_BYTES)
        .read_to_end(&mut bytes)
        .ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn extension(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.rsplit_once('.').map_or("", |(_, extension)| extension)
}

fn is_kept(path: &str) -> bool {
    let allowed = |character: char| {
        character.is_ascii_alphanumeric() || "._@+$()[]/-".contains(character) || character == '_'
    };
    if path.is_empty() || !path.chars().all(allowed) {
        return false;
    }
    let mut parts: Vec<&str> = path.split('/').collect();
    let Some(name) = parts.pop() else {
        return false;
    };
    if parts.iter().any(|part| SKIPPED_FOLDERS.contains(part)) {
        return false;
    }
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return false;
    };
    EXTENSIONS.contains(&extension)
        && !stem.starts_with("test_")
        && !SKIPPED_SUFFIXES.iter().any(|suffix| stem.ends_with(suffix))
}

fn ordered(paths: Vec<String>) -> Vec<String> {
    let mut kept: Vec<String> = paths
        .into_iter()
        .filter(|path| is_kept(path))
        .take(MAXIMUM_KEPT)
        .collect();
    kept.sort_by_cached_key(|path| {
        let lower = path.to_lowercase();
        let priority = PRIORITY_WORDS.iter().any(|word| lower.contains(word));
        (!priority, path.clone())
    });
    kept
}

fn render(files: &[(String, Vec<String>)]) -> String {
    let mut folders: Vec<(String, Vec<&[String]>)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for (path, names) in files {
        let folder = match path.rsplit_once('/') {
            Some((parent, _)) => format!("{parent}/"),
            None => "./".to_owned(),
        };
        let position = *index.entry(folder.clone()).or_insert_with(|| {
            folders.push((folder, Vec::new()));
            folders.len() - 1
        });
        folders[position].1.push(names);
    }
    let mut lines = String::new();
    let mut used = 0;
    let mut skipped = 0;
    for (folder, files) in &folders {
        let take = if files.len() > 8 { 1 } else { 3 };
        let mut names: Vec<&str> = Vec::new();
        for name in files.iter().flat_map(|names| names.iter().take(take)) {
            if !names.contains(&name.as_str()) {
                names.push(name);
            }
        }
        let mut listed = names
            .iter()
            .take(MAXIMUM_NAMES)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        if names.len() > MAXIMUM_NAMES {
            listed.push_str(", ...");
        }
        let line = format!("{folder}: {listed}\n");
        let length = line.chars().count();
        if used + length > BUDGET {
            skipped += 1;
        } else {
            used += length;
            lines.push_str(&line);
        }
    }
    if lines.is_empty() {
        return String::new();
    }
    let mut map = format!(
        "# Codebase map\n\nWhat already exists, one folder per line; reuse it, and read a file only \
         when you need its details.\n\n{lines}"
    );
    if skipped > 0 {
        map.push_str(&format!("({skipped} more folders not listed)\n"));
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn keeps_source_files_and_skips_tests_generated_and_vendored_ones() {
        for path in [
            "main.rs",
            "src/hook/map.rs",
            "web/app.tsx",
            "pkg/my_module.py",
            "lib/[id]/page.ts",
        ] {
            assert!(is_kept(path), "{path} should be kept");
        }
        for path in [
            "README.md",
            "node_modules/react/index.js",
            "src/tests/map.rs",
            "build/out.js",
            "types/index.d.ts",
            "app/test_api.py",
            "app/api_test.go",
            "app/api.spec.ts",
            "app/bundle.min.js",
            "app/schema.gen.ts",
            "app/has space.rs",
            "app/ünicode.rs",
            "Makefile",
        ] {
            assert!(!is_kept(path), "{path} should be skipped");
        }
    }

    #[test]
    fn orders_priority_paths_first_then_by_name() {
        let paths = owned(&[
            "b/main.rs",
            "a/main.rs",
            "z/Utils.rs",
            "README.md",
            "x/api.ts",
        ]);
        assert_eq!(
            ordered(paths),
            owned(&["x/api.ts", "z/Utils.rs", "a/main.rs", "b/main.rs"])
        );
    }

    #[test]
    fn renders_one_line_per_folder_in_the_order_of_its_first_file() {
        let files = vec![
            (
                "src/hook.rs".to_owned(),
                owned(&["emit", "emit_checkpoint", "run", "extra"]),
            ),
            ("main.rs".to_owned(), owned(&["main"])),
            ("src/git.rs".to_owned(), owned(&["run", "is_installed"])),
        ];
        assert_eq!(
            render(&files),
            "# Codebase map\n\nWhat already exists, one folder per line; reuse it, and read a file only \
             when you need its details.\n\nsrc/: emit, emit_checkpoint, run, is_installed\n./: main\n"
        );
    }

    #[test]
    fn takes_one_name_per_file_past_eight_files_and_caps_at_thirty() {
        let many: Vec<(String, Vec<String>)> = (0..40)
            .map(|number| {
                (
                    format!("src/file{number}.rs"),
                    owned(&[&format!("first{number}"), "second"]),
                )
            })
            .collect();
        let map = render(&many);
        let line = map
            .lines()
            .find(|line| line.starts_with("src/"))
            .expect("the line");
        assert!(!line.contains("second"));
        assert_eq!(line.matches(", ").count(), 30);
        assert!(line.ends_with("first29, ..."));
    }

    #[test]
    fn skips_lines_past_the_budget_and_counts_them() {
        let long = "n".repeat(900);
        let files = vec![
            ("a/x.rs".to_owned(), vec![long.clone()]),
            ("b/x.rs".to_owned(), vec![long.clone()]),
            ("c/x.rs".to_owned(), vec![long]),
            ("d/x.rs".to_owned(), owned(&["short"])),
        ];
        let map = render(&files);
        assert!(map.contains("d/: short\n"));
        assert!(!map.contains("c/:"));
        assert!(map.ends_with("(1 more folders not listed)\n"));
    }

    #[test]
    fn renders_nothing_without_names() {
        assert_eq!(render(&[]), "");
    }
}
