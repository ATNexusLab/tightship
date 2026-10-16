# tightship binary

**Lives in:** `tightship/` · **Technology:** Rust · **Milestone:** M1, from epic #5; epic #6 adds the handoff;
M3, epic #59 adds `hook contract`, `options.toml`, and `injectSteps` output

The one program the plugin ships, with a subcommand per function. Epic #5 builds the first two:

| Subcommand | Run by | Does |
| --- | --- | --- |
| `tightship hook contract [--gate <option>] <file>` | The plugin's `SessionStart` hooks in Antigravity CLI (M3, #59) | Prints one rendered contract part unless `contract` or `<option>` is `false` |
| `tightship hook start\|export\|checkpoint\|prompt` | The plugin's hooks | Puts the [memory](../concepts/memory.md) context and the checkpoints in the session, and exports the vault's folder to it |
| `tightship vault path\|init\|search\|new\|lint\|sync` | The agent, through its shell | Reads, writes, lints, and syncs the adopter's vault, and creates one from the template |
| `tightship vault handoff release\|claim` | The agent, through the command `handoff` (M1, #6) | Writes the project's handoff, or prints and removes it; never runs git |
| `tightship hook status-line` | The plugin's hook, at `startup` | Offers the [status line](../concepts/status-line.md), follows the plugin's folder, or puts the previous one back (M1, #7) |
| `tightship status-line enable\|decline` | The agent, on the adopter's answer | Writes the workflow's status line into the user settings, or records the no (M1, #7) |
| `tightship status-line render` | Claude Code and Antigravity CLI, as the status line | Prints the baseline's two rows for the session's JSON (M1, #7; M3, #59) |

The binary reads `TIGHTSHIP_*` variables, which the hook lines or the launcher of each harness set from that
harness's own, and falls back to `${TIGHTSHIP_DATA}/options.toml` when an option variable is unset (M3, #59),
so it names no harness, plus `COLUMNS`, the terminal's width, for the status line (M1, #7). When
`TIGHTSHIP_HOOK_FORMAT=inject-steps`, non-empty hook output is wrapped in
`{"injectSteps":[{"systemMessage":{"systemMessage":"..."}}]}` (M3, #59).

`bin/tightship` is a `sh` launcher (and `bin/tightship.cmd` is a Windows batch launcher for harnesses that run
hooks under `cmd /c`, M3, #59) that runs `bin/<platform>/tightship` for the machine, among `macos`,
`linux-x64`, `linux-arm64`, and `windows-x64`. The plugin's `bin/` is on the `PATH` of the agent's shell tool,
so a piece calls `tightship` by name. Until epic #8, `cargo xtask build` compiles only the platform of the
machine that builds. From epic #8, Windows on arm64 runs the `windows-x64` binary emulated, the hooks and the
status line run the launcher through `sh`, and the launcher restores the executable bit of the binary and its
own when an install dropped it.

Inside, the code is split by domain, `vault/`, `hook/`, and `status/` (M1, #7), with `options.rs` (M3, #59),
a pure core, and the I/O at the edges, and it depends on `clap`, `jiff`, `serde_json`, `deunicode`, and
`unicode-width` (M1, #7)
([ADR 0002](../../decisions/0002-tightship-binary-modules-and-dependencies.md)).

**It never** connects to any host other than the vault's git remote, and only from `vault sync`, never from a
hook. It writes only where [security](../concepts/security.md) lists, and never exits with 2, the code with
which a hook blocks a prompt. A failure never blocks the session ([NFR-REL-01](../../requirements/non-functional/reliability.md#nfr-rel-01)).

**To change it safely:** every rule, message, and exit code is in the specs of
[epic #5](../../specs/memory/keep-the-vault.md), [epic #6](../../specs/handoff/hand-off-the-session.md),
[epic #7](../../specs/status-line/show-the-status-line.md), and
[epic #59](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md), and the tests in `tightship/` lock
them.
