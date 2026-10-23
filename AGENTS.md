# AGENTS.md

The workflow, packaged as a native plugin for Claude Code, Cursor, Antigravity CLI, and Codex, built from
one canonical source in this repository. What it must do: [the SRS](docs/requirements/README.md).

## Stack

Rust in one Cargo workspace, with the canonical content in Markdown and TOML
([ADR 0001](docs/decisions/0001-rust-build-and-tokenized-canonical-content.md)):

- The `xtask` crate: the build, run as `cargo xtask build`, the release package, run as `cargo xtask package`,
  and the repository scan, run as `cargo xtask scan`.
  The toolchain is pinned in `rust-toolchain.toml`, and `rustup` installs it on the first `cargo` call.
- The `tightship` crate: the binary the plugin ships, which `cargo xtask build` compiles for the machine that
  builds into `dist/claude-code/bin/` and `dist/cursor/bin/`.
- gitleaks 8.30.1 on the `PATH`, for `cargo xtask scan`.
- PowerShell 7 (`pwsh`) on the `PATH`, for the installers' tests; the GitHub runners ship it.
- `.identity-terms` at the root of each clone that builds, ignored by git: one line per name, email address,
  user name, and handle of the maintainer.
- Markdown, linted by `markdownlint-cli2`, configured in `.markdownlint-cli2.jsonc`.
- Diagrams in draw.io, as `.drawio.svg`.
- Brand assets as SVG written by hand, in `docs/design/brand/`; the wordmark's letters are outlines.
- The local docs site on Zensical, configured in `zensical.toml`, run through `uvx`.

## Programming languages & scripts

Rust for application code, for the build, and for every new script, except the two installers, `install.sh`
in POSIX `sh` and `install.ps1` in PowerShell, and the Windows launcher `tightship/launcher.cmd`, which run
before any binary is on the machine
([ADR 0001](docs/decisions/0001-rust-build-and-tokenized-canonical-content.md)).

## Conventions

- Prose in `docs/` wraps at about 105 columns; the lint stops a line past 110, outside tables.
- `<a>` is the only HTML allowed in Markdown: the anchors of requirement IDs.

## Commands

| Gate | Command | Status |
| --- | --- | --- |
| Static analysis | `npx markdownlint-cli2@0.23.3` | Runs on every `.md` git does not ignore |
| Static analysis | `cargo clippy --workspace --all-targets -- -D warnings` | Lints the Rust workspace |
| Static analysis | `cargo xtask scan` | gitleaks over every commit of every branch, and no tracked vault note |
| Type checking | `cargo check --workspace --all-targets` | Type-checks the Rust workspace |
| Formatting | `cargo fmt --all --check` | rustfmt defaults |
| Build | `uvx zensical==0.0.70 build` | Builds the docs site into `site/` |
| Build | `cargo xtask build` | Builds the plugins, with the `tightship` binary for this machine, into `dist/claude-code/`, `dist/cursor/`, and `dist/antigravity-cli/`, and the installers into `dist/install.sh` and `dist/install.ps1` |
| Tests | `cargo test --workspace` | The build's, the package's, the scan's, the installer's, and the binary's scenarios, over fixtures in a temporary folder, each installer scenario against both `install.sh` and `install.ps1`; needs gitleaks 8.30.1 and `pwsh` |
| Run | `uvx zensical==0.0.70 serve --open` | Serves the docs at `http://localhost:8000` |
| Run | `claude --plugin-dir dist/claude-code` | Loads the built Claude Code plugin in a session |
| Run | `agent --plugin-dir dist/cursor` | Loads the built Cursor plugin in an interactive CLI session |
| Run | `claude plugin marketplace add ./dist/claude-code`, then `claude plugin install tightship@tightship` | Installs the built plugin with its companions, once the catalogs `DietrichGebert/ponytail`, `ayghri/i-have-adhd`, and `blader/humanizer` are added |
| Run | `agy plugin validate dist/antigravity-cli --strict` | Validates the built Antigravity CLI plugin |
| Release | `cargo xtask build --binaries <folder>` | The plugins with the four prebuilt binaries of `<folder>`, as `macos/tightship`, `linux-x64/tightship`, `linux-arm64/tightship`, and `windows-x64/tightship.exe`, nothing compiled |
| Release | `cargo xtask package <tag>` | Packs the plugins into `dist/tightship-claude-code.zip`, `dist/tightship-antigravity-cli.zip`, and `dist/tightship-cursor.zip`, writes `dist/marketplace.json` and `dist/cursor-marketplace.json`, and `dist/install.sh` and `dist/install.ps1` with the tag and the Antigravity CLI and Cursor zips' `sha256`; the tag is `v` and the built version |

