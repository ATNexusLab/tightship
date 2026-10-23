# tightship binary

**Lives in:** `tightship/` · **Technology:** Rust · **Milestone:** M1, from epic #5; epic #6 adds the handoff;
M3, epic #59 adds `options.toml` and `injectSteps` output

The one program the plugin ships, with a subcommand per function. Epic #5 builds the first two:

| Subcommand | Run by | Does |
| --- | --- | --- |
| `tightship hook start\|export\|checkpoint` | The plugin's hooks | Puts the [memory](../concepts/memory.md) context and the checkpoint in the session, and exports the vault's folder to it; `hook prompt` is gone since M4 (#118) |
| `tightship vault path\|init\|search\|new\|lint\|sync` | The agent, through its shell | Reads, writes, lints, and syncs the adopter's vault, and creates one from the template; the sync also expires notes and keeps the rules block (M4, #118) |
| `tightship vault handoff release\|claim` | The agent, through the command `handoff` (M1, #6) | Writes the project's handoff, or removes it and prints what it removed; never runs git |
| `tightship hook status-line` | The plugin's hook, at `startup` | Writes the [status line](../concepts/status-line.md) into the user settings while its option is on, follows the plugin's folder, or puts the previous one back (M1, #7; #55) |
| `tightship status-line render` | Claude Code, Cursor's CLI, and Antigravity CLI, as the status line | Prints the baseline's two rows for the session's JSON (M1, #7; M2, #58; M3, #59) |
| `tightship hook contract-part <file>` | The plugin's hooks in Cursor (M2, #58) and Antigravity CLI (#89) | Prints one packed contract part from `${TIGHTSHIP_ROOT}/contract/` unless its option is `false` |
| `tightship hook update` | The session-start hook in Cursor and Antigravity CLI | Prints the pending update notice once, and starts `tightship update` as a detached process when the last check is more than a day old |
| `tightship update` | That hook, or the adopter | Downloads the latest release when it is newer, checks its sha256, and swaps the plugin's files in place, rolling back on failure |
| `tightship hook marketplace-auto-update` | Claude Code's hook, at `startup` | Writes `autoUpdate` on the plugin's entry in `extraKnownMarketplaces`, following the `auto_update` option |
| `tightship options set\|list\|reset\|disabled` | The adopter | Edits `${TIGHTSHIP_DATA}/options.json`; `disabled` exits 0 when the option is `false`, and rejects `vault_folder`, which is no on/off option (#99) |

The binary reads `TIGHTSHIP_*` variables, which the hook lines or the launcher of each harness set from
that harness's own, and falls back to `${TIGHTSHIP_DATA}/options.toml` when an option variable is unset
(M3, #59), or to no file at all when `TIGHTSHIP_DATA` is unset (#90), so it names no harness, plus
`COLUMNS`, the terminal's width, for the status line (M1, #7). When `TIGHTSHIP_HOOK_OUTPUT` names a JSON
file, non-empty hook output fills that file's `"{text}"` slot, and the binary prints the filled JSON; the
harness folder ships that file (M3, #59). The memory checkpoint takes its shape from
`TIGHTSHIP_CHECKPOINT_OUTPUT` when it is set. `hook start` reads the project's folder at the JSON pointer
`TIGHTSHIP_PROJECT_POINTER` names in the hook's input, when it is set. `update` reads the plugin's name,
version, and repository from the manifest `TIGHTSHIP_MANIFEST` names, and picks its archive by
`TIGHTSHIP_HARNESS`. `TIGHTSHIP_STATUS_LINE_COMMAND` replaces the status line command the binary writes
into the settings, and `status-line render` strips `TIGHTSHIP_MODEL_PREFIX` from the model's name (#78).
`cargo xtask build` fails when a harness's neutrality term appears in `tightship/src/`.

`bin/tightship` is a `sh` launcher (and `bin/tightship.cmd` is a Windows batch launcher that PowerShell, cmd,
and hooks under `cmd /c` find, M3, #59, #116) that runs `bin/<platform>/tightship` for the machine, among
`macos`,
`linux-x64`, `linux-arm64`, and `windows-x64`. The plugin's `bin/` is on the `PATH` of the agent's shell tool,
so a piece calls `tightship` by name. Until epic #8, `cargo xtask build` compiles only the platform of the
machine that builds. From epic #8, Windows on arm64 runs the `windows-x64` binary emulated, the hooks and the
status line run the launcher through `sh`, and the launcher restores the executable bit of the binary and its
own when an install dropped it.

Inside, the code is split by domain, `vault/`, `hook/`, `status/` (M1, #7), and `update/`, with
`options.rs` (M3, #59), a pure core, and the I/O at the edges, and it depends on `clap`, `jiff`,
`serde_json`, `deunicode`, `unicode-width` (M1, #7), and `toml` (M3, #59)
([ADR 0002](../../decisions/0002-tightship-binary-modules-and-dependencies.md)). `update/` downloads,
hashes, and extracts through `curl`, `sha256sum` or `shasum`, and `unzip` on macOS and Linux, and through
`powershell` on Windows, run as processes the way `git` is
([ADR 0003](../../decisions/0003-the-plugin-updates-itself-from-github-releases.md)).

**It never** connects to any host other than the vault's git remote, from `vault sync`, and the GitHub
Releases of the plugin's repository, from `update`, which a hook starts as a detached process and never waits
on ([ADR 0003](../../decisions/0003-the-plugin-updates-itself-from-github-releases.md)). It writes only
where [security](../concepts/security.md) lists, and never exits with 2, the code with which a hook blocks a
prompt. A failure never blocks the session ([NFR-REL-01](../../requirements/non-functional/reliability.md#nfr-rel-01)).

**To change it safely:** every rule, message, and exit code is in the specs of
[epic #5](../../specs/memory/keep-the-vault.md), [epic #6](../../specs/handoff/hand-off-the-session.md),
[epic #7](../../specs/status-line/show-the-status-line.md),
[epic #59](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md), and
[auto-update](../../specs/auto-update/update-the-plugin.md), and the tests in `tightship/` lock them.
