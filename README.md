# ![tightship logo](docs/assets/logo.svg) tightship

[![License: MIT](https://img.shields.io/badge/license-MIT-0E2440)](LICENSE)
[![Rust](https://github.com/ATNexusLab/tightship/actions/workflows/rust.yml/badge.svg)](https://github.com/ATNexusLab/tightship/actions/workflows/rust.yml)
[![Scan](https://github.com/ATNexusLab/tightship/actions/workflows/scan.yml/badge.svg)](https://github.com/ATNexusLab/tightship/actions/workflows/scan.yml)
[![Claude Code 2.1.295+](https://img.shields.io/badge/Claude%20Code-2.1.295%2B-E0A63A)](docs/install/claude-code/README.md)
[![Status: pre-release](https://img.shields.io/badge/status-pre--release-4A5B73)](docs/roadmap/README.md)

**A disciplined workflow for AI coding agents, packaged as a plugin.**

tightship gives your agent a working contract and a command for each step from requirement to pull
request. It adds a skeptical verifier and a memory that follows you across sessions and machines. You
install it with the harness's own plugin mechanism and turn each optional part on or off.

> [!NOTE]
> tightship is in pre-release. Milestone M1, the Claude Code plugin, is being finished, and no stable release
> exists yet. See the [roadmap](docs/roadmap/README.md).

## Why

A coding agent left to its defaults writes code fast and forgets everything between sessions. It skips the
spec, and it calls the work done before anyone has checked it. Teams fix this with process: requirements
with IDs, an issue for every change, gates that must pass, and a review before the commit. tightship
writes that process down once and loads it into the agent, so a solo developer gets the same discipline a
team would hold them to.

## What you get

| Piece | What it does |
| --- | --- |
| Contract | Rules loaded into every session: how the agent plans, writes code, tests, handles git and secrets, and what counts as done |
| 10 commands | One per step of the loop: `/requirements`, `/bootstrap`, `/sprint`, `/spec`, `/epic`, `/audit`, `/closeout`, `/pr-review`, `/handoff`, `/sync` |
| 13 skills | Loaded on demand: API, database, frontend and hexagonal architecture, requirements engineering, spec writing, security audit, testing, performance, technical writing, and more |
| Verifier | A read-only subagent that tries to disprove a fix, a finding, or a "done" claim before you trust it |
| Memory | An optional vault of curated notes in a git repository you own, shared across sessions, machines, and harnesses |
| Status line | An optional status line, written to your settings only after you agree |

Three companion plugins install with it: [ponytail](https://github.com/DietrichGebert/ponytail) pushes the
agent toward the simplest code that works, [i-have-adhd](https://github.com/ayghri/i-have-adhd) shapes
answers so you can act on them, and [humanizer](https://github.com/blader/humanizer) rewrites prose so it
does not read as generated.

## How a feature moves

```text
/requirements → /bootstrap → /sprint plan → grilling → /spec → /epic
      → plan → tests → code → gates green → /audit → /closeout → commit → PR
```

Each requirement has an ID. A spec cites the IDs it covers, an epic and its sub-issues come from the spec,
and every commit and pull request points back to its issue. Nothing reaches a commit until the gates pass
with zero errors and zero warnings and you have read the change.

## Quickstart

You need Claude Code 2.1.295 or later. On Windows you also need Git for Windows, since the plugin's hooks
run under its `sh`.

```sh
claude plugin marketplace add DietrichGebert/ponytail
claude plugin marketplace add ayghri/i-have-adhd
claude plugin marketplace add blader/humanizer
claude plugin marketplace add https://github.com/ATNexusLab/tightship/releases/latest/download/marketplace.json
claude plugin install tightship@tightship
```

The next session asks which options to turn on. The [install guide](docs/install/claude-code/README.md)
covers the options, updates, uninstalling, and moving from the pre-plugin install.

## Harness support

| Harness | Status | Milestone |
| --- | --- | --- |
| Claude Code | In progress | M1 |
| Cursor | Planned | M2 |
| Antigravity CLI | Planned | M3 |
| Codex | Planned | M4 |

Every piece is written once in `content/`, which names no harness. A Rust build turns it into each
harness's native plugin format ([ADR 0001](docs/decisions/0001-rust-build-and-tokenized-canonical-content.md)).
The plugin works on macOS, Linux, and Windows.

## Build from source

You need `rustup`, which installs the pinned toolchain on the first `cargo` call, and gitleaks 8.30.1 on
the `PATH`.

```sh
git clone https://github.com/ATNexusLab/tightship.git
cd tightship
cargo xtask build
claude --plugin-dir dist/claude-code
```

The build expects a `.identity-terms` file at the root. [CONTRIBUTING.md](CONTRIBUTING.md) explains it and
lists every gate.

## Documentation

| Where | What |
| --- | --- |
| [Install guide](docs/install/claude-code/README.md) | Installing, options, updates, uninstalling, migration |
| [Requirements](docs/requirements/README.md) | The versioned SRS, with every requirement and its acceptance criteria |
| [Architecture](docs/architecture/README.md) | The system map, in arc42 sections with C4 diagrams |
| [Decisions](docs/decisions/README.md) | The architecture decision records |
| [Roadmap](docs/roadmap/README.md) | Milestones, epics, and requirement coverage |

## Contributing

Issues and pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull
request, and report vulnerabilities as [SECURITY.md](SECURITY.md) describes, not in a public issue.
Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

[MIT](LICENSE) © ATNexusLab
