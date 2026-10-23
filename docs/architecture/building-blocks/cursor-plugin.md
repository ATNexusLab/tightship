# Cursor plugin

**Lives in:** `dist/cursor/`, generated and ignored by git · **Milestone:** M2, epic #58

The `tightship` plugin as Cursor loads it: `.cursor-plugin/plugin.json`, `skills/<name>/`, `commands/<name>.md`,
and `agents/<name>.md`. Skills and commands are separate folders; nothing is prefixed with the plugin name in
the UI.

The manifest declares [options](../concepts/options.md) as `variables` for Customize → Configure. Hooks and
the binary read option state only from `$HOME/.cursor/tightship/options.json`, which `tightship options`
maintains (M2, #58).

`hooks/hooks.json` uses camelCase events. The [contract](../concepts/contract.md) reaches the agent through
`sessionStart` and `beforeSubmitPrompt` hooks that run `${CURSOR_PLUGIN_ROOT}/bin/tightship hook contract-part`
and print JSON with `additional_context`, the shape of the plugin's `hook-output.json` (M2, #58). Memory
hooks run the same binary subcommands as in Claude Code, mapped to `sessionStart` and `preCompact`, and
`beforeSubmitPrompt` carries only the contract since M4 (#118); the `preCompact` checkpoint prints
`user_message`, the shape of `checkpoint-output.json` (#78). The last `sessionStart` hook runs
`tightship hook update`, which announces an update and starts the next check
([auto-update spec](../../specs/auto-update/update-the-plugin.md)).

`bin/` holds the [tightship binary](tightship-binary.md), its `.cmd` launcher on Windows, and the four release
targets. Hook commands call the binary by path, never through `sh`, because Cursor runs hooks through PowerShell
on Windows (M2, #58).

The Cursor editor has no status line. With the status line option on, the CLI alone runs
`tightship status-line render` through `statusLine` in `~/.cursor/cli-config.json` (M2, #58). The launcher
hands the binary that command, with `${CURSOR_PLUGIN_ROOT}` left for the CLI to expand, in
`TIGHTSHIP_STATUS_LINE_COMMAND` (#78). The CLI runs hooks in the plugin's folder, so the launcher sets
`TIGHTSHIP_PROJECT_POINTER` to `/workspace_roots/0`, where the hook's input names the project.

Companion plugins have no `dependencies` field in Cursor's manifest. The unified installer copies each
companion into `~/.cursor/plugins/local/<name>/` before `tightship` (M2, #58; #85).

**It never** gets edited. Every change goes to the canonical content or `harnesses/cursor/harness.toml`, then
through the build.

**To try it:** copy `dist/cursor/` to `~/.cursor/plugins/local/tightship/` and reload the window, or
`agent --plugin-dir dist/cursor`. **To install it:** follow `docs/install/cursor/`.