At each pull request into `develop` or `main`, `.github/workflows/` runs only the gates its files call for:
`rust.yml` the Rust gates on `ubuntu-latest`, `macos-latest`, and `windows-latest` (in Git Bash) when the
workspace, `content/`, `harnesses/`, or `installer/` changes; `docs.yml` markdownlint and the docs build on
Linux when Markdown or the docs site changes; `scan.yml` the scan on Linux every time. `rust.yml` writes
`.identity-terms` from the repository secret `IDENTITY_TERMS`, so a change to the file is copied there with
`gh secret set IDENTITY_TERMS < .identity-terms`. It clears `USER`: the build counts the system's user name as
an identity term, and a runner's, `runner`, is a word the content uses.

A tag `v<version>` pushed on `main` runs `release.yml`: it builds the binary for the four platforms, then
`cargo xtask build --binaries binaries` and `cargo xtask package <tag>`, and publishes the zips, catalog,
and installers as a pre-release. It is the only workflow with `contents: write`, on its pack job.

## Architecture

Decided in [ADR 0001](docs/decisions/0001-rust-build-and-tokenized-canonical-content.md).
The map: [docs/architecture/](docs/architecture/README.md).

- `content/` holds the one authored copy of every piece and names no harness, its paths, or its tools.
- `harnesses/<harness>/` is the only place a harness is named: `harness.toml` holds its data, `plugin/` the
  files copied as they are into its plugin, `launcher-env.sh` and `launcher-env.cmd` the variables its
  launchers set, and `install.sh` and `install.ps1` the steps that install it. `cargo xtask build` joins
  those to the neutral `tightship/launcher.*` and `installer/install.*`. The code keeps only what data
  cannot say, in the renderer `xtask/src/build/<harness>.rs`.
- A reference to another piece, to a harness capability, or to an option is a `{{<kind>:<name>}}` token,
  never a literal.
- `dist/` is generated by `cargo xtask build` and never committed or edited by hand.
- `xtask/src/` is split by command, `build/` and `scan/`; `tightship/src/` by domain, `vault/` and `hook/`,
  with a pure core and the I/O at the edges. Neither crate depends on the other
  ([ADR 0002](docs/decisions/0002-tightship-binary-modules-and-dependencies.md)).

## Branches

- `develop` is the integration branch; every PR targets it.
- `main` receives only releases from `develop`, as a merge commit.
- Neither branch has protection configured on GitHub: both rules are convention.

## Language

English throughout the repository: code, file names, docs, commits, branches, issues
([BR-03](docs/requirements/business-rules.md#br-03)). English prevails. The root `README.md`,
`CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, and every page of `docs/install/` also have a
Brazilian Portuguese `<file>.pt-BR.md` next to them. A change to one of those English files updates its
translation in the same commit.

## Gotchas

- Zensical parses Markdown with Python-Markdown: a list right under a paragraph line, with no blank line
  between, renders as plain text with literal hyphens, while GitHub shows a list. The lint rule MD032
  catches it.
- `install.ps1` and its harness parts fail with `throw "error: …"`, never `exit`: under `irm … | iex`,
  `exit` closes the adopter's terminal. The one `catch` at the top prints the message as a plain line, since pwsh's
  own rendering wraps it across decorated lines, and exits 1 only when the installer runs as a file.
- A session check of a built plugin needs each harness set up. Claude Code under `--plugin-dir` has no
  vault option: export `CLAUDE_PLUGIN_OPTION_VAULT_FOLDER` before `claude`. Cursor's `agent` runs from
  PowerShell with `$env:SHELL = $null`. `agy plugin install` writes into the adopter's own config, and on
  Windows needs `hooks.windows.json` moved over `hooks.json` first, as `install.ps1` does.
- A sentence a command tells the agent to say comes back paraphrased unless the instruction says "word for
  word". A placeholder in it is written `\<scope\>`: MD033 rejects a bare `<scope>` in prose, and one in
  backticks reaches the reply with its backticks.
