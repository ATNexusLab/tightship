# Build

**Lives in:** `xtask/` · **Technology:** Rust, run as `cargo xtask` · **Milestone:** M1, epic #1

The maintainer's tool. It never runs on an adopter's machine.

| Command | Does |
| --- | --- |
| `cargo xtask build` | Reads the content and the definitions, checks every token, literal reference, harness term, and front matter, renders `dist/claude-code/`, then scans it for the maintainer's identity |
| `cargo xtask scan` | Runs gitleaks over the whole history and looks for vault notes in the working tree |

**It never** writes outside `dist/`, and never leaves a half-written plugin behind: it writes into a
temporary folder and renames it only when every check passed.

**To change it safely:** every rule, error code, and message is in the
[epic #1 spec](../../specs/canonical-source-and-build/build-the-plugin.md), and the tests
in `xtask/` lock them.
