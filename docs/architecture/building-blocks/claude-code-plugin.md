# Claude Code plugin

**Lives in:** `dist/claude-code/`, generated and ignored by git · **Milestone:** M1, epics #1 to #6

The `tightship` plugin as Claude Code loads it: `.claude-plugin/plugin.json`, one `skills/<name>/` per
skill and per command, and `agents/<name>.md` for the subagent. Commands ship as skills, because Claude
Code documents its `commands/` folder as legacy and runs a skill as a command.

The same folder is the plugin's catalog, `.claude-plugin/marketplace.json`, with the plugin's source `./`
(M1, #2). The manifest declares the three companion plugins as dependencies, each from its own catalog, and
the catalog allows those catalogs, so Claude Code installs the companions with the plugin and reuses any the
adopter already has. The manifest also declares the [options](../concepts/options.md) as `userConfig`, each a
boolean on by default (M1, #3).

Its first hooks carry the [contract](../concepts/contract.md): `hooks/hooks.json` runs one `sh` line per file
of `contract/` at `SessionStart`, each printing its part unless its options are off (M1, #4).

Its `bin/` holds the [tightship binary](tightship-binary.md) and its launcher, and its memory hooks run it at
`SessionStart` and on each prompt, unless the memory option is off (M1, #5). The manifest declares
`vault_folder`, a `directory` setting with no default, next to the options.

One more `SessionStart` hook, on `startup`, runs `tightship hook status-line` whatever the status line option's
state, since turning the option off is what puts the adopter's status line back. A plugin cannot set
`statusLine` itself, so the binary writes it into the adopter's user settings after consent
([status line](../concepts/status-line.md), M1, #7).

Every hook line that runs the binary, and the status line command, calls the launcher through `sh`, so an
install that drops the executable bit still runs it (M1, #8).

**It never** gets edited. Every change goes to the canonical content or the harness definition, then
through the build.

**To try it:** `claude --plugin-dir dist/claude-code`. **To install it:** add the companions' catalogs, then
`claude plugin marketplace add ./dist/claude-code` and `claude plugin install tightship@tightship`. Epic #8
moves the catalog to the repository root, pointing at the release's zip.
