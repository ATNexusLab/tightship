//! The codebase map hook.

use super::*;

fn map_in(sandbox: &Sandbox, folder: &Path) -> std::process::Output {
    let mut command = sandbox.tightship(&["hook", "map"]);
    command.current_dir(folder);
    run(&mut command)
}

fn git_repository(sandbox: &Sandbox, name: &str, files: &[(&str, &str)]) -> PathBuf {
    for (path, text) in files {
        sandbox.write(&format!("{name}/{path}"), text);
    }
    let folder = sandbox.path(name);
    for arguments in [&["init", "-q"][..], &["add", "-A"][..]] {
        let mut git = Command::new("git");
        git.args(arguments)
            .current_dir(&folder)
            .env("GIT_CONFIG_GLOBAL", sandbox.path(".gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1");
        assert_succeeds(&run(&mut git));
    }
    folder
}

#[test]
fn map_lists_each_source_folder_with_its_names_and_skips_tests_and_generated_files() {
    let sandbox = Sandbox::new();
    let folder = git_repository(
        &sandbox,
        "project",
        &[
            ("src/lib.rs", "pub fn run() {}\npub struct Store;\n"),
            ("web/app.ts", "export function render() {}\n"),
            ("web/app.test.ts", "export function testOnly() {}\n"),
            ("web/types.d.ts", "export interface Declared {}\n"),
            ("tests/map.rs", "pub fn fixture() {}\n"),
            ("dist/bundle.js", "export const built = 1;\n"),
        ],
    );

    let output = map_in(&sandbox, &folder);

    assert_succeeds(&output);
    let map = stdout_of(&output);
    assert!(map.starts_with("# Codebase map\n"), "{map}");
    assert!(map.contains("\nsrc/: run, Store\n"), "{map}");
    assert!(map.contains("\nweb/: render\n"), "{map}");
    for absent in ["testOnly", "Declared", "fixture", "built"] {
        assert!(!map.contains(absent), "{absent} in {map}");
    }
}

#[test]
fn map_past_its_budget_counts_the_folders_it_leaves_out() {
    let sandbox = Sandbox::new();
    let long = "a_name_long_enough_to_fill_the_budget_quickly".repeat(2);
    let text = format!("pub fn {long}() {{}}\n");
    let files: Vec<(String, &str)> = (0..40)
        .map(|number| (format!("folder{number:02}/lib.rs"), text.as_str()))
        .collect();
    let files: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), *text))
        .collect();
    let folder = git_repository(&sandbox, "project", &files);

    let output = map_in(&sandbox, &folder);

    assert_succeeds(&output);
    let map = stdout_of(&output);
    let lines: usize = map
        .lines()
        .filter(|line| line.contains("/: "))
        .map(|line| line.chars().count() + 1)
        .sum();
    assert!(lines <= 2_000, "{lines}");
    let last = map.lines().last().expect("a last line");
    assert!(
        last.starts_with('(') && last.ends_with(" more folders not listed)"),
        "{map}"
    );
}

#[test]
fn map_prints_nothing_with_the_contract_off_or_without_names() {
    let sandbox = Sandbox::new();
    let folder = git_repository(&sandbox, "project", &[("src/lib.rs", "pub fn run() {}\n")]);
    let data = sandbox.write("data/options.json", r#"{"contract": false}"#);
    let mut command = sandbox.tightship(&["hook", "map"]);
    command
        .current_dir(&folder)
        .env("TIGHTSHIP_DATA", data.parent().expect("the data folder"));
    let contract_off = run(&mut command);
    assert_succeeds(&contract_off);
    assert_eq!(stdout_of(&contract_off), "");

    let empty = sandbox.path("empty");
    sandbox.write("empty/README.md", "# Nothing to map\n");
    let no_names = map_in(&sandbox, &empty);
    assert_succeeds(&no_names);
    assert_eq!(stdout_of(&no_names), "");
}
