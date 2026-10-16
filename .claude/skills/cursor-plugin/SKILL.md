---
name: cursor-plugin
description: Use when building, rendering, or debugging the Cursor plugin of this repository — Cursor's plugin layout, what each piece and hook does in the editor and in the `agent` CLI, how hooks run on Windows, and how to probe a behavior the docs do not settle.
---

# Cursor plugin

What Cursor does with a plugin, as observed by probing, not as its docs promise. The docs are wrong or silent
on several points below, so a fact marked **docs** is a hypothesis until a probe confirms it.

Probed on 2026-10-15, Windows 11: editor 3.25.6, CLI `agent` 2026.10.14-941492e. On 2026-10-16, Linux
(maintainer dev, editor 3.25.6) confirmed Unix hook execution and the 10 000-character kit payload (issue
65). Plugin **code** is cross-checked in CI (`rust.yml` on ubuntu, macos, windows); hook behavior in the
editor is documented from the maintainer's dev machine plus the Windows probe, not from manual sessions on
every OS. Re-probe a fact before relying on it at a newer version.

## Probe checklist (#65)

Issue 65 is **closed** when this table matches what you measured. Git marketplace install E2E is tasks 72
and 75, not this task.

| Platform | Hooks (shell, paths, JSON stdout) | `additional_context` size cap |
| --- | --- | --- |
| Windows | Done 2026-10-15: PowerShell 7 wrapper; `${CURSOR_PLUGIN_ROOT}/bin/…` resolves `.cmd` | **10000** (M2 contract limit; kit payload matches spec) |
| Linux | Done 2026-10-16: `${CURSOR_PLUGIN_ROOT}/bin/…` runs directly (no `pwsh`); `workspace_roots` absolute paths | **10000** (kit `limit-10000.json` prints one line, 10 000 chars inside `additional_context`) |

## Layout

```text
<plugin>/
  .cursor-plugin/plugin.json   name (kebab-case, required), version, description, author{name}, variables
  skills/<name>/SKILL.md       front matter: name (= folder), description
  commands/<name>.md           front matter: name, description; $ARGUMENTS is replaced
  agents/<name>.md             front matter: name, description, model, readonly
  rules/<name>.mdc | .md       front matter: description, alwaysApply, globs
  hooks/hooks.json             {"version": 1, "hooks": {"<event>": [{"command": "..."}]}}
```

A path field in the manifest replaces the default folder; it does not add to it (**docs**). The manifest has
no field for dependencies, so companion plugins are installed by hand (probed 2026-10-15: no `dependencies`
key in `.cursor-plugin/plugin.json`).

The catalog is `.cursor-plugin/marketplace.json` at a git repository's root. Adopters add it with
`agent plugin marketplace add <owner>/<repo>` (same shape as `claude plugin marketplace add`, **docs** until
E2E probe) or from the editor under Customize → Plugins. Install companions before tightship; see
`docs/install/cursor/companions.md`. Every documented source is git.

## Loading a plugin to test it

- **Editor and CLI:** copy the plugin to `~/.cursor/plugins/local/<name>/`. The editor shows it as **Local**
  under Customize → Plugins, after a reload. The CLI loads it too.
- **CLI only:** `agent --plugin-dir <folder>`.
- **Non-interactive:** `agent -p --trust "<prompt>"`. `-p` skips some hooks (below), so confirm a hook's
  behavior in an interactive session too.
