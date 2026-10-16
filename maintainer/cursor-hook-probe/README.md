# Cursor hook probe (issue #65)

Maintainer-only plugin. It exercises the same hook command shape tightship will ship:
`${CURSOR_PLUGIN_ROOT}/bin/cursor-probe-hook <event>`. Logs land in `.cursor-hook-probe/` at the workspace
root. The first `beforeSubmitPrompt` prints the one-line payload in `payloads/limit-10000.json` (10 000
characters inside `additional_context`) so you can see whether Cursor accepts or truncates it.

Not built by `cargo xtask build` and not published to adopters.

## Prerequisites

- Cursor editor **3.25.6** or later; CLI **`agent` 2026.10.14** or later when you probe the CLI.
- A clone of this repository opened as the workspace.

## Install the probe locally

From the repository root:

```sh
cp -R maintainer/cursor-hook-probe ~/.cursor/plugins/local/cursor-hook-probe
chmod +x ~/.cursor/plugins/local/cursor-hook-probe/bin/cursor-probe-hook
```

On Windows (PowerShell):

```powershell
Copy-Item -Recurse maintainer\cursor-hook-probe $env:USERPROFILE\.cursor\plugins\local\cursor-hook-probe
```

Reload the editor window. Under Customize → Plugins, **cursor-hook-probe** should appear as **Local**.

CLI-only load:

```sh
agent --plugin-dir maintainer/cursor-hook-probe
```

On Windows, run `agent` from PowerShell with `$env:SHELL = $null` if hooks fail under Git Bash (see the
cursor-plugin skill).

## Production run (Unix editor)

Run once on your dev machine (Linux is enough; CI covers macOS/Windows for Rust, not live hooks).

1. Remove any old logs: `rm -rf .cursor-hook-probe` in the workspace.
2. New chat in the editor; send one short prompt. Confirm `.cursor-hook-probe/*.log` files exist for
   `sessionStart` and `beforeSubmitPrompt`.
3. In the `sessionStart` log, confirm `parent_comm` is **not** `pwsh` or `powershell`.
4. Record `workspace_roots` path shape from stdin (editor vs CLI if you test both).
5. On the first prompt, the kit fires the 10 000-character `additional_context` line. Note truncation,
   errors, or silence if you ask the agent what it received.
6. Optional: trigger a compaction and confirm `preCompact` logs and that `sessionStart` does not run again.
7. Copy results into `.claude/skills/cursor-plugin/SKILL.md` (probe checklist).

Optional: `agent --plugin-dir maintainer/cursor-hook-probe` in an interactive CLI session when `agent` is
installed.

## Windows notes

Expect `parent_comm` to involve PowerShell 7 (probed 2026-10-15). Git marketplace install E2E is tracked in
`docs/install/cursor/`, not issue #65.

## After production

- If the 10 000-character `additional_context` is rejected or truncated below 10 000, open a change request
  against `docs/specs/cursor-plugin/ship-for-cursor.md` before lowering `[contract] limit`.
- Close GitHub issue #65 when the skill checklist is **Done** for Windows and your Unix dev platform.
