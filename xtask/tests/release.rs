mod common;

use std::collections::BTreeMap;
use std::fs;
use std::process::Output;

use sha2::{Digest, Sha256};
use zip::ZipArchive;

use common::{Repository, assert_fails_with, assert_succeeds};

const BINARIES: [&str; 4] = [
    "macos/tightship",
    "linux-x64/tightship",
    "linux-arm64/tightship",
    "windows-x64/tightship.exe",
];

fn release_repository(version: &str) -> Repository {
    let repository = Repository::with(&[
        (
            "Cargo.toml",
            format!("[workspace.package]\nversion = \"{version}\"\n"),
        ),
        (
            "content/commands/epic.md",
            "+++\nname = \"epic\"\ndescription = \"A piece\"\n+++\nOpen the epic.\n".to_owned(),
        ),
    ]);
    for binary in BINARIES {
        repository.write(
            &format!("binaries/{binary}"),
            &format!("prebuilt {binary}\n"),
        );
    }
    repository
}

fn run(repository: &Repository, arguments: &[&str]) -> Output {
    let (subcommand, rest) = arguments.split_first().expect("a subcommand");
    repository
        .xtask(subcommand)
        .args(rest)
        .output()
        .expect("xtask runs")
}

fn built_file_count(output: &Output) -> usize {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .find(|line| line.starts_with("Built dist/claude-code/"))
        .expect("the build summary");
    let before_files = line.strip_suffix(" files.").expect("a file count");
    let (_, count) = before_files.rsplit_once(' ').expect("a file count");
    count.parse().expect("a number")
}

fn plugin_files(repository: &Repository) -> Vec<String> {
    fn walk(folder: &std::path::Path, prefix: &str, files: &mut Vec<String>) {
        for entry in fs::read_dir(folder).expect("a folder") {
            let entry = entry.expect("an entry");
            let relative = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.file_type().expect("a file type").is_dir() {
                walk(&entry.path(), &format!("{relative}/"), files);
            } else {
                files.push(relative);
            }
        }
    }
    let mut files = Vec::new();
    walk(&repository.path("dist/claude-code"), "", &mut files);
    files.sort();
    files
}

#[test]
fn a_build_with_the_four_binaries_copies_them_without_compiling() {
    let repository = release_repository("0.1.0");
    let host = repository.run("build");
    assert_succeeds(&host);

    let prebuilt = run(&repository, &["build", "--binaries", "binaries"]);
    assert_succeeds(&prebuilt);
    assert_eq!(built_file_count(&prebuilt), built_file_count(&host) + 3);
    assert!(repository.path("dist/claude-code/bin/tightship").is_file());
    for binary in BINARIES {
        let copied = fs::read_to_string(repository.path(&format!("dist/claude-code/bin/{binary}")))
            .expect("the copied binary");
        assert_eq!(copied, format!("prebuilt {binary}\n"));
    }
}

#[test]
fn a_missing_binary_fails_and_leaves_no_plugin() {
    let repository = release_repository("0.1.0");
    fs::remove_file(repository.path("binaries/linux-arm64/tightship")).expect("the binary");

    assert_fails_with(
        &run(&repository, &["build", "--binaries", "binaries"]),
        "error: binaries/linux-arm64/tightship is missing; --binaries needs macos, linux-x64, linux-arm64, and windows-x64",
    );
    assert!(!repository.path("dist/claude-code").exists());
}

#[test]
fn a_package_holds_the_plugin_with_fixed_modes_and_its_catalog() {
    let repository = release_repository("1.0.0");
    assert_succeeds(&run(&repository, &["build", "--binaries", "binaries"]));

    let output = run(&repository, &["package", "v1.0.0"]);
    assert_succeeds(&output);
    let bytes = fs::read(repository.path("dist/tightship-claude-code.zip")).expect("the zip");
    let sha256: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!("sha256: {sha256}\n")));

    let mut archive = ZipArchive::new(std::io::Cursor::new(bytes)).expect("a readable zip");
    let mut modes = BTreeMap::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index).expect("an entry");
        let mode = entry.unix_mode().expect("a unix mode") & 0o777;
        modes.insert(entry.name().expect("a UTF-8 name").into_owned(), mode);
    }
    let mut expected = plugin_files(&repository);
    expected.retain(|file| file != ".claude-plugin/marketplace.json");
    assert_eq!(modes.keys().cloned().collect::<Vec<_>>(), expected);
    let executables: Vec<&str> = modes
        .iter()
        .filter(|(_, mode)| **mode == 0o755)
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(
        executables,
        [
            "bin/linux-arm64/tightship",
            "bin/linux-x64/tightship",
            "bin/macos/tightship",
            "bin/tightship",
            "bin/windows-x64/tightship.exe",
        ]
    );
    assert!(modes.values().all(|mode| *mode == 0o755 || *mode == 0o644));

    let catalog_text =
        fs::read_to_string(repository.path("dist/marketplace.json")).expect("a catalog");
    let catalog: serde_json::Value = serde_json::from_str(&catalog_text).expect("valid JSON");
    assert_eq!(
        catalog["plugins"][0]["source"],
        serde_json::json!({
            "source": "archive",
            "url": "https://github.com/ATNexusLab/tightship/releases/download/v1.0.0/tightship-claude-code.zip",
            "sha256": sha256,
        })
    );
    assert_eq!(
        catalog["allowCrossMarketplaceDependenciesOn"],
        serde_json::json!(["ponytail", "i-have-adhd", "humanizer"])
    );
}

#[test]
fn a_tag_other_than_the_version_fails_and_writes_nothing() {
    let repository = release_repository("1.0.0");
    assert_succeeds(&run(&repository, &["build", "--binaries", "binaries"]));

    assert_fails_with(
        &run(&repository, &["package", "v1.0.1"]),
        "error: the tag v1.0.1 does not match the version 1.0.0; tag v1.0.0",
    );
    assert!(!repository.path("dist/tightship-claude-code.zip").exists());
    assert!(!repository.path("dist/marketplace.json").exists());
}

#[test]
fn a_plugin_without_every_platform_fails() {
    let repository = release_repository("1.0.0");
    assert_succeeds(&run(&repository, &["build", "--binaries", "binaries"]));
    fs::remove_dir_all(repository.path("dist/claude-code/bin/linux-arm64")).expect("the platform");

    assert_fails_with(
        &run(&repository, &["package", "v1.0.0"]),
        "error: dist/claude-code/bin/ has no linux-arm64; run cargo xtask build --binaries <folder> first",
    );
}

#[test]
fn a_package_without_a_tag_prints_the_usage() {
    let repository = release_repository("1.0.0");

    assert_fails_with(
        &repository.run("package"),
        "error: usage: cargo xtask build [--binaries <folder>] | package <tag> | scan",
    );
}