- **Hook probe kit:** `maintainer/cursor-hook-probe/` (issue #65). Same `${CURSOR_PLUGIN_ROOT}/bin/…` command
  shape as tightship; logs under `.cursor-hook-probe/` in the workspace.

## What each piece does

| Piece | Editor | CLI |
| --- | --- | --- |
| Skill | Listed and loaded, with no plugin prefix: `<skill>`, never `<plugin>:<skill>` | Same |
| Command | `/<command> <args>`; `$ARGUMENTS` becomes the arguments | Same |
| Subagent | Dispatched by the `Task` tool with `subagent_type` = its name | Same |
| `readonly: true` | **Not enforced:** the subagent edited a file | **Not enforced**, with or without `--force` |
| Rule with `alwaysApply` | Loaded, `.mdc` and `.md` | **Not loaded** |
| `bin/` | Not probed | **Not on the agent's shell `PATH`** |

The agent's tools in the CLI include `Shell`, `Read`, `Write`, `StrReplace`, `Grep`, `Glob`, `Delete`, `Task`,
`TodoWrite`, `WebFetch`, `WebSearch`, and `SwitchMode`. Plan mode exists in both (`--plan` in the CLI).

## Hooks

### How a command runs

On Windows, the editor and the CLI run every command hook through PowerShell 7:

```text
pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -c "... Get-Content <payload.json> -Raw | & { $input | <command> }"
```

- The command is PowerShell, not `sh`. `sh` is not on the Windows `PATH`, so `sh "<file>"` fails.
- A command that is only a path, `${CURSOR_PLUGIN_ROOT}/bin/<name> <args>`, resolves `<name>.cmd` through
  `PATHEXT` on Windows (probed). On Linux the same string runs the executable without `.cmd` (probed
  2026-10-16; `parent_comm` is not `pwsh` or `powershell`).
- Never wrap the path in `sh`; that is the form tightship ships in `hooks/hooks.json`.
- Contract and memory hooks must print **one line** of JSON on stdout, for example
  `{"additional_context":"<text>"}`. Multi-line stdout is untested.
- **Trap:** the CLI hands that PowerShell text to the agent's shell. Started from Git Bash, where `SHELL` is
  set, the shell is bash and every hook fails with `syntax error near unexpected token '&'`. A failing
  `beforeShellExecution` hook then blocks every shell call. Probe from PowerShell with `$env:SHELL = $null`.
- `${CURSOR_PLUGIN_ROOT}` and `${CLAUDE_PLUGIN_ROOT}` are replaced in the command and exported to it.
- The working directory is the workspace root.

### Environment and input

The environment holds `CURSOR_PLUGIN_ROOT`, `CLAUDE_PLUGIN_ROOT`, `CURSOR_PROJECT_DIR`, `CLAUDE_PROJECT_DIR`,
`CURSOR_VERSION`, and `CURSOR_USER_EMAIL`. No variable names a data folder for the plugin.

Every event gets JSON on stdin with `conversation_id`, `generation_id`, `session_id`, `model`,
`hook_event_name`, `cursor_version`, `workspace_roots`, `user_email`, and `transcript_path`, which is `null`
at `sessionStart`. The editor writes a root as `/c:/Users/...` on Windows; the CLI writes `C:\Users\...`.
On Linux the editor uses normal absolute paths in `workspace_roots` (for example `/mnt/data/...`, probed
2026-10-16).

### Events

| Event | Fires | Its `additional_context` reaches the agent |
| --- | --- | --- |
| `sessionStart` | Once per conversation. **Not again after a compaction.** Not in Cloud agents (**docs**) | Yes, but the editor does not wait for it: a hook slower than the first request misses the first turn. Its `env` output never reaches the agent's shell |
| `beforeSubmitPrompt` | Every prompt, in the editor and the interactive CLI; **not under `agent -p`** | Yes in the editor, against the docs. Not confirmed in the CLI |
| `postToolUse` | After every tool call | Yes, in both |
| `preCompact` | Before a compaction; stdin has `trigger`, `context_tokens`, `context_window_size`, `is_first_compaction` | No: it returns only `user_message` |
| `stop`, `sessionEnd` | End of a turn, end of a conversation | Not probed |
| `beforeShellExecution` | Before each shell call; returns `{"permission": "allow"}` | — |

The `sessionStart` input has no `source` field, so a hook cannot tell a new conversation from a resumed one.

## Options: `variables`

The manifest's `variables` is a JSON Schema whose properties the editor shows in the plugin's **Configure**
dialog, filled with each `default`. **Hooks cannot read them:** the CLI replaces `${<NAME>}` in a hook command
with an empty string, and the editor leaves it literal. The saved value is in no local file. A plugin keeps
its options somewhere it can read itself.

## Plugins of the other harness

The editor and the CLI import the plugins installed in Claude Code, listed as **Imported**. Their skills and
subagents load unprefixed, next to the Cursor plugin's. A hook of theirs runs too, through the same PowerShell
wrapper on Windows: one written in `sh` produces nothing on Windows, one that runs a program does. A plugin
shipped for both harnesses loads twice in Cursor.

## Status line

None in the editor. The CLI runs the command of `statusLine` in `~/.cursor/cli-config.json` (**docs**, not
probed).

## Probing a behavior

1. Prefer `maintainer/cursor-hook-probe/` for hook shell, JSON stdout, 10 000-character `additional_context`,
   and `workspace_roots` shape (issue #65).
2. For other markers, build a throwaway plugin with one distinctive marker per piece and per hook, such as
   `RULE-MARK-2W`.
3. Ask the agent, in a new chat, which markers its context holds, and read `.cursor-hook-probe/` or your logs.
   The agent's report is a hint: check the transcript under `~/.cursor/projects/<workspace>/agent-transcripts/`.
4. A free plan runs out after a few dozen requests; spend them on what only the model can show.

## Not probed yet

- Whether `beforeSubmitPrompt`'s `additional_context` reaches the agent in the CLI.
- Whether `preToolUse` or `subagentStart` tells which subagent calls a tool, so a hook can deny edits to a
  read-only one.
- The CLI's `statusLine` format and uninstall.
