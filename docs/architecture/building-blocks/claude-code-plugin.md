# Claude Code plugin

**Lives in:** `dist/claude-code/`, generated and ignored by git · **Milestone:** M1, epic #1

The `tightship` plugin as Claude Code loads it: `.claude-plugin/plugin.json`, one `skills/<name>/` per
skill and per command, and `agents/<name>.md` for the subagent. Commands ship as skills, because Claude
Code documents its `commands/` folder as legacy and runs a skill as a command.

**It never** gets edited. Every change goes to the canonical content or the harness definition, then
through the build.

**To try it:** `claude --plugin-dir dist/claude-code`. Installing it from a release is epic #2 and #8.
