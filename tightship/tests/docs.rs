#[allow(dead_code)]
mod common;

use std::path::PathBuf;
use std::process::Output;

use common::{Sandbox, assert_succeeds, commit, git, run, stderr_of, stdout_of};

fn repository(sandbox: &Sandbox) -> PathBuf {
    let root = sandbox.path("project");
    sandbox.write(
        "project/tool/src/vault/search.rs",
        "pub fn search_notes() {}\n",
    );
    sandbox.write(
        "project/tool/src/main.rs",
        "enum Command { Search, MarketplaceAutoUpdate }\nconst FILE: &str = \"options.toml\";\n",
    );
    git(sandbox, &root, &["init", "--quiet"]);
    root
}

fn docs(sandbox: &Sandbox, arguments: &[&str]) -> Output {
    let mut command = sandbox.tightship(arguments);
    command.current_dir(sandbox.path("project"));
    run(&mut command)
}

fn assert_findings(output: &Output, expected: &[&str]) {
    let stdout = stdout_of(output);
    assert_eq!(output.status.code(), Some(1), "stdout: {stdout}");
    assert_eq!(stderr_of(output), "", "the findings go to stdout only");
    let found: Vec<&str> = stdout.lines().filter(|line| line.contains(" — ")).collect();
    assert_eq!(found, expected, "stdout: {stdout}");
}

#[test]
fn passes_docs_that_match_the_code() {
    let sandbox = Sandbox::new();
    repository(&sandbox);
    sandbox.write(
        "project/docs/system/vault.md",
        "---\ncovers: [tool/src/vault/**, tool/src/main.rs]\n---\n\n# Vault\n\n`tool search` and `tool marketplace-auto-update` run `search_notes()` from \
         `tool/src/vault/search.rs`, which reads `options.toml`. See [the index](README.md#docs) and \
         [the site](https://example.test).\n\n```sh\ntool gone TODO\n```\n",
    );
    sandbox.write("project/docs/system/README.md", "# Docs\n\nThe index.\n");
    let output = docs(&sandbox, &["docs", "check"]);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "2 docs, 0 findings\n");
}

#[test]
fn reports_links_to_missing_files_and_anchors() {
    let sandbox = Sandbox::new();
    repository(&sandbox);
    sandbox.write(
        "project/docs/guide.md",
        "# Guide\n\n[a](gone.md) and [b](other.md#nowhere) and [c](#guide).\n",
    );
    sandbox.write("project/docs/other.md", "# Other\n");
    assert_findings(
        &docs(&sandbox, &["docs", "check"]),
        &[
            "docs/guide.md:3 — link to gone.md: no such file",
            "docs/guide.md:3 — link to other.md#nowhere: no anchor #nowhere",
        ],
    );
}

#[test]
fn reports_references_the_covered_code_dropped() {
    let sandbox = Sandbox::new();
    repository(&sandbox);
    sandbox.write(
        "project/docs/system/vault.md",
        "---\ncovers:\n  - tool/src/vault/**\n  - tool/src/gone/**\n---\n\n# Vault\n\n`tool/src/vault/lint.rs` holds `lint_notes`.\n\n`tool vault prune` runs it.\n",
    );
    assert_findings(
        &docs(&sandbox, &["docs", "check"]),
        &[
            "docs/system/vault.md:4 — covers `tool/src/gone/**`, which matches no file",
            "docs/system/vault.md:9 — `lint_notes` is in none of the code the doc covers",
            "docs/system/vault.md:9 — `tool/src/vault/lint.rs` names no file in the repository",
            "docs/system/vault.md:11 — `tool vault prune`: `prune` is in none of the code under `tool/`",
        ],
    );
}

#[test]
fn reports_system_docs_without_covers_and_markers_outside_work() {
    let sandbox = Sandbox::new();
    repository(&sandbox);
    sandbox.write("project/docs/system/README.md", "# System\n");
    sandbox.write("project/docs/system/build.md", "# Build\n\nTBD.\n");
    sandbox.write(
        "project/docs/work/sprint.md",
        "# Sprint\n\nTODO later, and [x](gone.md).\n",
    );
    sandbox.write(
        "project/client-docs/start.md",
        "# Start\n\nFIXME, see `TODO`.\n",
    );
    assert_findings(
        &docs(&sandbox, &["docs", "check"]),
        &[
            "client-docs/start.md:3 — FIXME marker left in the doc",
            "docs/system/build.md:1 — covers no code; list the paths it describes under `covers` in its front matter",
            "docs/system/build.md:3 — TBD marker left in the doc",
            "docs/work/sprint.md:3 — link to gone.md: no such file",
        ],
    );
}

#[test]
fn names_the_docs_a_diff_affects() {
    let sandbox = Sandbox::new();
    let root = repository(&sandbox);
    sandbox.write(
        "project/docs/system/vault.md",
        "---\ncovers: [tool/src/vault/**]\n---\n\n# Vault\n",
    );
    sandbox.write(
        "project/docs/system/main.md",
        "---\ncovers: [tool/src/main.rs]\n---\n\n# Main\n",
    );
    commit(&sandbox, &root, "start");

    let output = docs(&sandbox, &["docs", "affected"]);
    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "no doc covers the changed files\n");

    sandbox.write(
        "project/tool/src/vault/search.rs",
        "pub fn find_notes() {}\n",
    );
    sandbox.write("project/tool/src/vault/new.rs", "pub fn write() {}\n");
    let output = docs(&sandbox, &["docs", "affected"]);
    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "docs/system/vault.md ← tool/src/vault/new.rs, tool/src/vault/search.rs\n"
    );

    commit(&sandbox, &root, "change");
    let output = docs(&sandbox, &["docs", "affected", "--base", "HEAD~1"]);
    assert_succeeds(&output);
    assert_eq!(
        stdout_of(&output),
        "docs/system/vault.md ← tool/src/vault/new.rs, tool/src/vault/search.rs\n"
    );
}
