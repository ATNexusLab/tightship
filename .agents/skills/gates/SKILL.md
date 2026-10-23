---
name: gates
description: Use before a commit or a pull request, or when CI fails — every gate of this repository as its exact command, cheapest first, its prerequisites, and what CI runs.
---

# Gates

## Prerequisites

- The toolchain is pinned in `rust-toolchain.toml`; `rustup` installs it on the first `cargo` call.
- gitleaks 8.30.1 on the `PATH`, for `cargo xtask scan` and `cargo test --workspace`.
- PowerShell 7 (`pwsh`) on the `PATH`, for the installers' tests; the GitHub runners ship it.
- `.identity-terms` at the root of the clone, ignored by git: one line per name, email address, user name,
  and handle of the maintainer.

## Gates, cheapest first

| Gate | Command | Covers |
| --- | --- | --- |
| Static analysis | `npx markdownlint-cli2@0.23.3` | Every `.md` git does not ignore, configured in `.markdownlint-cli2.jsonc` |
| Static analysis | `cargo clippy --workspace --all-targets -- -D warnings` | The Rust workspace |
| Static analysis | `cargo xtask scan` | gitleaks over every commit of every branch, and no tracked vault note |
| Type checking | `cargo check --workspace --all-targets` | The Rust workspace |
| Formatting | `cargo fmt --all --check` | rustfmt defaults |
| Build | `uvx zensical==0.0.70 build` | The docs site into `site/`, configured in `zensical.toml` |
| Build | `cargo xtask build` | The plugins, with the `tightship` binary for this machine, into `dist/claude-code/`, `dist/cursor/`, and `dist/antigravity-cli/`, and the installers into `dist/install.sh` and `dist/install.ps1` |
| Tests | `cargo test --workspace` | The build's, the package's, the scan's, the installers', and the binary's scenarios, over fixtures in a temporary folder, each installer scenario against both `install.sh` and `install.ps1` |

## Run

| What | Command |
| --- | --- |
| The docs | `uvx zensical==0.0.70 serve --open`, at `http://localhost:8000` |
| A built plugin | The `session-check` skill |

## CI

At each pull request into `develop` or `main`, `.github/workflows/` runs only the gates its files call for:
`rust.yml` the Rust gates on `ubuntu-latest`, `macos-latest`, and `windows-latest` (in Git Bash) when the
workspace, `content/`, `harnesses/`, or `installer/` changes; `docs.yml` markdownlint and the docs build on
Linux when Markdown or the docs site changes; `scan.yml` the scan on Linux every time.

`rust.yml` writes `.identity-terms` from the repository secret `IDENTITY_TERMS`, so a change to the file is
copied there with `gh secret set IDENTITY_TERMS < .identity-terms`. It clears `USER`: the build counts the
system's user name as an identity term, and a runner's, `runner`, is a word the content uses.
