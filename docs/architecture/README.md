# Architecture

The map of the system, in the sections of arc42, with C4 diagrams. The structure follows
[ADR 0001](../decisions/0001-rust-build-and-tokenized-canonical-content.md). Everything not built yet
is drawn dashed, with the milestone and epic that builds it.

## Introduction and goals

The workflow is a way of working with coding agents: an always-on contract, on-demand skills, a command
per step of the development loop, a read-only verifier, an optional memory layer, and an optional status
line. The product packages it as a native plugin for four harnesses, built from one canonical source in
this repository ([SRS](../requirements/README.md)).

Quality goals, highest first:

1. One authored copy of each piece, whatever the harness ([NFR-MNT-01](../requirements/non-functional/maintainability.md#nfr-mnt-01),
   [NFR-FLEX-06](../requirements/non-functional/flexibility.md#nfr-flex-06)).
2. A plugin failure never blocks a session ([NFR-REL-01](../requirements/non-functional/reliability.md#nfr-rel-01)).
3. Nothing of the maintainer's identity or vault ships ([NFR-SEC-01](../requirements/non-functional/security.md#nfr-sec-01),
   [NFR-SEC-02](../requirements/non-functional/security.md#nfr-sec-02)).
4. Session start and each prompt stay within their time budgets
   ([NFR-PERF-01](../requirements/non-functional/performance.md#nfr-perf-01),
   [NFR-PERF-02](../requirements/non-functional/performance.md#nfr-perf-02)).

## Constraints

See [the operating environment and constraints](../requirements/overview.md).

## Context and scope

![C4 context](diagrams/context.drawio.svg)

Dashed boxes are not built yet. Every element belongs to the milestone in its label.

| Element | Kind | What flows |
| --- | --- | --- |
| Maintainer | Person | Edits each workflow piece once, in this repository |
| Adopter | Person | Installs the plugin, picks its options, invokes its commands |
| This repository | External system, GitHub | Holds the canonical source and the catalog each harness installs from |
| Claude Code, Cursor, Antigravity CLI, Codex | External systems, one milestone each ([BR-01](../requirements/business-rules.md#br-01)) | Load the plugin in its harness's own format |
| Companion plugins | External systems | Installed with the plugin where the harness allows it ([FR-INS-02](../requirements/functional/install.md#fr-ins-02)); in Claude Code, as the plugin's dependencies, each from its own catalog (M1, #2); in Antigravity CLI, by `install.sh` / `install.ps1` or the install guide's commands (M3, #59) |
| Vault | External system, the adopter's git repository | Notes and handoffs the memory layer reads and writes |

Out of scope: [the SRS's list](../requirements/README.md#scope).

## Solution strategy

From [ADR 0001](../decisions/0001-rust-build-and-tokenized-canonical-content.md):

- **One authored copy, many plugins.** Every piece is one Markdown file under `content/`; a build renders
  it into each harness's plugin under `dist/`, which is never committed.
- **Harness-neutral content.** A piece refers to another piece, or to a harness capability, through a
  [token](concepts/tokens.md). Harness names, paths, and tools live only in `harnesses/<harness>/`.
- **A rule lives where its step reaches it** (M4, #118). The contract keeps only the hard lines and what every
  task needs; a rule of one step is in that step's skill or command, and a repository's procedures are project
  skills in `.agents/skills/`.
- **Rust throughout.** The build is a Cargo `xtask`; the plugin ships one compiled binary, `tightship`, so an
  adopter installs no runtime.
- **The plugin installs as a release archive** that carries the binary for every target and a `sh` launcher
  (plus `bin/tightship.cmd` for Windows `cmd /c`, M3, #59) that picks one.

## Building blocks

[Building blocks](building-blocks/README.md), with the C4 container diagram.

## Runtime

**Session start: the contract** (M1, #4; M3, #59). Claude Code starts a session, runs `/clear`, or compacts
the context, and fires the plugin's `SessionStart` hooks in parallel. Each prints one part of the contract
from `contract/`, or nothing when the contract option or its own option is `false`, and Claude Code adds each
print to the context as its own reminder. In Antigravity CLI (M3, #59), `SessionStart` runs `tightship hook
contract` for each part, which wraps the text in `injectSteps` JSON; after `/compact`, the agent loads the
`contract` skill ([contract](concepts/contract.md)).

**Session start: memory** (M1, #5; M3, #59). With memory on, the same event runs `tightship hook start`. It
reads the pinned notes of the project's scope and of `all`, and the project's handoff, from the vault folder
the adopter set, and, since M4 (#118), the project's notes tagged `to-repo` as debts, and prints them within
9,000 characters (wrapped in `injectSteps` JSON in Antigravity CLI,
M3, #59). In Claude Code it also appends `export TIGHTSHIP_VAULT=…` to the session's environment file; in
Antigravity CLI, `tightship` reads `vault_folder` directly from `${PLUGIN_DATA}/options.toml`
([memory](concepts/memory.md)).

**A compaction: the checkpoint** (M1, #5; M2, #58; M4, #118). A compaction prints the checkpoint, from
`SessionStart` on the source `compact` in Claude Code and from `preCompact` in Cursor. No hook runs on a prompt
since M4, and Antigravity CLI, with no compaction hook, gets no checkpoint.

**The agent reaches the vault** (M1, #5; M3, #59). The agent runs `tightship vault search`, `new`, or `lint`
through its shell. The binary finds the vault in `TIGHTSHIP_VAULT` (exported in Claude Code) or in
`vault_folder` of `${TIGHTSHIP_DATA}/options.toml` (in Antigravity CLI, M3, #59).

**The adopter releases a handoff** (M1, #6). The command `handoff release` has the agent read the project's
repository, write the handoff's body, run `tightship vault handoff release`, which writes it and removes the
scope's earlier ones, and run `tightship vault sync` to push it. The project's repository is only read
([memory](concepts/memory.md)).

**The adopter claims a handoff** (M1, #6). The command `handoff claim` has the agent sync the vault, run
`tightship vault handoff claim`, which removes each of the scope's handoffs and prints the ones it removed, so
two sessions that claim at once never both take one, sync again, and compare each handoff with the
repository after a `git fetch`. It shows the next step and the open decisions, and waits.

**Session start: the status line** (M1, #7; M3, #59). On `startup`, `tightship hook status-line` runs whatever
the status line option's state. With the option on, it saves the adopter's `statusLine` in the plugin's data
folder and writes the workflow's into the user settings (`~/.claude/settings.json`,
`~/.cursor/cli-config.json`, or `~/.gemini/antigravity-cli/settings.json`), and the harness then runs
`tightship status-line render` as the status line. With the option off, the hook puts the saved value back
([status line](concepts/status-line.md)).

**The adopter syncs the vault** (M1, #5). The command `sync` has the agent run `tightship vault sync`, which
commits the vault's changes, rebases them on its git remote, renumbers colliding ids, removes the notes past
their lifetime and rewrites the vault's rules block, each in its own commit (M4, #118), and pushes. A conflict
or a git failure stops it with exit 1 and the next step, and the agent resolves it and syncs again
([memory](concepts/memory.md)).

## Deployment

[Deployment](deployment.md): environments, packaging, and the adopter's prerequisites.

## Crosscutting concepts

[Crosscutting concepts](concepts/README.md): tokens, options, the contract, memory, and security.

## Architecture decisions

[Decisions](../decisions/README.md).

## Quality requirements

See [the non-functional requirements](../requirements/non-functional/README.md).

## Risks and technical debt

The known risks are the SRS's [assumptions and dependencies](../requirements/overview.md#assumptions-and-dependencies):
each harness's plugin format and the companion plugins can change without notice.

## Glossary

See [the glossary](../requirements/glossary.md).
