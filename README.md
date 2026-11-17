# [![tightship: ship it tied down](docs/product/design/brand/banner.svg)](https://github.com/ATNexusLab/tightship)

English · [Português](README.pt-BR.md)

[![License: MIT](https://img.shields.io/badge/license-MIT-5C6670?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/github/actions/workflow/status/ATNexusLab/tightship/rust.yml?style=flat-square&label=rust)](https://github.com/ATNexusLab/tightship/actions/workflows/rust.yml)
[![Scan](https://img.shields.io/github/actions/workflow/status/ATNexusLab/tightship/scan.yml?style=flat-square&label=scan)](https://github.com/ATNexusLab/tightship/actions/workflows/scan.yml)
[![Claude Code 2.1.295+](https://img.shields.io/badge/Claude%20Code-2.1.295%2B-FF5A1F?style=flat-square)](docs/guides/install/claude-code/README.md)
[![Cursor 3.25.6+](https://img.shields.io/badge/Cursor-3.25.6%2B-121212?style=flat-square)](docs/guides/install/cursor/README.md)
[![Antigravity CLI 1.4.0+](https://img.shields.io/badge/Antigravity%20CLI-1.4.0%2B-4285F4?style=flat-square)](docs/guides/install/antigravity-cli/README.md)
[![Release](https://img.shields.io/github/v/release/ATNexusLab/tightship?include_prereleases&style=flat-square&label=release&color=121212)](https://github.com/ATNexusLab/tightship/releases)

**A workflow for AI coding agents, from requirement to release.**

tightship loads a full engineering process into your coding agent. Every change gets an issue and a spec.
Gates must come back green, and a verifier has to fail to disprove the claim of done. Only then does the
change ship. It installs as a native plugin and keeps memory across sessions and machines.

## Why

Left to its defaults, a coding agent writes fast and forgets everything between sessions. It skips the spec,
and it calls work done before anyone checks it.

Teams fix that with process: requirements with IDs, an issue for every change, gates that must pass, a
review before the commit. tightship writes that process down once and loads it into the agent, so working
alone gets the same rigor a team would demand.

## What ships

| Piece | What it does |
| --- | --- |
| Contract | Rules in every session: how the agent plans, writes code, tests, handles git and secrets, and what counts as done |
| 10 commands | One per step: `/requirements`, `/bootstrap`, `/sprint`, `/spec`, `/epic`, `/audit`, `/closeout`, `/pr-review`, `/handoff`, `/sync` |
| 14 skills | Loaded on demand: API, database, frontend, and hexagonal architecture, requirements engineering, spec writing, security audit, testing, performance, technical writing, and more |
| Verifier | A read-only subagent that tries to disprove a fix, a finding, or a "done" before you trust it |
| Memory | Optional. Curated notes in a git repository you own, shared across sessions, machines, and harnesses |
| Status line | Optional. Written to your settings only while you have it turned on |
| Auto-update | On unless you turn it off. Each new release installs by itself, and the next session says so |

## How a change moves

```text
/requirements → /bootstrap → /sprint plan → /spec → /epic
      →  grilling  → tests → code → gates green → /audit → /closeout → commit → PR
```

Every requirement has an ID. A spec cites the IDs it covers. An epic and its sub-issues come from the
spec. Every commit and pull request points back to its issue. Nothing is committed until the gates pass
with zero errors and zero warnings and you have read the change.

## Quickstart

Run the installer for Claude Code (2.1.295+), Cursor (3.25.6+, `agent` 2026.10.14+), Antigravity CLI
(1.4.0+), or all three:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | sh
```

On Windows in PowerShell (5.1 or 7):

```powershell
irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1 | iex
```

From `cmd`:

```bat
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1 | iex"
```

The installer detects your CLIs, asks where to install and which options to turn on, and writes nothing
until you confirm the summary. The install guides
([Claude Code](docs/guides/install/claude-code/README.md), [Cursor](docs/guides/install/cursor/README.md),
[Antigravity CLI](docs/guides/install/antigravity-cli/README.md)) cover options, updates, uninstalling,
and installing by hand.

### Native install, without the installer

Each harness can also install tightship on its own. The installer's questions are then skipped: the
options and the status line are set as each guide's manual installation shows.

Claude Code, from its plugin catalog ([guide](docs/guides/install/claude-code/install.md#manual-installation)):

```sh
claude plugin marketplace add ATNexusLab/tightship
claude plugin install tightship@tightship
```

Antigravity CLI, from the release zip ([guide](docs/guides/install/antigravity-cli/install.md#manual-installation),
with the Windows step):

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/tightship-antigravity-cli.zip -o /tmp/tightship.zip
unzip -q -o /tmp/tightship.zip -d /tmp/tightship
agy plugin install /tmp/tightship
rm -f ~/.gemini/config/plugins/tightship/hooks.windows.json
```

Cursor, as a local plugin ([guide](docs/guides/install/cursor/install.md#manual-installation)); reload the editor
window afterwards, and it shows under Customize → Plugins:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/tightship-cursor.zip -o /tmp/tightship.zip
mkdir -p ~/.cursor/plugins/local/tightship
unzip -q -o /tmp/tightship.zip -d ~/.cursor/plugins/local/tightship
```

## Harnesses

| Harness | Status | Milestone |
| --- | --- | --- |
| Claude Code | Available | M1 |
| Cursor | Available | M2 |
| Antigravity CLI | Available | M3 |
| Codex | Planned | M4 |

Every piece is written once in `content/`, which names no harness. A Rust build turns it into each
harness's native plugin format ([ADR 0001](docs/decisions/0001-rust-build-and-tokenized-canonical-content.md)).
It runs on macOS, Linux, and Windows.

## Build from source

You need `rustup`, which installs the pinned toolchain on the first `cargo` call, and gitleaks 8.30.1 on
the `PATH`.

```sh
git clone https://github.com/ATNexusLab/tightship.git
cd tightship
cargo xtask build
claude --plugin-dir dist/claude-code
agent --plugin-dir dist/cursor
```

The build needs a `.identity-terms` file at the root. [CONTRIBUTING.md](CONTRIBUTING.md) explains it and
lists every gate.

## Docs

| Where | What |
| --- | --- |
| [Install guide (Claude Code)](docs/guides/install/claude-code/README.md) | Installing, options, updates, uninstalling, migration |
| [Install guide (Cursor)](docs/guides/install/cursor/README.md) | Installing, options, updates, uninstalling |
| [Install guide (Antigravity CLI)](docs/guides/install/antigravity-cli/README.md) | Installing, options, updates, uninstalling, migration |
| [Requirements](docs/product/requirements/README.md) | The versioned SRS: every requirement and its acceptance criteria |
| [Architecture](docs/system/architecture/README.md) | The system map, in arc42 sections with C4 diagrams |
| [Decisions](docs/decisions/README.md) | The architecture decision records |
| [Roadmap](docs/work/roadmap/README.md) | Milestones, epics, and requirement coverage |
| [Design](docs/product/design/README.md) | The brand, its voice, and its tokens |

## Contributing

Issues and pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first. Report
vulnerabilities as [SECURITY.md](SECURITY.md) says, never in a public issue. Everyone follows the
[Code of Conduct](CODE_OF_CONDUCT.md).

## License

[MIT](LICENSE) © ATNexusLab
