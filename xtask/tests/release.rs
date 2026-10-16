mod common;

use std::collections::BTreeMap;
use std::fs;
use std::process::Output;

use sha2::{Digest, Sha256};
use zip::ZipArchive;

use common::{Repository, assert_fails_with, assert_succeeds, harness_files};

const BINARIES: [&str; 4] = [
    "macos/tightship",
    "linux-x64/tightship",
    "linux-arm64/tightship",
    "windows-x64/tightship.exe",
];

fn release_repository(version: &str) -> Repository {
    let antigravity = harness_files("antigravity-cli");
    let cursor = harness_files("cursor");
    let mut files: Vec<(&str, String)> = antigravity
        .iter()
        .chain(cursor.iter())
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    files.extend([
        (
            "Cargo.toml",
            format!("[workspace.package]\nversion = \"{version}\"\n"),
        ),
        (
            "content/commands/epic.md",
            "+++\nname = \"epic\"\ndescription = \"A piece\"\n+++\nOpen the epic.\n".to_owned(),
        ),
    ]);
    let repository = Repository::with(&files);
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

fn plugin_files(repository: &Repository, harness: &str) -> Vec<String> {
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
    walk(&repository.path(&format!("dist/{harness}")), "", &mut files);
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
    assert!(
        repository
            .path("dist/antigravity-cli/bin/tightship")
            .is_file()
    );
    assert!(
        repository
            .path("dist/antigravity-cli/bin/tightship.cmd")
            .is_file()
    );
    for binary in BINARIES {
        let copied = fs::read_to_string(repository.path(&format!("dist/claude-code/bin/{binary}")))
            .expect("the copied binary");
        assert_eq!(copied, format!("prebuilt {binary}\n"));

        let agy_copied =
            fs::read_to_string(repository.path(&format!("dist/antigravity-cli/bin/{binary}")))
                .expect("the copied binary for antigravity-cli");
        assert_eq!(agy_copied, format!("prebuilt {binary}\n"));
    }
}

#[test]
fn a_missing_binary_fails_and_leaves_no_plugin() {
    let repository = release_repository("0.1.0");
    fs::remove_file(repository.path("binaries/linux-arm64/tightship")).expect("the binary");
    repository.write("dist/notes/kept.md", "not the build's\n");

    assert_fails_with(
        &run(&repository, &["build", "--binaries", "binaries"]),
        "error: binaries/linux-arm64/tightship is missing; --binaries needs macos, linux-x64, linux-arm64, and windows-x64",
    );
    assert!(!repository.path("dist/claude-code").exists());
    assert!(!repository.path("dist/antigravity-cli").exists());
    assert!(repository.path("dist/notes/kept.md").is_file());
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
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("sha256 (claude-code): {sha256}"))
    );

    let mut archive = ZipArchive::new(std::io::Cursor::new(bytes)).expect("a readable zip");
    let mut modes = BTreeMap::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index).expect("an entry");
        let mode = entry.unix_mode().expect("a unix mode") & 0o777;
        modes.insert(entry.name().expect("a UTF-8 name").into_owned(), mode);
    }
    let mut expected = plugin_files(&repository, "claude-code");
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

    let agy_bytes =
        fs::read(repository.path("dist/tightship-antigravity-cli.zip")).expect("the agy zip");
    let agy_sha256: String = Sha256::digest(&agy_bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("sha256 (antigravity-cli): {agy_sha256}"))
    );

    let mut agy_archive = ZipArchive::new(std::io::Cursor::new(agy_bytes)).expect("a readable zip");
    let mut agy_modes = BTreeMap::new();
    for index in 0..agy_archive.len() {
        let entry = agy_archive.by_index(index).expect("an entry");
        let mode = entry.unix_mode().expect("a unix mode") & 0o777;
        agy_modes.insert(entry.name().expect("a UTF-8 name").into_owned(), mode);
    }
    let agy_expected = plugin_files(&repository, "antigravity-cli");
    assert_eq!(agy_modes.keys().cloned().collect::<Vec<_>>(), agy_expected);
    let agy_executables: Vec<&str> = agy_modes
        .iter()
        .filter(|(_, mode)| **mode == 0o755)
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(
        agy_executables,
        [
            "bin/linux-arm64/tightship",
            "bin/linux-x64/tightship",
            "bin/macos/tightship",
            "bin/tightship",
            "bin/tightship.cmd",
            "bin/windows-x64/tightship.exe",
        ]
    );
    assert!(
        agy_modes
            .values()
            .all(|mode| *mode == 0o755 || *mode == 0o644)
    );

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
    for (installer, tag_line, sha256_line) in [
        (
            "install.sh",
            "RELEASE_TAG=\"v1.0.0\"",
            format!("ANTIGRAVITY_CLI_SHA256=\"{agy_sha256}\""),
        ),
        (
            "install.ps1",
            "$ReleaseTag = \"v1.0.0\"",
            format!("$AntigravityCliSha256 = \"{agy_sha256}\""),
        ),
    ] {
        let released = fs::read_to_string(repository.path(&format!("dist/{installer}")))
            .expect("the released installer");
        assert!(released.lines().any(|line| line == tag_line), "{installer}");
        assert!(
            released.lines().any(|line| line == sha256_line),
            "{installer}"
        );
    }
    assert_eq!(
        catalog["allowCrossMarketplaceDependenciesOn"],
        serde_json::json!(["ponytail", "i-have-adhd", "humanizer"])
    );

    let cursor_bytes =
        fs::read(repository.path("dist/tightship-cursor.zip")).expect("the cursor zip");
    let cursor_sha256: String = Sha256::digest(&cursor_bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("sha256 (cursor): {cursor_sha256}"))
    );

    let mut cursor_archive =
        ZipArchive::new(std::io::Cursor::new(cursor_bytes)).expect("a readable zip");
    let mut cursor_modes = BTreeMap::new();
    for index in 0..cursor_archive.len() {
        let entry = cursor_archive.by_index(index).expect("an entry");
        let mode = entry.unix_mode().expect("a unix mode") & 0o777;
        cursor_modes.insert(entry.name().expect("a UTF-8 name").into_owned(), mode);
    }
    let mut cursor_expected = plugin_files(&repository, "cursor");
    cursor_expected.retain(|file| file != ".cursor-plugin/marketplace.json");
    assert_eq!(cursor_modes.keys().cloned().collect::<Vec<_>>(), cursor_expected);
    assert!(
        cursor_modes
            .values()
            .all(|mode| *mode == 0o755 || *mode == 0o644)
    );

    let cursor_catalog_text =
        fs::read_to_string(repository.path("dist/cursor-marketplace.json")).expect("cursor catalog");
    let cursor_catalog: serde_json::Value =
        serde_json::from_str(&cursor_catalog_text).expect("valid JSON");
    assert_eq!(
        cursor_catalog["plugins"][0]["source"],
        serde_json::json!({
            "source": "archive",
            "url": "https://github.com/ATNexusLab/tightship/releases/download/v1.0.0/tightship-cursor.zip",
            "sha256": cursor_sha256,
        })
    );
    assert_eq!(
        cursor_catalog["allowCrossMarketplaceDependenciesOn"],
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
    assert!(
        !repository
            .path("dist/tightship-antigravity-cli.zip")
            .exists()
    );
    assert!(!repository.path("dist/marketplace.json").exists());
    assert!(!repository.path("dist/tightship-cursor.zip").exists());
    assert!(!repository.path("dist/cursor-marketplace.json").exists());
    let installer =
        fs::read_to_string(repository.path("dist/install.sh")).expect("the built installer");
    assert!(installer.lines().any(|line| line == "RELEASE_TAG=\"\""));
}

#[test]
fn a_plugin_without_every_platform_fails() {
    let repository = release_repository("1.0.0");
    assert_succeeds(&run(&repository, &["build", "--binaries", "binaries"]));
    fs::remove_file(repository.path("dist/claude-code/bin/linux-arm64/tightship"))
        .expect("the binary");

    assert_fails_with(
        &run(&repository, &["package", "v1.0.0"]),
        "error: dist/claude-code/bin/linux-arm64/tightship is missing; run cargo xtask build --binaries <folder> first",
    );
}

#[test]
fn antigravity_cli_missing_a_platform_binary_fails() {
    let repository = release_repository("1.0.2");
    assert_succeeds(&run(&repository, &["build", "--binaries", "binaries"]));
    fs::remove_file(repository.path("dist/antigravity-cli/bin/macos/tightship"))
        .expect("the binary");

    assert_fails_with(
        &run(&repository, &["package", "v1.0.2"]),
        "error: dist/antigravity-cli/bin/macos/tightship is missing; run cargo xtask build --binaries <folder> first",
    );
    assert!(
        !repository
            .path("dist/tightship-antigravity-cli.zip")
            .exists()
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
