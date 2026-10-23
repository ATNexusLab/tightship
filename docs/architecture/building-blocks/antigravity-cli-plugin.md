# Antigravity CLI plugin

**Lives in:** `dist/antigravity-cli/`, generated and ignored by git · **Milestone:** M3, epic #59

The `tightship` plugin as Antigravity CLI (`agy`) loads it: `plugin.json` and `hooks.json` at the plugin
root, one `skills/<name>/SKILL.md` per skill and per command, `skills/contract/SKILL.md` for reloading the
contract after compaction, and `agents/adversarial-verifier.md` for the read-only subagent. Commands ship as
skills, because Antigravity CLI has no `commands/` folder and exposes every skill in `skills/<name>/SKILL.md`
as a `/<name>` slash command.

Antigravity CLI rejects `userConfig` in `plugin.json` under `agy plugin validate --strict` and sets no
per-option environment variables, so the adopter's [options](../concepts/options.md) live in
`~/.gemini/config/plugin_data/tightship/options.toml`, the plugin's data folder, which Antigravity CLI
preserves across updates.

Its `SessionStart` hooks in `hooks.json` run `tightship hook contract-part` for `part-1.md` and
`memory.md`, `tightship hook start`, `tightship hook status-line`, and `tightship hook update`, which
announces an update and starts the next check
([auto-update spec](../../specs/auto-update/update-the-plugin.md)). Since M4 (#118) no hook runs on a
prompt, so Antigravity CLI, which has no compaction hook, gets no checkpoint.

Antigravity CLI 1.4.2 runs a hook from the plugin's folder and gives it neither `PLUGIN_ROOT` nor
`PLUGIN_DATA`, and the plugin's `bin/` is not on its `PATH` (probed on macOS, #89). It runs the hook
under `sh -c` on macOS and Linux and `cmd /c` on Windows, where Go escapes every `"` of the command as `\"`,
which `cmd` cannot read. So each hook names the launcher relative to the plugin's folder, with no quotes and
no variables. No relative path reads the same in both shells, so the build writes two files: `hooks.json`
with `bin/tightship` and `hooks.windows.json` with `bin\tightship`, which `cmd` resolves to
`bin\tightship.cmd`. After `agy plugin install`, the installer moves `hooks.windows.json` over `hooks.json` on
Windows, Git Bash included, and deletes it elsewhere. Both launchers set `TIGHTSHIP_ROOT`, `TIGHTSHIP_DATA`,
`TIGHTSHIP_SETTINGS`, and `TIGHTSHIP_HOOK_OUTPUT` to the plugin's `hook-output.json`,
`{"injectSteps":[{"systemMessage":{"systemMessage":"{text}"}}]}`, before running the platform binary, so non-empty
hook output reaches Antigravity CLI as a system message. They also set `TIGHTSHIP_PROJECT_POINTER` to
`/workspacePaths/0`: Antigravity CLI runs hooks in the plugin's folder and names the project only in the hook's
input.

**It never** gets edited. Every change goes to the canonical content or the harness definition, then through
the build.

**To validate it:** `agy plugin validate dist/antigravity-cli --strict`. **To install it:** run `install.sh`
or `install.ps1`, or follow `docs/install/antigravity-cli/`.
