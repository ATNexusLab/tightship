# Build

**Lives in:** `xtask/` · **Technology:** Rust, run as `cargo xtask` ·
**Milestone:** M1, epics #1, #4, #5, #6, and #8; M2, epic #58 (Cursor renderer)

The maintainer's tool. It never runs on an adopter's machine.

| Command | Does |
| --- | --- |
| `cargo xtask build` | Reads the content and every `harnesses/<name>/harness.toml`, checks every token, the option tokens of M1, #6 included, literal reference, harness term, and front matter, renders each harness into `dist/<name>/` — for Claude Code the [contract](../concepts/contract.md) packed into parts within that harness's limit (M1, #4), memory hooks (M1, #5), and for Cursor the contract and hooks of epic #58 — compiles the [tightship binary](tightship-binary.md) for the machine that builds, with home paths remapped, then scans each output tree for the maintainer's identity |
| `cargo xtask build --binaries <folder>` | The same, with the four prebuilt binaries of `<folder>` in place of the compile step, for the release pipeline (M1, #8; both harnesses in M2, #58) |
| `cargo xtask package <tag>` | Packs `dist/claude-code/` into `dist/tightship-claude-code.zip` and `dist/cursor/` into `dist/tightship-cursor.zip`, with fixed file modes, and writes each release catalog, taking the version and the repository from the built manifests (M1, #8; Cursor archive M2, #58) |
| `cargo xtask scan` | Runs gitleaks over the whole history and looks for vault notes in the working tree |

The code is split by command: `xtask/src/build/`, `xtask/src/package.rs`, and `xtask/src/scan/`, with
`git.rs` and `diagnostics.rs` shared at the root ([ADR 0002](../../decisions/0002-tightship-binary-modules-and-dependencies.md)).

**It never** writes outside `dist/`, and never leaves a half-written plugin behind: it writes into a
temporary folder and renames it only when every check passed.

**To change it safely:** every rule, error code, and message is in the
[epic #1 spec](../../specs/canonical-source-and-build/build-the-plugin.md), and the tests
in `xtask/` lock them.
