# 0002. The tightship binary: modules by domain, a pure core, five dependencies

- Status: accepted
- Date: 2026-10-15

## Context

[ADR 0001](0001-rust-build-and-tokenized-canonical-content.md) splits the workspace into two crates: `xtask/`,
the maintainer's build, and `tightship/`, the one binary the plugin ships. It does not say what goes inside
either one. Epic #5 writes the binary's first code, epic #6 adds the handoff to it, and epic #7 adds the
status line. Three things were left open: which crates the binary depends on, how its code is split, and how
its folders are laid out. `xtask/src/` had also grown to 14 files in one folder, with the build and the scan
mixed together.

The forces:

- A hook adds at most 100 ms to a prompt at the 95th percentile
  ([NFR-PERF-02](../requirements/non-functional/performance.md#nfr-perf-02)), and a hook that exits with 2
  blocks the prompt. The [epic #5 spec](../specs/memory/keep-the-vault.md) forbids exit code 2 everywhere.
- The spec fixes every message, every exit code, and the usage text.
- Every input and output of the binary has exactly one implementation: the file system, git run as a
  process, the environment, and stdin.
- The spec's scenarios run the binary over a vault in a temporary folder.

## Decision

**Two crates, one per audience.** `xtask/` runs only on the maintainer's machine. `tightship/` ships to the
adopter. Neither depends on the other, and no third crate is shared between them. The scan's check for vault
notes reads front matter in its own ten lines, since it only needs to know whether a file is a note.

**The binary is split by domain.** `vault/` holds one file per subcommand plus the note format, `hook/`
holds one file per hook, and epics #6 and #7 add `handoff/` and `status/` next to them:

```text
tightship/src/
  main.rs       parses the arguments, runs one subcommand, maps a failure to exit 1 and its message
  failure.rs    one enum with the spec's error codes; each failure is reported once, by main
  project.rs    the project's scope, from the git repository's top folder
  git.rs        runs git in a folder
  vault.rs      finds the vault in TIGHTSHIP_VAULT
  vault/        note, path, init, search, new, lint, sync, context
  hook.rs
  hook/         start, export, checkpoint, prompt
```

A subcommand reads what it needs at the start (files, git, the environment, stdin), passes it to pure
functions, and writes once at the end. No trait stands for the file system, git, or the clock, because
each has one implementation. Tests run the built binary over real folders.

**Dependencies.** Amended by the [epic #7 spec](../specs/status-line/show-the-status-line.md) with
`unicode-width`. Each one is pinned to an exact version:

| Crate | For | Why not the alternative |
| --- | --- | --- |
| `clap` 4.6.7, derive | Arguments, subcommands, required flags | Parsing by hand, or with `lexopt`, means about 150 more lines of our own for missing values and unknown flags |
| `jiff` 0.2.38 | The local date of a note, the age of a note | The standard library has no time zone |
| `serde_json` 1.0.154 | The JSON a hook reads on stdin | Already in the workspace |
| `deunicode` 1.6.2 | Folding a title to the ASCII of its slug | Unicode normalization drops what it cannot decompose, where `deunicode` transliterates it |
| `unicode-width` 0.2.2 | The width a terminal gives the status line's text (M1, #7) | Counting characters overstates nothing for ASCII and gets every wide character, such as CJK or an emoji, wrong |

`serde_json` runs with its `preserve_order` feature (M1, #7), so rewriting the adopter's settings for the
status line keeps their keys in order.

`clap` runs through `try_parse`, never `parse`, since clap exits with 2 on a usage error. `main` turns each
usage error into the spec's message and exit 1. Front matter is read and written by hand: it is flat
`key: value` lines, and ADR 0001 already turned down the archived YAML crate. git runs as a process, never
through `git2` or `gix`, so the binary uses the adopter's own git, credentials, and configuration.

**The build is split by command.** `xtask/src/build/` holds the reading, checking, and rendering of the
plugin, `xtask/src/scan/` holds gitleaks and the vault-note check, and `git.rs` and `diagnostics.rs` stay at
the root because both commands use them.

Module files follow the Rust 2018 layout, `build.rs` beside `build/`, not `mod.rs`.

## Consequences

- A new function of the binary is a new folder and one more match arm in `main`. It needs no new crate,
  launcher, or release target.
- The core cannot be unit-tested against a fake file system. Every rule is tested through the binary, the
  way the spec's scenarios state it.
- The adopter's binary carries five dependencies and their own dependencies, and each one is bumped on purpose.
- clap's generated help is replaced by the spec's usage text.

## Alternatives considered

- **One crate per function** (`vault`, `hook`, `status`). Rejected: every function would be its own binary,
  with its own launcher and its own release targets, against the single binary of ADR 0001.
- **A crate named `vault`.** Rejected: the binary also holds the hooks, the handoff, and the status line.
- **Hexagonal ports and adapters.** Rejected: every port would have a single adapter. It would double the
  files with no second implementation in sight.
- **A shared library crate for the note format.** Rejected: the build would depend on code that ships to
  adopters, when it only needs to tell whether a file is a note.
- **A `crates/` folder.** Rejected: it only pays off with three or more crates.
