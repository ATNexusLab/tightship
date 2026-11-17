# Where tightship runs

English · [Português](README.pt-BR.md)

Where tightship runs: one row per surface of each harness on your machine, with how the plugin gets there
and which of its pieces load. A piece that does not load says why.

| Harness | Surface | How it gets there | Contract | Skills | Commands | Hooks | Status line | Vault |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Claude Code | CLI¹ | The installer, or `claude plugin install` | Yes | Yes | Yes | Yes | Yes | Yes |
| Claude Code | Desktop app | The same install as the CLI: the app's local sessions read `~/.claude` | Yes | Yes | Yes | Yes | Yes | Yes |
| Claude Code | Zed's terminal¹ | The same install as the CLI | Yes | Yes | Yes | Yes | Yes | Yes |
| Claude Code | Zed's agent panel | The `claude-acp` registry entry | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) |
| Cursor | Editor | The installer, or a copy into `~/.cursor/plugins/local/` | Yes | Yes | Yes | Yes | No⁴ | Yes |
| Cursor | `agent` CLI² | The same install as the editor | Yes | Yes | Yes | Yes | Yes | Yes |
| Cursor | Zed's terminal² | The same install as the editor | Yes | Yes | Yes | Yes | Yes | Yes |
| Cursor | Zed's agent panel | The `cursor` registry entry | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) |
| Antigravity | CLI³ | The installer, or `agy plugin install` | Yes | Yes | Yes | Yes | Yes | Yes |
| Antigravity | IDE | The same install as the CLI: the IDE reads `~/.gemini/config/plugins/` | Yes | Yes | Yes | Yes | No⁵ | Yes |
| Antigravity | Desktop app | The same install as the CLI: the app reads `~/.gemini/config/plugins/` | Yes | Yes | Yes | Yes | No⁵ | Yes |
| Antigravity | Zed's terminal³ | The same install as the CLI | Yes | Yes | Yes | Yes | Yes | Yes |
| Antigravity | Zed's agent panel | The `antigravity-acp` registry entry | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) | [Not checked yet](https://github.com/ATNexusLab/tightship/issues/163) |

On a CLI row, a `Yes` comes from the unit and integration tests, which run the CLI. Zed's terminal runs the
same CLI, so its row repeats the CLI's. No test runs a desktop app, an IDE, or Zed's agent panel: a `Yes`
there comes from a session in use, and a cell not checked yet links the issue that tracks it.

1. Claude Code: `xtask/tests/contract.rs` builds the contract and its hooks and runs them;
   `xtask/tests/build/references.rs` builds the skills and commands; `xtask/tests/install/claude_code.rs`
   installs the plugin, applies the status line, and creates the vault.
2. Cursor: `xtask/tests/cursor_contract.rs` checks the hooks and runs the contract;
   `xtask/tests/build/cursor.rs` builds the skills, commands, and status line hook;
   `xtask/tests/install/cursor.rs` installs into `~/.cursor/plugins/local/`;
   `tightship/tests/hook/start.rs` reads the vault. No test runs the vault's hooks from the built plugin.
3. Antigravity: `xtask/tests/antigravity_cli_contract.rs` runs the contract hooks;
   `xtask/tests/build/manifest.rs` builds the skills and commands; `xtask/tests/install/antigravity_cli.rs`
   installs and writes the status line; `tightship/tests/hook/contract_part.rs` injects the vault's notes. No
   test renders the status line from Antigravity's settings folder.
4. Cursor's editor has no status line: only the `agent` CLI reads `statusLine` in `~/.cursor/cli-config.json`.
5. Antigravity documents `statusLine` [for the CLI only](https://antigravity.google/docs/settings); the IDE
   and the app draw none.

Web surfaces are not supported: claude.ai, claude.ai/code, Cursor's Cloud Agents, and Antigravity's Remote
Agents. tightship works on a checkout on your machine.

Install guides: [Claude Code](claude-code/README.md) · [Cursor](cursor/README.md) ·
[Antigravity CLI](antigravity-cli/README.md).
