# Antigravity CLI plugin

**Lives in:** `dist/antigravity-cli/`, generated and ignored by git · **Milestone:** M3, epic #59

The `tightship` plugin as Antigravity CLI (`agy`) loads it: `plugin.json` and `hooks.json` at the plugin
root, one `skills/<name>/SKILL.md` per skill and per command, `skills/contract/SKILL.md` for reloading the
contract after compaction, and `agents/adversarial-verifier.md` for the read-only subagent. Commands ship as
skills, because Antigravity CLI has no `commands/` folder and exposes every skill in `skills/<name>/SKILL.md`
as a `/<name>` slash command.

Antigravity CLI rejects `userConfig` in `plugin.json` under `agy plugin validate --strict` and sets no
per-option environment variables, so the adopter's [options](../concepts/options.md) live in
`${PLUGIN_DATA}/options.toml` (`~/.gemini/config/plugin_data/tightship/options.toml`), which Antigravity CLI
preserves across updates.

Its `SessionStart` hooks in `hooks.json` run `tightship hook contract` for `contract/part-1.md` and
`contract/memory.md`, `tightship hook start`, and `tightship hook status-line`. Its `PreInvocation` hook runs
`tightship hook prompt`. Each hook calls `"${PLUGIN_ROOT}/bin/tightship"` directly: on macOS and Linux that
runs the POSIX `sh` launcher `bin/tightship`, and on Windows `cmd /c` resolves `bin/tightship.cmd`. Both
launchers set `TIGHTSHIP_ROOT`, `TIGHTSHIP_DATA`, `TIGHTSHIP_SETTINGS`, and `TIGHTSHIP_HOOK_OUTPUT` to the
plugin's `hook-output.json`, `{"injectSteps":[{"systemMessage":{"systemMessage":"{text}"}}]}`, before running
the platform binary, so non-empty hook output reaches Antigravity CLI as a system message.

**It never** gets edited. Every change goes to the canonical content or the harness definition, then through
the build.

**To validate it:** `agy plugin validate dist/antigravity-cli --strict`. **To install it:** run `install.sh`
or `install.ps1`, or follow `docs/install/antigravity-cli/`.
