# [![tightship: ship it tied down](docs/design/brand/banner.svg)](https://github.com/ATNexusLab/tightship)

[![License: MIT](https://img.shields.io/badge/license-MIT-5C6670?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/github/actions/workflow/status/ATNexusLab/tightship/rust.yml?style=flat-square&label=rust)](https://github.com/ATNexusLab/tightship/actions/workflows/rust.yml)
[![Scan](https://img.shields.io/github/actions/workflow/status/ATNexusLab/tightship/scan.yml?style=flat-square&label=scan)](https://github.com/ATNexusLab/tightship/actions/workflows/scan.yml)
[![Claude Code 2.1.295+](https://img.shields.io/badge/Claude%20Code-2.1.295%2B-FF5A1F?style=flat-square)](docs/install/claude-code/README.md)
[![Status: pre-release](https://img.shields.io/badge/status-pre--release-121212?style=flat-square)](docs/roadmap/README.md)

**A workflow for AI coding agents, from requirement to release.**

tightship loads a full engineering process into your coding agent. Every change gets an issue and a spec.
Gates must come back green, and a verifier has to fail to disprove the claim of done. Only then does the
change ship. It installs as a native plugin and keeps memory across sessions and machines.

> [!NOTE]
> Pre-release. Milestone M1, the Claude Code plugin, is being finished. No stable release yet. See the
> [roadmap](docs/roadmap/README.md).

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
| 13 skills | Loaded on demand: API, database, frontend, and hexagonal architecture, requirements engineering, spec writing, security audit, testing, performance, technical writing, and more |
| Verifier | A read-only subagent that tries to disprove a fix, a finding, or a "done" before you trust it |
| Memory | Optional. Curated notes in a git repository you own, shared across sessions, machines, and harnesses |
| Status line | Optional. Written to your settings only after you agree |

Three companion plugins install with it. [ponytail](https://github.com/DietrichGebert/ponytail) keeps the
code to the simplest thing that works. [i-have-adhd](https://github.com/ayghri/i-have-adhd) shapes answers
so you can act on them. [humanizer](https://github.com/blader/humanizer) keeps prose from reading as
generated.

## How a change moves

```text
/requirements → /bootstrap → /sprint plan → grilling → /spec → /epic
      → plan → tests → code → gates green → /audit → /closeout → commit → PR
```

Every requirement has an ID. A spec cites the IDs it covers. An epic and its sub-issues come from the
spec. Every commit and pull request points back to its issue. Nothing is committed until the gates pass
with zero errors and zero warnings and you have read the change.

## Quickstart

You need Claude Code 2.1.295 or later. On Windows, also Git for Windows: the hooks run under its `sh`.

```sh
claude plugin marketplace add DietrichGebert/ponytail
claude plugin marketplace add ayghri/i-have-adhd
claude plugin marketplace add blader/humanizer
claude plugin marketplace add https://github.com/ATNexusLab/tightship/releases/latest/download/marketplace.json
claude plugin install tightship@tightship
```

The next session asks which options to turn on. The [install guide](docs/install/claude-code/README.md)
covers options, updates, uninstalling, and moving from the pre-plugin install.

## Harnesses

| Harness | Status | Milestone |
| --- | --- | --- |
| Claude Code | In progress | M1 |
| Cursor | Planned | M2 |
| Antigravity CLI | Planned | M3 |
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
```

The build needs a `.identity-terms` file at the root. [CONTRIBUTING.md](CONTRIBUTING.md) explains it and
lists every gate.

## Docs

| Where | What |
| --- | --- |
| [Install guide](docs/install/claude-code/README.md) | Installing, options, updates, uninstalling, migration |
| [Requirements](docs/requirements/README.md) | The versioned SRS: every requirement and its acceptance criteria |
| [Architecture](docs/architecture/README.md) | The system map, in arc42 sections with C4 diagrams |
| [Decisions](docs/decisions/README.md) | The architecture decision records |
| [Roadmap](docs/roadmap/README.md) | Milestones, epics, and requirement coverage |
| [Design](docs/design/README.md) | The brand, its voice, and its tokens |

## Contributing

Issues and pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first. Report
vulnerabilities as [SECURITY.md](SECURITY.md) says, never in a public issue. Everyone follows the
[Code of Conduct](CODE_OF_CONDUCT.md).

## License

[MIT](LICENSE) © ATNexusLab
