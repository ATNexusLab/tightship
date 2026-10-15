# Build

**Lives in:** `xtask/` · **Technology:** Rust, run as `cargo xtask` · **Milestone:** M1, epics #1, #4, and #5

The maintainer's tool. It never runs on an adopter's machine.

| Command | Does |
| --- | --- |
| `cargo xtask build` | Reads the content and the definitions, checks every token, literal reference, harness term, and front matter, renders `dist/claude-code/`, the [contract](../concepts/contract.md) packed into parts within the harness's limit (M1, #4), the [tightship binary](tightship-binary.md) compiled for the machine that builds, with home paths remapped, and the memory hooks (M1, #5), then scans it for the maintainer's identity |
| `cargo xtask scan` | Runs gitleaks over the whole history and looks for vault notes in the working tree |

**It never** writes outside `dist/`, and never leaves a half-written plugin behind: it writes into a
temporary folder and renames it only when every check passed.

**To change it safely:** every rule, error code, and message is in the
[epic #1 spec](../../specs/canonical-source-and-build/build-the-plugin.md), and the tests
in `xtask/` lock them.
