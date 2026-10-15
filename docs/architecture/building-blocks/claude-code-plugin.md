# Claude Code plugin

**Lives in:** `dist/claude-code/`, generated and ignored by git · **Milestone:** M1, epics #1 and #2

The `tightship` plugin as Claude Code loads it: `.claude-plugin/plugin.json`, one `skills/<name>/` per
skill and per command, and `agents/<name>.md` for the subagent. Commands ship as skills, because Claude
Code documents its `commands/` folder as legacy and runs a skill as a command.

The same folder is the plugin's catalog, `.claude-plugin/marketplace.json`, with the plugin's source `./`
(M1, #2). The manifest declares the three companion plugins as dependencies, each from its own catalog, and
the catalog allows those catalogs, so Claude Code installs the companions with the plugin and reuses any the
adopter already has. The manifest also declares the [options](../concepts/options.md) as `userConfig`, each a
boolean on by default (M1, #3).

**It never** gets edited. Every change goes to the canonical content or the harness definition, then
through the build.

**To try it:** `claude --plugin-dir dist/claude-code`. **To install it:** add the companions' catalogs, then
`claude plugin marketplace add ./dist/claude-code` and `claude plugin install tightship@tightship`. Epic #8
moves the catalog to the repository root, pointing at the release's zip.
