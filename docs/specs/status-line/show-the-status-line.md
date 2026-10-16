# #7 — Show the workflow's status line, with the adopter's consent

> **Status:** published
> **Profile:** API
> **Module:** `tightship/`, `xtask/`
> **Epic:** #7
> **Requirements:** FR-STL-01, FR-STL-02, FR-STL-03
> **Sprint:** 4

The `tightship` binary gains the baseline's status line, `tightship status-line render`. Claude Code lets a
plugin set no status line of its own, so the binary writes one into the adopter's user settings, and only
after the adopter says yes. A session-start hook has the agent ask once. Turning the `status_line` option off
puts back the status line the adopter had, or none when they had none.

Terms from the SRS, as used here:

- **Status line**: the line a harness shows under the prompt with the session's state
  ([glossary](../../requirements/glossary.md)).
- **Baseline**: commit `1f41699` of the maintainer's private configuration. Its status line is the Bun script
  `statusline-command.ts`, which the baseline's setup wrote into the user settings as
  `{"type": "command", "command": "bun \"<folder>/statusline-command.ts\""}`.
- **Option**: a part of the plugin the adopter turns on or off. This spec reads `status_line` by rule 4 of the
  [epic #3 spec](../options/choose-the-options.md): off only when the value is exactly `false`.
- **Adopter**: whoever installs the plugin, the maintainer included
  ([overview](../../requirements/overview.md#stakeholders)).
- **User settings**: Claude Code's `settings.json` in its configuration folder, `$CLAUDE_CONFIG_DIR` when it
  is set and `~/.claude` otherwise.

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-15. They rest on Claude Code 2.1.299 and its plugin, settings, hooks,
and status line references:

1. **The binary writes `statusLine` into the user settings.** A plugin's `settings.json` and its manifest take
   only `agent` and `subagentStatusLine`, and drop every other key. So `tightship status-line enable` writes
   `statusLine`, after consent, into the user settings only, never into a project's or a folder's. Claude Code
   reloads the file while the session runs, so the status line shows at once.
2. **The agent asks.** No hook can put a question to the adopter. A `SessionStart` hook on `startup` prints,
   for the agent, the question to ask word for word and the two commands to run on yes and on no. A no is
   recorded, and the hook does not ask again until the option is turned off and on.
3. **Turning it off is turning the `status_line` option off**, with the mechanism of epic #3
   ([FR-INS-04](../../requirements/functional/install.md#fr-ins-04)). The next session start puts the previous
   value back exactly, or removes the key when there was none. The previous value waits in the plugin's data
   folder.
4. **Uninstalling needs the option off first.** Claude Code runs no plugin code on uninstall, and `--prune`
   deletes the data folder with the previous value in it. The install guide (#8) tells the adopter to turn
   `status_line` off and open one session before uninstalling. [FR-INS-06.1](../../requirements/functional/install.md#fr-ins-06)
   changes to say so, by the change request #31.
5. **Every session start points the status line at the current plugin folder.** Claude Code installs each
   version of a plugin in its own folder, so the command written by `enable` would break at the next update.
   The hook rewrites the command when it is ours and names another folder.
6. **`render` ports the baseline's script.** Same two rows, segments, colors, and cut by priority. Two
   differences: the reset times print as `HH:MM` and `DD/MM HH:MM`, the shape the baseline's `pt-BR` format
   gave, with no locale; and without `COLUMNS` the rows print whole, where the baseline stopped with an error.
7. **Two dependency changes, which amend ADR 0002.** `unicode-width` 0.2.2 measures a row as a terminal shows
   it, as `Bun.stringWidth` did. `serde_json` gains its `preserve_order` feature, already on in `xtask/`, so
   rewriting the user settings keeps the order of the adopter's keys.

## Acceptance Criteria

### Contract

| Command | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| `cargo xtask build` | Maintainer | Yes | Also writes the status line's hook into `dist/claude-code/hooks/hooks.json` |
| A session start (`startup`) | Adopter, through Claude Code | No: it may write the user settings and the record | Offers the status line, points it at the current plugin folder, or puts the previous one back |
| `tightship status-line enable` | Agent, after the adopter said yes | Yes | Saves the current `statusLine` and writes the workflow's |
| `tightship status-line decline` | Agent, after the adopter said no | Yes | Records the no |
| `tightship status-line render` | Claude Code, as the status line command | Yes | Prints the status line for the JSON on stdin |
| `tightship --help` | Agent | Yes | Prints the usage, which gains the three `status-line` lines |

### Request

#### The hook's command line

Rendered by the build for Claude Code, after the memory hooks of `SessionStart`, with the matcher `startup`.
It runs whatever the option's state, since turning the option off is what puts the previous value back:

```sh
TIGHTSHIP_STATUS_LINE="$CLAUDE_PLUGIN_OPTION_STATUS_LINE" TIGHTSHIP_SETTINGS="${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json" TIGHTSHIP_DATA="$CLAUDE_PLUGIN_DATA" TIGHTSHIP_ROOT="$CLAUDE_PLUGIN_ROOT" "${CLAUDE_PLUGIN_ROOT}/bin/tightship" hook status-line
```

#### Environment

The binary reads only these, and names no harness:

| Variable | Type | Required | From | Read by |
| --- | --- | --- | --- | --- |
| `TIGHTSHIP_STATUS_LINE` | string | No: absent when the adopter never set the option | The `status_line` option | `hook status-line` |
| `TIGHTSHIP_SETTINGS` | absolute path | Yes | The user settings file | `hook status-line`, `status-line enable` |
| `TIGHTSHIP_DATA` | absolute path | Yes | The plugin's data folder | `hook status-line`, `status-line enable`, `status-line decline` |
| `TIGHTSHIP_ROOT` | absolute path | Yes | The plugin's folder for the installed version | `hook status-line`, `status-line enable` |
| `COLUMNS` | positive integer | No | Claude Code, the terminal's width | `status-line render` |

The agent's shell does not carry the first four: `enable` and `decline` get them on the command line the hook
prints.

#### `status-line render` stdin

Claude Code's status line JSON. `render` reads these fields; a field absent, `null`, or of another type counts
as absent, and every other field is ignored:

| Field | Type | Feeds |
| --- | --- | --- |
| `workspace.current_dir` | string | The folder segment, and the folder the branch is read from |
| `workspace.repo.owner`, `workspace.repo.name` | string | The repository segment, only when both are present |
| `pr.number` | integer | The pull request segment |
| `pr.review_state` | string | The pull request's state; `open` when absent |
| `worktree.name` | string | The worktree segment |
| `agent.name` | string | The agent segment |
| `model.display_name` | string | The model segment |
| `effort.level` | string | The effort segment |
| `context_window.used_percentage` | number | The context gauge |
| `context_window.total_input_tokens`, `context_window.context_window_size` | integer | The context gauge's detail, only when both are present |
| `rate_limits.five_hour.used_percentage`, `rate_limits.seven_day.used_percentage` | number | The 5-hour and 7-day gauges |
| `rate_limits.five_hour.resets_at`, `rate_limits.seven_day.resets_at` | integer, Unix seconds | Each gauge's detail |

```json
{
  "workspace": { "current_dir": "/home/adopter/tightship", "repo": { "owner": "ATNexusLab", "name": "tightship" } },
  "pr": { "number": 31, "review_state": "approved" },
  "model": { "display_name": "Claude Opus 5.5 (1M context)" },
  "effort": { "level": "medium" },
  "context_window": { "used_percentage": 11.4, "total_input_tokens": 109000, "context_window_size": 1000000 },
  "rate_limits": {
    "five_hour": { "used_percentage": 4, "resets_at": 1792086600 },
    "seven_day": { "used_percentage": 66, "resets_at": 1792249200 }
  }
}
```

### Response

#### `render` stdout

Up to two rows, each ended by a newline: the identity row, then the usage row. A row with no segment prints
nothing. Segments are joined by a space, `·` in dim, and a space. For the input above, `COLUMNS=200`, `TZ=UTC`,
and a checkout of `feat/7-status-line`, stdout with the color codes removed is:

```text
tightship · ATNexusLab/tightship · feat/7-status-line · PR #31 (approved) · Opus 5.5 · medium
ctx █░░░░░ 11% 109k/1M · 5h ░░░░░░ 4% 17:50 · 7d ████░░ 66% 17/10 15:00
```

A colored text is `ESC[<code>m`, the text, then `ESC[0m`:

| Segment | Text | Color | Priority |
| --- | --- | --- | --- |
| Folder | The last component of `workspace.current_dir` | cyan `36` | 1 |
| Branch | The branch `HEAD` names, by rule 8 | yellow `33` | 2 |
| Context gauge | `ctx`, the gauge, and in dim `<input>/<size>` | by rule 9 | 3 |
| Model | `model.display_name` without its leading `Claude` and space, and without a trailing space and `(…)` | bold white `1;37` | 4 |
| 5-hour gauge | `5h`, the gauge, and in dim the reset as `HH:MM` | by rule 9 | 5 |
| 7-day gauge | `7d`, the gauge, and in dim the reset as `DD/MM HH:MM` | by rule 9 | 6 |
| Pull request | `PR #<number> (<state>)` | green `32` when `approved`, red `31` when `changes_requested`, yellow `33` otherwise | 7 |
| Effort | `effort.level` | `low` dim `2`, `medium` yellow `33`, `high` cyan `36`, `xhigh` and `max` red `31`, any other white `37` | 8 |
| Repository | `<owner>/<name>` | dim white `2;37` | 9 |
| Worktree | `wt:<name>` | dim cyan `2;36` | 10 |
| Agent | `agent:<name>` | magenta `35` | 11 |

The identity row holds, in this order: folder, repository, branch, pull request, worktree, agent, model,
effort. The usage row: context, 5-hour, 7-day.

#### The user settings after `enable`

The `statusLine` key, where it was or at the end when it is new, and every other key as it was:

```json
{
  "statusLine": {
    "type": "command",
    "command": "\"/home/adopter/.claude/plugins/cache/tightship/tightship/0.1.0/bin/tightship\" status-line render"
  }
}
```

#### `<TIGHTSHIP_DATA>/status-line.json`

| Key | Type | Required | Domain |
| --- | --- | --- | --- |
| `state` | string | Yes | `on` or `declined` |
| `previous` | any JSON value | Only with `on`, and only when the adopter had a `statusLine` | The adopter's `statusLine` before `enable`, kept as it was |

```json
{ "state": "on", "previous": { "type": "command", "command": "bun \"/home/adopter/.claude/statusline-command.ts\"" } }
```

#### What the hook prints

Only when it offers the status line (rule 3), for the agent. Each `<…>` value is wrapped in `'`, each `'` in
it written as `'\''`:

```text
tightship can show its status line in this session. In your first reply, ask the adopter, word for word:
"tightship can show its status line under the prompt. It replaces the status line in your user settings, and turning the plugin's status_line option off brings yours back. Turn it on?"
On yes, run: TIGHTSHIP_SETTINGS=<settings> TIGHTSHIP_DATA=<data> TIGHTSHIP_ROOT=<root> tightship status-line enable
On no, run: TIGHTSHIP_DATA=<data> tightship status-line decline
```

#### What `enable` and `decline` print

| Case | Stdout | Exit |
| --- | --- | --- |
| `enable` wrote the status line | `tightship's status line is on. Turn the plugin's status_line option off to bring back the one you had.` | `0` |
| `enable` found it already on | `tightship's status line is already on.` | `0` |
| `decline` recorded the no | `The status line stays as it is. tightship will not ask again until its status_line option is turned off and on.` | `0` |
| `decline` found the status line on | `tightship's status line is on; turn the plugin's status_line option off to bring back the one you had.` | `0` |

#### Usage

The usage text gains three lines, at its end, after the handoff's:

```text
       tightship status-line render
       tightship status-line enable
       tightship status-line decline
```

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Adopter | Their own user settings | Says yes or no, and turns the option off to undo |
| Agent | The adopter's shell | Runs `enable` or `decline`, only on the adopter's answer |
| Maintainer | Write access to the repository | Builds the plugin |

---

## Business Rules

### 1. Authorization

N/A. The binary runs as the adopter, on their machine, against their own files, with no service behind it.

### 2. Our status line is the one `enable` writes

- **When:** every rule below that asks whether the status line is ours.
- `statusLine` is ours when it is an object whose `command` is a string ending in
  `/bin/tightship" status-line render`.
- The command for the current version is `"<TIGHTSHIP_ROOT>/bin/tightship" status-line render`.

### 3. The session start offers the status line once

- **When:** `hook status-line`, with the option on.
- `statusLine` not ours and no record: prints the offer, and changes nothing (FR-STL-02).
- A record of `declined`: prints nothing.
- A record of `on` with a `statusLine` that is no longer ours, because the adopter replaced it: the record
  becomes `declined`, the settings stay as they are, and it prints nothing.
- `statusLine` ours: rule 6.

### 4. `enable` saves, then writes

- **When:** `tightship status-line enable`.
- `statusLine` already ours prints the already-on line and changes nothing.
- Otherwise it writes the record `on`, with `previous` when the settings held a `statusLine`, then writes
  ours into the settings. A settings file that does not exist is created, holding only `statusLine`.
- When writing the settings fails, it deletes the record it wrote, so nothing changed.

### 5. `decline` records the no

- **When:** `tightship status-line decline`.
- A record of `on` prints its line and changes nothing.
- Otherwise it writes the record `declined`. The settings are never read or written (FR-STL-02.1).

### 6. The status line follows the plugin's folder

- **When:** `hook status-line`, with the option on and `statusLine` ours.
- A `command` other than the current version's is replaced by it; every other key of `statusLine` stays. It
  prints nothing.

### 7. Turning the option off puts the previous one back

- **When:** `hook status-line`, with the option off.
- `statusLine` ours, with a record holding `previous`: `statusLine` becomes `previous`, exactly
  (FR-STL-03.1).
- `statusLine` ours, with no `previous` or no record: the `statusLine` key is removed (FR-STL-03.2).
- `statusLine` not ours: the settings stay as they are.
- In every case the record is deleted, so turning the option back on offers the status line again. It prints
  nothing.

### 8. The branch comes from `HEAD`, without git

- **When:** `render`, with `workspace.current_dir` present.
- From that folder up to the root, the first folder holding `.git` is the repository. A `.git` file is a
  worktree's pointer: its `gitdir: <path>` line, relative to that folder, names the git folder.
- `HEAD` holding `ref: refs/heads/<branch>` gives `<branch>`. A detached `HEAD`, no repository, or a file that
  cannot be read gives no branch segment, and no error.
- When the branch has more than 20 characters and the identity row does not fit, the branch becomes its first
  19 characters and `…`, before rule 10 drops anything.

### 9. A gauge reads a percentage

- **When:** `render`, for each of the three gauges.
- The percentage rounds half away from zero to `<n>`. The bar is 6 characters: `round(n × 6 / 100)` of `█`,
  between 0 and 6, then `░` up to 6.
- `ctx`, the bar, and `<n>%` are green `32` under 50, yellow `33` from 50 to 79, and red `31` from 80.
- The context detail is each count in short form: from 1,000,000, millions to one decimal with `M`, the
  decimal dropped when it is 0 (`1M`, `1.5M`); from 1,000, thousands rounded with `k` (`109k`); below, the
  number.
- A reset prints in the machine's time zone. A gauge whose reset is absent prints no detail.

### 10. A row fits the terminal

- **When:** `render`, with `COLUMNS` a positive integer.
- The width is `COLUMNS` minus 4. While a row is wider and holds more than one segment, the segment with the
  highest priority number is dropped. Width is measured as the terminal shows it, color codes left out.
- With `COLUMNS` absent or not a positive integer, each row prints whole.

### 11. The settings are rewritten whole, in place

- **When:** every write to the user settings.
- The file is read as a JSON object; any other content fails with `SETTINGS_INVALID`, and nothing changes.
- It is written with 2-space indentation and a final newline, with every key in its order. The write goes to a
  temporary file beside the real one, then a rename over it. When `TIGHTSHIP_SETTINGS` is a symbolic link,
  the write goes to the file it points to, and the link stays.

### 12. A failure never blocks the session

- The binary never exits with 2. Every failure exits 1 with its message on stderr, which Claude Code shows
  without stopping the session.
- `hook status-line` adds at most 50 ms to the session start at the 95th percentile, over 20 starts, within
  the 500 ms of [NFR-PERF-01](../../requirements/non-functional/performance.md#nfr-perf-01).

### 13. Persistence and audit

- **Files changed:** the user settings' `statusLine` key, by `enable` and the hook. In the plugin's data
  folder, `status-line.json`, by `enable`, `decline`, and the hook. The build writes one more hook into
  `dist/claude-code/hooks/hooks.json`.
- **Audit:** N/A. The settings and the data folder are the adopter's; nothing records who changed them.
- **Events/integrations fired:** N/A. Nothing uses the network.

---

## Errors

Every message goes to stderr, prefixed by `error:` and a space, and exits 1.

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `STATUS_LINE_NOT_SET` | `1` | A required variable of the environment table is absent or empty | "`` <variable> is not set; run the exact command the session start printed ``" |
| `SETTINGS_UNREADABLE` | `1` | Rules 3, 4, 6, 7 | "`` cannot read the settings file <path>: <the system's message> ``" |
| `SETTINGS_INVALID` | `1` | Rule 11 | "`` the settings file <path> is not a JSON object; nothing changed ``" |
| `SETTINGS_UNWRITABLE` | `1` | Rule 11 | "`` cannot write the settings file <path>: <the system's message>; nothing changed ``" |
| `STATUS_RECORD_INVALID` | `1` | A `status-line.json` that is not valid JSON or breaks its table | "`` the status line record <path> is damaged; delete it and start a new session ``" |
| `STATUS_RECORD_UNWRITABLE` | `1` | Rules 3, 4, 5, 7 | "`` cannot write the status line record <path>: <the system's message> ``" |
| `STATUS_INPUT_INVALID` | `1` | `render`'s stdin is not a JSON object | "`` the status line input is not a JSON object: <the parser's message> ``", the message being `expected an object` for JSON of another type |

A settings file that does not exist is not `SETTINGS_UNREADABLE`: it reads as an empty object.

## Side Effects

- **Persistence:** the settings are replaced by one rename; the record is written whole by one write call.
- **Concurrency:** Claude Code may write the same settings file between the read and the rename, and that
  write is then lost. The window is the few milliseconds of one run, at a session start or on the adopter's
  answer, when Claude Code writes the file only on the adopter's own action.
- **Transaction:** none across the two files. `enable` writes the record first, so a settings file never holds
  our status line without the previous value saved, and it deletes the record when the settings write fails.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — Build the status line's hook (happy path)

```gherkin
Given the canonical content of develop
When the maintainer runs `cargo xtask build`
Then it exits 0
And dist/claude-code/hooks/hooks.json holds, under SessionStart with matcher "startup", the hook's command line
  of the Request section
```

### Scenario 2 — Render both rows (FR-STL-01.1, happy path)

```gherkin
Given the stdin of the Request section, COLUMNS=200, TZ=UTC
And workspace.current_dir is a git checkout of feat/7-status-line
When `tightship status-line render` runs
Then it exits 0
And stdout without its color codes is the two rows of the Response section
And each segment carries the color of the Response section's table
```

### Scenario 3 — The same segments as the baseline (FR-STL-01.1)

```gherkin
Given 6 stdin fixtures: the Request section's, one with every field absent, one with a pull request in
  changes_requested, one in a worktree with an agent, one at 85% context with 1,500,000 tokens, and one with a
  branch of 30 characters
When the baseline's statusline-command.ts and `tightship status-line render` each run on every fixture, with
  COLUMNS=60 and COLUMNS=200, TZ=UTC
Then both print the same segments in the same order, with the same colors, for all 12 runs
```

### Scenario 4 — A narrow terminal drops by priority (alternative)

```gherkin
Given the stdin of the Request section and COLUMNS=60
When `tightship status-line render` runs
Then the identity row holds the folder, the branch, and the model, and no repository or effort
And no row is wider than 56 columns
```

### Scenario 5 — A long branch is cut before anything drops (alternative)

```gherkin
Given a checkout of a branch of 30 characters and an identity row wider than COLUMNS minus 4
When `tightship status-line render` runs
Then the branch segment is its first 19 characters followed by "…"
```

### Scenario 6 — No field, no COLUMNS (alternative)

```gherkin
Given stdin "{}" and no COLUMNS
When `tightship status-line render` runs
Then it exits 0 and prints nothing
Given the stdin of the Request section and no COLUMNS
When `tightship status-line render` runs
Then both rows print whole
```

### Scenario 7 — Input that is not JSON (exception)

```gherkin
Given stdin "nope"
When `tightship status-line render` runs
Then it exits 1
And stderr starts with "error: the status line input is not a JSON object: "
```

### Scenario 8 — The session start offers it (FR-STL-02, happy path)

```gherkin
Given the option is absent, a settings file holding a statusLine of its own, and an empty data folder
When `tightship hook status-line` runs
Then it exits 0 and prints the offer of the Response section, with the three folders filled in
And the settings file is byte for byte as it was
And the data folder holds no status-line.json
```

### Scenario 9 — The adopter says yes (FR-STL-01.1, happy path)

```gherkin
Given a settings file holding {"model": "opus", "statusLine": {"type": "command", "command": "mine"}, "theme": "dark"}
When the agent runs `tightship status-line enable` with the three variables
Then it exits 0 and prints "tightship's status line is on. Turn the plugin's status_line option off to bring back the one you had."
And the settings file holds model, statusLine, theme, in that order, statusLine being ours for TIGHTSHIP_ROOT
And status-line.json holds {"state": "on", "previous": {"type": "command", "command": "mine"}}
```

### Scenario 10 — Yes, with no status line before (alternative)

```gherkin
Given no settings file
When the agent runs `tightship status-line enable`
Then the settings file holds only statusLine, ours
And status-line.json holds {"state": "on"}
When it runs again
Then it prints "tightship's status line is already on." and nothing changes
When the agent runs `tightship status-line decline`
Then it prints "tightship's status line is on; turn the plugin's status_line option off to bring back the one you had."
And nothing changes
```

### Scenario 11 — The adopter says no (FR-STL-02.1)

```gherkin
Given a settings file holding a statusLine of its own
When the agent runs `tightship status-line decline`
Then it exits 0 and prints "The status line stays as it is. tightship will not ask again until its status_line option is turned off and on."
And the settings file is byte for byte as it was
And the next `tightship hook status-line` with the option on prints nothing
```

### Scenario 12 — Off puts the previous one back (FR-STL-03.1)

```gherkin
Given scenario 9 has run
When `tightship hook status-line` runs with TIGHTSHIP_STATUS_LINE=false
Then the settings file holds {"model": "opus", "statusLine": {"type": "command", "command": "mine"}, "theme": "dark"},
  in that order
And status-line.json does not exist
```

### Scenario 13 — Off with none before (FR-STL-03.2)

```gherkin
Given scenario 10's first run, then a key "theme" added to the settings
When `tightship hook status-line` runs with TIGHTSHIP_STATUS_LINE=false
Then the settings file holds {"theme": ...} and no statusLine key
```

### Scenario 14 — Off and on again asks again (alternative)

```gherkin
Given scenario 11 has run
When `tightship hook status-line` runs with TIGHTSHIP_STATUS_LINE=false, then with it absent
Then the first prints nothing and deletes status-line.json
And the second prints the offer
```

### Scenario 15 — An update moves the plugin's folder (alternative)

```gherkin
Given our statusLine with "padding": 1, written for TIGHTSHIP_ROOT /plugins/0.1.0
When `tightship hook status-line` runs with TIGHTSHIP_ROOT /plugins/0.2.0 and the option absent
Then statusLine's command is "\"/plugins/0.2.0/bin/tightship\" status-line render"
And statusLine keeps "padding": 1, and it prints nothing
```

### Scenario 16 — The adopter replaced ours by hand (alternative)

```gherkin
Given status-line.json holds {"state": "on"} and the settings' statusLine is {"type": "command", "command": "theirs"}
When `tightship hook status-line` runs with the option absent
Then the settings file is byte for byte as it was
And status-line.json holds {"state": "declined"}, and it prints nothing
```

### Scenario 17 — The settings file is a symbolic link (alternative)

```gherkin
Given TIGHTSHIP_SETTINGS is a symbolic link to a file in another folder
When the agent runs `tightship status-line enable`
Then the link still points to that file, and that file holds our statusLine
```

### Scenario 18 — Settings that are not a JSON object (exception)

```gherkin
Given a settings file holding "[1, 2"
When `tightship status-line enable` runs
Then it exits 1 with "error: the settings file <path> is not a JSON object; nothing changed"
And neither the settings file nor the data folder changed
```

### Scenario 19 — Settings that cannot be written (exception)

```gherkin
Given a settings file in a folder the adopter cannot write
When `tightship status-line enable` runs
Then it exits 1 and stderr starts with "error: cannot write the settings file <path>: "
And status-line.json does not exist
```

### Scenario 20 — A damaged record, a missing variable (exception)

```gherkin
Given status-line.json holds {"state": "paused"}
When `tightship hook status-line` runs
Then it exits 1 with "error: the status line record <path> is damaged; delete it and start a new session"
And the settings file is byte for byte as it was
Given TIGHTSHIP_ROOT is absent
When `tightship status-line enable` runs
Then it exits 1 with "error: TIGHTSHIP_ROOT is not set; run the exact command the session start printed"
```

### Scenario 21 — A record that cannot be written (exception)

```gherkin
Given a data folder the adopter cannot write
When `tightship status-line decline` runs
Then it exits 1 and stderr starts with "error: cannot write the status line record <path>: "
```

### Scenario 22 — In a session, from offer to off (FR-STL-01.1, FR-STL-02.1, FR-STL-03.1)

```gherkin
Given an isolated Claude Code configuration whose user settings hold the baseline's statusLine
And tightship installed from dist/claude-code with no option set
When a session starts
Then the agent asks the offer's question word for word
When the adopter says yes
Then the status line under the prompt shows the segments of scenario 2 for that session, without a restart
When the adopter sets status_line to false and starts a new session
Then the user settings hold the baseline's statusLine exactly, and the status line under the prompt is the baseline's
```

### Scenario 23 — The time budget (NFR-PERF-01)

```gherkin
Given every option on, our statusLine in the settings, and a vault of 1,000 notes
When 20 sessions start
Then `tightship hook status-line` takes at most 50 ms at the 95th percentile
And the session start, every hook together, stays within 500 ms at the 95th percentile
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md), marked M1, #7:

- **Building blocks:**
  - tightship binary gains `status-line render|enable|decline` and `hook status-line`, in a new `status/`
    domain folder, and reads `TIGHTSHIP_STATUS_LINE`, `TIGHTSHIP_SETTINGS`, and `TIGHTSHIP_ROOT`.
  - Claude Code plugin gains the status line's `SessionStart` hook, which runs whatever the option's state.
- **Concepts:** a new page, status line, holds the consent, the record, and how turning the option off puts
  the previous value back.
- **Security:** the write to the harness's settings file, already listed, is narrowed to the `statusLine` key
  of the user settings; the plugin's data folder gains `status-line.json`.
- **Deployment:** "Uninstalling" gains the step of turning `status_line` off and opening one session first.
- **ADR 0002** is amended: the dependency table gains `unicode-width` 0.2.2, and `serde_json` gains
  `preserve_order`.

## Data model impact

Applied to [docs/data-model/](../../data-model/README.md), marked M1, #7:

- **Option:** `status_line` gains its effect: the `statusLine` key of the user settings, written by `enable`.
- **Status line record:** a new physical model, `status-line.json` in the plugin's data folder, with `state`
  and `previous`.
- **Invariant:** while the user settings hold our status line, the record holds `on`, and `previous` when the
  adopter had one.
- **Access patterns:** the session start reads the settings and the record; `enable`, `decline`, and the
  session start write them.

## Out of Scope

- **Restoring the status line on an uninstall without turning the option off first:** Claude Code runs no
  plugin code on uninstall. The install guide's step is epic #8's, and FR-INS-06.1 changes by the change
  request #31.
- **The status line in the Cursor editor:** [epic #58](../cursor-plugin/ship-for-cursor.md) (CLI only).
  **Antigravity CLI and Codex:** [OQ-08](../../requirements/open-questions.md#oq-08).
- **A project's or a folder's settings:** a `statusLine` there overrides ours, and the plugin never writes
  there. The adopter keeps that choice.
- **Segments beyond the baseline's:** cost, session name, vim mode, and the rest of Claude Code's input. FR-STL-01
  asks for the baseline's segments.
- **A `refreshInterval`:** the baseline set none, so the status line runs on Claude Code's events.
- **A `subagentStatusLine`:** the baseline had none.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on | Issue |
| --- | --- | --- | --- | --- | --- |
| 1 | Render the baseline's status line | `tightship/src/status.rs` and `tightship/src/status/render.rs` (new), `tightship/src/main.rs` (subcommand, usage), `tightship/src/failure.rs` (`STATUS_INPUT_INVALID`), `tightship/Cargo.toml` (`unicode-width`), `tightship/tests/status.rs` and its stdin fixtures (new), about 8 files | Scenarios 2 and 4 to 7 by tests; 3 by running both scripts in a scratch folder | — | #34 |
| 2 | Ask, turn on, and put back the status line | `tightship/src/status/settings.rs` and `tightship/src/hook/status_line.rs` (new), `tightship/src/hook.rs`, `tightship/src/main.rs`, `tightship/src/failure.rs`, `tightship/Cargo.toml` (`preserve_order`), `xtask/src/build/claude_code.rs` (the hook), `xtask/tests/` (hooks.json), `tightship/tests/status.rs`, about 9 files | Scenarios 1 and 8 to 21 by tests; 22 in a session on an isolated configuration; 23 measured | 1 | #35 |
