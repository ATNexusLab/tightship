# #3 — Choose, change, and keep the options

> **Status:** published
> **Profile:** API
> **Module:** `content/`, `xtask/`
> **Epic:** #3
> **Requirements:** FR-INS-03, FR-INS-04, FR-INS-05, FR-INS-06
> **Sprint:** 2

The plugin declares its three options, and the harness asks the adopter for each one, stores the answers,
keeps them across an update, and deletes them on uninstall. This epic builds the declaration and the rule
every piece follows to read an option. What each option turns on or off arrives with the epic that builds
that part.

Terms from the SRS, as used here:

- **Option**: a part of the plugin the adopter turns on or off: the contract, the memory layer, or the
  status line ([glossary](../../requirements/glossary.md),
  [FR-INS-03](../../requirements/functional/install.md#fr-ins-03)).
- **Adopter**: whoever installs the plugin, the maintainer included
  ([overview](../../requirements/overview.md#stakeholders)).
- **Companion plugins**: ponytail, i-have-adhd, and humanizer, installed as the plugin's dependencies by
  epic #2 ([BR-04](../../requirements/business-rules.md#br-04)).
- **User configuration**: Claude Code's `userConfig`, the options a plugin's manifest declares. Claude Code
  asks for them, stores them under `pluginConfigs` in the adopter's user settings, and passes each one to the
  plugin's hooks as `CLAUDE_PLUGIN_OPTION_<NAME>`.

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-14, after tests in an isolated Claude Code 2.1.299 configuration and
in one session:

1. The options are the harness's own user configuration: three booleans, `contract`, `memory`, and
   `status_line`, each with default `true`. Claude Code asks, stores, changes, keeps, and deletes them. The
   plugin ships no command and no file for them.
2. The plugin writes no option state. ADR 0001 said the state lives in the harness's persistent plugin
   data folder; it is amended to say the harness keeps it in its own user settings.
3. An option never set reaches no piece: a hook gets no `CLAUDE_PLUGIN_OPTION_<NAME>`, and a skill keeps the
   literal text `${user_config.<name>}`. So the piece that reads an option treats an absent value as on
   (FR-INS-03.2), and no skill reads an option.
4. The list of options is canonical, in `content/options.toml`. The Claude Code renderer turns it into
   `userConfig`; a later harness renders the same list its own way.
5. This epic delivers the options' state, and FR-INS-03.1 and FR-INS-04.1 are verified on that state. What
   each option does in a session is verified by its epic: the contract by #4, memory by #5 and #6, the status
   line by #7.
6. One task. The branch `feat/3-options` is cut from `feat/2-install-skills-commands-verifier`, because this
   epic's build extends epic #2's manifest; its PR opens after epic #2's PR is merged into `develop`.

## Acceptance Criteria

### Contract

The build keeps its command, its exit codes, and its output. The adopter uses Claude Code's own commands.

| Command | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| `cargo xtask build` | Maintainer | Yes | Writes `dist/claude-code/`, the manifest now with `userConfig` |
| `/plugin install tightship@tightship`, in a session | Adopter | Yes | Installs, then asks for each option not set yet |
| `claude plugin install tightship@tightship --config <name>=<true\|false>` | Adopter | Yes | Installs and sets each option passed; `--config` repeats |
| `/plugin configure tightship@tightship`, in a session | Adopter | Yes | Shows the options and saves the new values |
| `claude plugin configure tightship@tightship` | Adopter | Yes | Lists each option and whether it is set |
| `claude plugin configure tightship@tightship --values-stdin` | Adopter | Yes | Saves the values of a JSON object on stdin; options left out keep theirs |
| `claude plugin update tightship@tightship` | Adopter | Yes | Updates the plugin; the options keep their values |
| `claude plugin uninstall tightship@tightship --prune -y` | Adopter | Yes | Uninstalls, deletes the options and the plugin's data folder, and removes the companions it installed |
| `claude plugin marketplace remove <catalog>` | Adopter | Yes | Removes one catalog the adopter added; run for `tightship` and each companion |

### Request

#### `content/options.toml`

New, optional. One `[[options]]` table per option, in the order the harness shows them:

| Key | Type | Required | Validation |
| --- | --- | --- | --- |
| `name` | string | Yes | 1 to 64 characters: lowercase ASCII letters, digits, and `_`, starting with a letter; unique in the file |
| `title` | string | Yes | Shown as the option's label; 1 to 40 characters |
| `description` | string | Yes | Shown under the label; 1 to 120 characters |

```toml
[[options]]
name = "contract"
title = "Contract"
description = "Load the workflow's rules into every session"

[[options]]
name = "memory"
title = "Memory"
description = "Keep curated notes in your vault and hand session state to the next session"

[[options]]
name = "status_line"
title = "Status line"
description = "Show the workflow's status line, after asking before it changes your settings"
```

Any other key, a missing key, a value out of its domain, or a repeated `name` fails the build (rule 3). A
missing file or an empty one declares no option. The maintainer writes it by hand. It is canonical content:
the neutrality check of epic #1 reads it like every other file of `content/`, so it names no harness.

### Response

#### `dist/claude-code/.claude-plugin/plugin.json`

Gains `userConfig`, after epic #2's `dependencies`, one entry per option in the order of `content/options.toml`:

```json
{
  "name": "tightship",
  "version": "0.1.0",
  "description": "Skills, a command per step of the development loop, and a read-only verifier",
  "author": { "name": "ATNexusLab" },
  "repository": "https://github.com/ATNexusLab/tightship",
  "dependencies": ["ponytail@ponytail", "i-have-adhd@i-have-adhd", "humanizer@humanizer"],
  "userConfig": {
    "contract": {
      "type": "boolean",
      "title": "Contract",
      "description": "Load the workflow's rules into every session",
      "default": true
    },
    "memory": {
      "type": "boolean",
      "title": "Memory",
      "description": "Keep curated notes in your vault and hand session state to the next session",
      "default": true
    },
    "status_line": {
      "type": "boolean",
      "title": "Status line",
      "description": "Show the workflow's status line, after asking before it changes your settings",
      "default": true
    }
  }
}
```

| Key | From |
| --- | --- |
| `userConfig.<name>` | `name` of one `[[options]]` |
| `type` | `"boolean"`, for every option |
| `title`, `description` | The option's `title` and `description` |
| `default` | `true`, for every option (FR-INS-03.2) |

With no option, `plugin.json` has no `userConfig` key. The file count stays 28, and the build's success
output does not change:

```text
Built dist/claude-code/: 22 pieces (13 skills, 8 commands, 1 agent), 28 files.
Try it: claude --plugin-dir dist/claude-code
```

#### What the adopter's harness stores

Claude Code writes the options to its user settings, never to a project's or a folder's settings:

```json
{
  "pluginConfigs": {
    "tightship@tightship": { "options": { "contract": true, "memory": false } }
  }
}
```

An option the adopter never set has no key there.

#### What a piece reads

| Where | Form | Set | Never set |
| --- | --- | --- | --- |
| Hook process | Environment variable `CLAUDE_PLUGIN_OPTION_<NAME>`, `<NAME>` uppercased | `true` or `false` | Absent |
| Skill or agent body | `${user_config.<name>}` | `true` or `false` | The literal `${user_config.<name>}` |

#### What the adopter sees

Literal output of Claude Code 2.1.299, observed on 2026-10-14:

| Moment | Output |
| --- | --- |
| `claude plugin install` without `--config` | `3 userConfig options not yet set — run /plugin configure tightship@tightship in Claude Code, or pass --config KEY=VALUE.` |
| `claude plugin configure tightship@tightship`, nothing set | `Options for tightship (tightship@tightship):`, then one line per option, such as `❯ contract — Contract (optional, not set)` |
| `claude plugin configure … --values-stdin` | `Configuration saved. Restart Claude Code to apply it.` |
| `claude plugin uninstall … --prune -y` | `✔ Successfully uninstalled plugin: tightship (scope: user)`, then a line starting `Removed` that names the companions it installed |

Claude Code prints these messages, and a later version may word them differently. The spec quotes them so
the scenarios can be checked.

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | Edits `content/options.toml` and runs the build |
| Adopter | None from this project | Chooses the options in their own Claude Code configuration |

---

## Business Rules

### 1. Authorization

N/A. The build runs on a local clone, and the options live in the adopter's own configuration, with no
service of this project behind either.

### 2. Every option is declared on, as a boolean

- **When:** `cargo xtask build` renders `plugin.json`.
- Each `[[options]]` entry becomes one `userConfig` entry with `type` `"boolean"` and `default` `true`, in file
  order.

### 3. `content/options.toml` is valid

- **When:** while `build` reads `content/`, before it renders anything.
- A key other than `name`, `title`, and `description`, a missing key, or a value that is not a string fails
  with `OPTIONS_INVALID`.
- A `name` outside its domain fails with `OPTION_NAME_INVALID`; a `title` or `description` out of its length
  fails with `OPTION_TEXT_INVALID`; a `name` already declared fails with `OPTION_NAME_TAKEN`.
- Any of them leaves `dist/claude-code/` as it was, as every build error of epic #1 does.

### 4. An option never set is on

- **When:** a piece reads an option.
- The option is off only when its value is exactly `false`. An absent value, or any other, is on.
- A hook reads `CLAUDE_PLUGIN_OPTION_<NAME>`. No skill and no agent body uses `${user_config.<name>}`,
  because an option never set reaches it as literal text.

### 5. The harness owns the state

- Claude Code asks for the options not yet set when the adopter installs or enables the plugin in a
  session, and the shell's `claude plugin install` sets only what `--config` passes.
- A change is saved at once and reaches the hooks from the next session start.
- `claude plugin update` keeps every value. Uninstalling from the last scope deletes the values and
  `~/.claude/plugins/data/tightship-tightship/`.

### 6. Uninstall leaves nothing the plugin added

- `claude plugin uninstall tightship@tightship --prune -y` also removes each companion that epic #2's
  dependencies installed, and keeps a companion the adopter had installed before.
- The adopter removes the four catalogs they added with `claude plugin marketplace remove`.
- The status line setting is out of this rule: epic #7 writes it into the adopter's settings, and restores
  it when the status line is turned off (FR-STL-03).

### 7. Persistence and audit

- **Files changed:** `dist/claude-code/.claude-plugin/plugin.json`, by the build. On the adopter's machine,
  only `pluginConfigs` in the user settings, by Claude Code.
- **Audit:** N/A. Git records every change to `content/options.toml`; the adopter's settings are theirs.
- **Events/integrations fired:** N/A. Neither the build nor the options use the network.

---

## Errors

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `OPTIONS_INVALID` | `1` | Rule 3 | "`content/options.toml: <reason>`" |
| `OPTION_NAME_INVALID` | `1` | Rule 3 | "`content/options.toml: "<name>" is not an option name; use 1 to 64 lowercase letters, digits, and underscores, starting with a letter`" |
| `OPTION_TEXT_INVALID` | `1` | Rule 3 | "`content/options.toml: the <key> of "<name>" must be 1 to <limit> characters`" |
| `OPTION_NAME_TAKEN` | `1` | Rule 3 | "`content/options.toml: the option "<name>" is declared twice`" |

`<reason>` is the TOML parser's message, such as ``missing field `title` ``. `<key>` is `title` or
`description`, and `<limit>` is 40 or 120.

## Side Effects

- **Persistence:** the build replaces `dist/claude-code/` whole, as in epic #1; the file count does not
  change.
- **Concurrency:** N/A. One maintainer runs one build at a time; Claude Code serializes its own plugin
  commands.
- **Transaction:** the manifest is written into `dist/.claude-code.tmp/` with the rest of the plugin and
  published by the same rename.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — Build the options (happy path)

```gherkin
Given content/options.toml declares contract, memory, and status_line
When the maintainer runs `cargo xtask build`
Then it exits 0 and reports 28 files
And dist/claude-code/.claude-plugin/plugin.json holds "userConfig" with contract, memory, and status_line,
  in that order, each with type "boolean" and default true
And `claude plugin validate dist/claude-code` reports "Validation passed" with no warning
```

### Scenario 2 — Choose the options at install (FR-INS-03.1, happy path)

```gherkin
Given a Claude Code configuration with the four catalogs added and no plugin installed
When the adopter runs `claude plugin install tightship@tightship --config memory=false`
Then `claude plugin configure tightship@tightship --json` reports memory "false" among "configured"
And the user settings hold "pluginConfigs" > "tightship@tightship" > "options" > "memory": false
```

### Scenario 3 — The harness could not ask (FR-INS-03.2, alternative)

```gherkin
Given the adopter ran `claude plugin install tightship@tightship` with no --config
Then Claude Code reports "3 userConfig options not yet set"
And the user settings hold no option of tightship@tightship
And by rule 4 each option is on
```

### Scenario 4 — Choose in the session's dialog (FR-INS-03.1, alternative)

```gherkin
Given the adopter installs tightship@tightship from `/plugin` in a session
When the dialog shows "Contract", "Memory", and "Status line" and the adopter turns "Memory" off
Then the user settings hold "memory": false for tightship@tightship
```

### Scenario 5 — Change an option later (FR-INS-04.1)

```gherkin
Given tightship@tightship is installed with memory on
When the adopter pipes {"memory":"false"} to `claude plugin configure tightship@tightship --values-stdin`
Then Claude Code reports "Configuration saved. Restart Claude Code to apply it."
And the user settings hold "memory": false and keep every other option as it was
And a hook of the plugin in the next session reads CLAUDE_PLUGIN_OPTION_MEMORY=false
```

### Scenario 6 — An update keeps the options (FR-INS-05.1)

```gherkin
Given tightship@tightship is installed with memory set to false
When a new version is published to its catalog and the adopter runs `claude plugin update tightship@tightship`
Then the plugin reports the new version
And the user settings still hold "memory": false for tightship@tightship
```

### Scenario 7 — Uninstall leaves nothing (FR-INS-06.1)

```gherkin
Given tightship@tightship is installed with its three companions and every option set
And ponytail@ponytail was installed by the adopter before tightship
When the adopter runs `claude plugin uninstall tightship@tightship --prune -y`
And removes the catalogs tightship, i-have-adhd, and humanizer with `claude plugin marketplace remove`
Then `claude plugin list` shows ponytail@ponytail and no other plugin
And the user settings hold no key naming tightship, i-have-adhd, or humanizer
And ~/.claude/plugins/data/tightship-tightship/ does not exist
```

### Scenario 8 — An option misses a key (exception)

```gherkin
Given content/options.toml holds an [[options]] entry with name "memory" and no title
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/options.toml: missing field `title`"
And dist/claude-code/ is as it was
```

### Scenario 9 — An option name out of its domain (exception)

```gherkin
Given content/options.toml declares an option named "status-line"
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/options.toml: \"status-line\" is not an option name; use 1 to 64 lowercase
  letters, digits, and underscores, starting with a letter"
```

### Scenario 10 — A description too long (exception)

```gherkin
Given content/options.toml declares "memory" with a description of 121 characters
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/options.toml: the description of \"memory\" must be 1 to 120 characters"
```

### Scenario 11 — An option declared twice (exception)

```gherkin
Given content/options.toml declares "memory" twice
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/options.toml: the option \"memory\" is declared twice"
```

### Scenario 12 — An option names the harness (exception)

```gherkin
Given content/options.toml describes "memory" as "Keep notes in ~/.claude"
When the maintainer runs `cargo xtask build`
Then it exits 1 with epic #1's HARNESS_TERM error, naming content/options.toml and its line
```

### Scenario 13 — No option (alternative)

```gherkin
Given content/ has no options.toml
When the maintainer runs `cargo xtask build`
Then it exits 0
And plugin.json has no "userConfig" key
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md), marked M1, #3:

- **Building blocks:**
  - Canonical content gains `options.toml`, the list of options.
  - Claude Code plugin declares the options as `userConfig`.
- **Concepts:** a new page, options, holds rule 4: how a piece reads an option.
- **Security:** the plugin no longer writes the options' state; the harness keeps it in its user settings.
- **Deployment:** "Installing" gains choosing and changing the options, and the uninstall that leaves
  nothing.
- **ADR 0001** is amended: the plugin writes in two places, the vault and, for the status line, the
  harness's settings.

## Data model impact

Applied to [docs/data-model/](../../data-model/README.md), marked M1, #3:

- **Option:** its physical model, a boolean per option under `pluginConfigs` in the harness's user
  settings, absent until the adopter sets it.
- **Invariant:** an absent option is on.
- **Access patterns:** reading an option is a hook reading its environment; changing one is the harness's
  own command.

## Out of Scope

- **What each option turns on or off in a session:** the contract by #4, memory by #5 and #6, the status line
  by #7. Each epic reads its option by rule 4.
- **Removing the status line setting on uninstall:** epic #7, with FR-STL-03.
- **The install guide's options, update, and uninstall sections** (FR-INS-08): epic #8.
- **Options in Cursor, Antigravity CLI, and Codex:** M2 to M4; each renders `content/options.toml` its own
  way.
- **An option's value other than on or off:** the SRS defines only the two states.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on | Issue |
| --- | --- | --- | --- | --- | --- |
| 1 | Declare the options and let the harness keep them | `content/options.toml` (new), `xtask/src/options.rs` (new: read and check), `xtask/src/content.rs` (accept the file), `xtask/src/claude_code.rs` (`userConfig`), `xtask/src/main.rs`, `xtask/tests/build.rs` and its fixture in `xtask/tests/common/`, about 7 files; built on epic #2's #14, which writes `dependencies` | Scenarios 1 and 8 to 13 by tests; 2, 3, 5 to 7 by an install in an isolated Claude Code configuration, the hook of scenario 5 being a throwaway one in a copy of `dist/claude-code/`; 4 in a session | — | #15 |
