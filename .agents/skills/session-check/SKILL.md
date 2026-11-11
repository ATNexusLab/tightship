---
name: session-check
description: Use when loading a built plugin in a harness session to check it — the command per harness and the setup each one needs first.
---

# Session check

Build first with `cargo xtask build`; each command loads what `dist/` holds.

| Harness | Command |
| --- | --- |
| Claude Code | `claude --plugin-dir dist/claude-code` |
| Claude Code, installed | `claude plugin marketplace add ./dist/claude-code`, then `claude plugin install tightship@tightship` |
| Cursor | `agent --plugin-dir dist/cursor`, an interactive CLI session |
| Antigravity CLI | `agy plugin validate dist/antigravity-cli --strict` |

## Setup per harness

- **Claude Code** under `--plugin-dir` has no vault option: export `CLAUDE_PLUGIN_OPTION_VAULT_FOLDER`
  before `claude`. Pass `--settings '{"enabledPlugins":{"tightship@tightship":false}}'`, so an installed copy
  stays off.
- **Cursor**'s `agent` runs from PowerShell with `$env:SHELL = $null`.
- **Antigravity CLI**: `agy plugin install` writes into the adopter's own config, and on Windows needs
  `hooks.windows.json` moved over `hooks.json` first, as `install.ps1` does.

## The leftover clean-up

`hook clean` removes only what the harness's `clean.json` lists, from the real home folder. Check it in a
throwaway home: set `HOME` (and `CLAUDE_CONFIG_DIR` for Claude Code) to an empty folder, seed a few listed
leftovers, such as `~/.config/ponytail/` and an `enabledPlugins` entry, then start a session. The first
session shows the notice and removes them; the second shows nothing.
