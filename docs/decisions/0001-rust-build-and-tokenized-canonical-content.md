# 0001. Rust build and tokenized canonical content

- Status: accepted
- Date: 2026-10-14
- Amended: 2026-10-14, by the [epic #3 spec](../specs/options/choose-the-options.md): the options' state
  is the harness's, not the plugin's
- Amended: 2026-10-15, by the [epic #4 spec](../specs/contract/load-the-contract.md): a hook command line that
  only prints a file the plugin ships is not a program
- Amended: 2026-10-15, by the [epic #5 spec](../specs/memory/keep-the-vault.md): the plugin also writes in two
  places the harness owns, its data folder and the session's environment file

## Context

The workflow is shipped as a native plugin for four harnesses, one milestone each
([BR-01](../requirements/business-rules.md#br-01)), from one authored copy of every piece
([NFR-MNT-01](../requirements/non-functional/maintainability.md#nfr-mnt-01)). Three open questions of the SRS
block the first line of code:

- **The stack** (OQ-02): the format of the canonical content, the language of the build, and the language of
  every script the plugin ships.
- **The plugin's reach** (OQ-03): which network connections it makes and where it writes on the adopter's
  machine.
- **Harness capabilities** (OQ-09): how a canonical piece names plan mode, a built-in review command, or
  another capability native to one harness.

The forces, checked against Claude Code 2.1.299 and its documentation on 2026-10-14:

- Claude Code guarantees no runtime on the adopter's machine. Its native installer carries neither Node.js
  nor Python, and the documentation promises neither.
- A Claude Code plugin is a directory tree. It has no mechanism to ship a different file per operating system
  or architecture.
- Hooks run in shell form through `sh` on macOS and Linux, and through Git Bash on Windows, falling back to
  PowerShell when Git Bash is absent. No command line reads the same in `sh` and in PowerShell.
- A plugin's `bin/` folder is on the `PATH` of the agent's shell tool while the plugin is enabled.
- Claude Code namespaces every plugin piece under the plugin's name: a command `spec` of plugin `tightship`
  is invoked as `/tightship:spec`, so a reference written for the pre-plugin install no longer resolves
  ([FR-SKL-03](../requirements/functional/skills.md#fr-skl-03)).
- A hook adds at most 100 ms to a prompt at the 95th percentile
  ([NFR-PERF-02](../requirements/non-functional/performance.md#nfr-perf-02)), on macOS, Linux, and Windows
  without WSL ([NFR-FLEX-05](../requirements/non-functional/flexibility.md#nfr-flex-05)).
- The canonical content names no harness, outside the places this decision sets apart
  ([NFR-FLEX-06](../requirements/non-functional/flexibility.md#nfr-flex-06)).

## Decision

**Language.** Rust, pinned in `rust-toolchain.toml`, for the build and for every program the plugin ships. The
repository is one Cargo workspace:

- `xtask/` — the build and the repository's scans, run as `cargo xtask build` and `cargo xtask scan`.
- `tightship/` — the one binary the plugin ships, with one subcommand per function (hooks, memory, status
  line). It starts with the first epic that needs a hook to run logic.

A hook command line that only prints a file the plugin ships, such as each part of the contract, is not a
program: it is one `sh` line, with the same prerequisite as the `sh` launcher.

**Canonical content.** One Markdown file per piece under `content/`: `skills/<name>/SKILL.md` with its
supporting files, `commands/<name>.md`, `agents/<name>.md`. Front matter is TOML between `+++` lines, with
harness-neutral keys only. A reference to another piece, or to a harness capability, is a token:

| Token | Names |
| --- | --- |
| `{{skill:<name>}}` | A skill in `content/skills/` |
| `{{command:<name>}}` | A command in `content/commands/` |
| `{{agent:<name>}}` | A subagent in `content/agents/` |
| `{{capability:<name>}}` | A harness capability, such as plan mode |

A new kind of token is added by the epic that adds a new kind of piece.

**Per-harness places.** `harnesses/<harness>/` is the only place where a harness's name, paths, and tools may
appear. Each harness folder holds one `harness.toml` with:

- How the harness names each kind of piece, such as `/{plugin}:{name}` for a command.
- The text of every capability: the harness's own feature, or the plain instruction that replaces it where the
  harness has none.
- The harness's denylist: its names, paths, and tool names that the canonical content must not hold.

**Build.** `cargo xtask build` reads `content/`, resolves every token for each harness, and writes that harness's
plugin to `dist/<harness>/`, which git ignores. A token that names a missing piece or an undefined capability
fails the build: that check is how references resolve. The generated plugin is never committed.

**Distribution.** A release attaches one archive per harness to a GitHub Release, and the catalog points at it.
The archive holds the `tightship` binary for every target: macOS universal (arm64 and x86_64), Linux x64 and
arm64 linked statically with musl, and Windows x64. `bin/tightship` is a `sh` launcher that picks the binary
for the machine it runs on. GitHub Actions builds every target on its macOS, Linux, and Windows runners, at
release only.

**Adopter prerequisites.** Nothing on macOS and Linux. Git for Windows on Windows, so that hooks run under Git
Bash. Git on every system when the memory option is on.

**Reach.** The plugin connects to one network host: the git remote of the adopter's vault, when it pulls and
pushes a handoff. Outside what the harness owns, it writes in two places only:

- The adopter's vault.
- The harness's settings file, for the status line only, after the adopter consents
  ([FR-STL-02](../requirements/functional/status-line.md#fr-stl-02)).

The options' state is kept by the harness, in its own user configuration, and reaches the plugin's hooks as
environment variables.

In two places the harness owns and deletes, the plugin also writes small state of its own:

- The plugin's data folder, for the prompt count of the memory checkpoint, one file per session.
- The session's environment file, for one `export` of the vault's folder, so the binary finds the vault when
  the agent runs it.

The build writes only to `dist/`.

## Consequences

- An adopter installs no runtime, and a hook starts in about 1 ms.
- A piece is edited in one file, and a harness renders it. Adding a harness adds a folder under `harnesses/`
  and a renderer, and leaves `content/` untouched
  ([NFR-FLEX-07](../requirements/non-functional/flexibility.md#nfr-flex-07)).
- The maintainer writes references as tokens, never as a harness's literal names. The build rejects a literal
  reference to a known piece.
- A release needs GitHub Actions and a build matrix. Each archive carries every target, from 15 to 20 MB.
- The macOS binary carries only the linker's ad-hoc signature. The release epic tests that it runs from an
  archive Claude Code downloaded.
- An adopter on Windows without Git for Windows cannot run the plugin's hooks.

## Alternatives considered

- **Python, standard library only.** Already the language of the baseline's memory script. Rejected: it
  makes every adopter install Python, and the maintainer wants to depend on it as little as possible.
- **TypeScript on Bun.** Starts in 10 ms, measured on 2026-10-14, and ships plain `.ts` files with no release
  matrix. Rejected: every adopter installs Bun, while a compiled binary needs nothing.
- **TypeScript on Node.js.** Starts in 49 ms on a fast Linux machine, which leaves little of the 100 ms
  budget on Windows. Rejected for the same prerequisite as Bun.
- **Go.** The same distribution as Rust. Rejected: the maintainer has never used it.
- **Downloading the binary on first use.** Rejected: it needs the network and races the first hook.
- **A separate installer for the binary.** Rejected: it breaks installation through the harness alone
  ([FR-INS-01](../requirements/functional/install.md#fr-ins-01)) and leaves the binary behind on uninstall
  ([FR-INS-06](../requirements/functional/install.md#fr-ins-06)).
- **Committing the generated plugin.** Rejected: every change would appear twice in a pull request, and the
  copy could be edited by hand.
- **YAML front matter.** Rejected: the serde YAML crate is archived, while the `toml` crate is maintained.
