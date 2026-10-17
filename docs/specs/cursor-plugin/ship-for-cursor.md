# #58 — Ship the workflow as a Cursor plugin

> **Status:** published
> **Profile:** API
> **Module:** `harnesses/cursor/`, `xtask/`, `tightship/`, `docs/install/cursor/`
> **Epic:** #58
> **Requirements:** NFR-FLEX-02, NFR-FLEX-07, NFR-COMP-01, FR-INS-01, FR-INS-02, FR-INS-03, FR-INS-04,
> FR-INS-05, FR-INS-06, FR-INS-07, FR-INS-08, FR-SKL-01, FR-SKL-02, FR-SKL-03, FR-VER-01, FR-VER-02,
> FR-CTR-01, FR-CTR-02, FR-CTR-03, FR-CTR-04, FR-CTR-05, FR-MEM-01 to FR-MEM-11, FR-STL-01, FR-STL-02,
> FR-STL-03, NFR-REL-01, NFR-PERF-01, NFR-PERF-02, NFR-SEC-01, NFR-SEC-02, BR-02, BR-03, BR-04, BR-05
> **Sprint:** 2

An adopter installs the workflow as a native **Cursor Plugin** (`.cursor-plugin/plugin.json`), in the editor
and in the `agent` command-line agent, and gets the same contract, skills, commands, verifier, options,
memory layer, and handoff behavior as in Claude Code, from one more folder under `harnesses/cursor/` and one
more renderer in `xtask`, without changing a line of `content/`.

Terms from the SRS, as used here:

- **Adopter**: whoever installs the plugin ([overview](../../requirements/overview.md#stakeholders)).
- **Cursor editor**: the Cursor IDE where Agent Chat runs.
- **Cursor CLI**: the `agent` binary (observed as `2026.10.14` on 2026-10-15).
- **Companion plugins**: ponytail, i-have-adhd, and humanizer ([BR-04](../../requirements/business-rules.md#br-04)).
- **Probe**: a throwaway plugin and a session question, as in `.claude/skills/cursor-plugin/SKILL.md`.

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-16, on the probes in `.claude/skills/cursor-plugin/SKILL.md`
(2026-10-15 Windows 11 and 2026-10-16 Linux dev, editor 3.25.6, CLI `2026.10.14` on Windows), Cursor's
hooks and plugins references, the
sprint-2 plan in `docs/roadmap/`, and the five shape decisions confirmed in chat on 2026-10-16:

1. **One epic carries M2**, as [roadmap issue #58](https://github.com/ATNexusLab/tightship/issues/58) says:
   build, install, options, contract, memory, handoff, release for Cursor. This spec settles Cursor's part of
   the former OQ-04, OQ-05, OQ-06, and OQ-08, and removes OQ-07 from the SRS.
2. **`cargo xtask build` renders every harness** that has a `harnesses/<name>/harness.toml`, in sorted name
   order, into `dist/<name>/`, in one run. Failure in any harness leaves no `dist/` behind for that harness,
   as today for Claude Code alone.
3. **Cursor keeps skills, commands, and agents in separate folders.** Commands are `commands/<name>.md`, not
   skills. References render without a plugin prefix: skill `spec`, command `/spec`, subagent
   `adversarial-verifier` for the `Task` tool.
4. **Hook commands invoke the binary by path**, never through `sh`: `"${CURSOR_PLUGIN_ROOT}/bin/tightship"
   hook <subcommand>"`. On Windows, Cursor runs hooks through PowerShell 7; this form resolves `tightship.cmd`
   via `PATHEXT`. Shell wrappers that worked in Claude Code are invalid in Cursor on Windows.
5. **Cursor's manifest has no `dependencies` field** (probed). The unified installer copies ponytail,
   i-have-adhd, and humanizer into `~/.cursor/plugins/local/<name>/` before `tightship` (#85); the manual path
   in the install guide copies them first, and each shows **Local** under Customize → Plugins before the
   adopter runs a workflow command.
6. **Options are not readable from the Configure dialog in hooks** (probed). The manifest still declares
   `variables` from `content/options.toml` for the editor UI, but hooks and the binary read option state only
   from `$HOME/.cursor/tightship/options.json`, which `tightship options` writes and the install guide explains.
   Off is still exactly `false`; absent is on, as in epic #3.
7. **The contract reaches the agent through hook JSON**, not `cat` of Markdown. Each contract part hook runs
   `${CURSOR_PLUGIN_ROOT}/bin/tightship hook contract-part <file>` and prints one line of JSON:
   `{"additional_context":"<rendered part>"}`. Packing uses the same greedy algorithm and `[contract] limit`
   as epic #4. The limit is **10,000** characters per part, the same value as Claude Code.
8. **`sessionStart` runs once per conversation and not again after compaction** (probed). To meet
   FR-CTR-01.2, a `beforeSubmitPrompt` hook runs the same contract-part commands on every prompt while the
   contract option is on, so the agent's context holds the whole contract after compaction too. Memory hooks
   follow the same events as Claude Code, mapped in the table below.
9. **The CLI loads skills, commands, agents, and hook programs** like the editor, but not `alwaysApply` rules.
   The workflow does not ship contract as rules. Hooks are not run under `agent -p` ([NFR-FLEX-02.2](../../requirements/non-functional/flexibility.md#nfr-flex-02));
   acceptance scenarios that need hooks use an interactive CLI session or the editor. `bin/` is not on the
   agent shell `PATH` in the CLI; every skill and command that runs the binary uses the
   `{{capability:binary-invocation}}` token.
10. **The Cursor editor has no status line.** FR-STL-01.1, FR-STL-02.1, and FR-STL-03.1 apply only in the CLI,
    through `statusLine` in `~/.cursor/cli-config.json`. With the status line option off, the plugin writes
    nothing there.
11. **Minimum supported versions:** Cursor editor **3.25.6** and CLI **`2026.10.14`**. The install guide and
    release checklist state "Cursor 3.25.6 or later" and "`agent` 2026.10.14 or later".
12. **Release:** `cargo xtask package` writes `dist/tightship-cursor.zip` and commits
    `.cursor-plugin/marketplace.json` at the repository root (Cursor marketplace, one `tightship` entry with an
    `archive` source like epic #8). `dist/tightship-claude-code.zip` and `.claude-plugin/marketplace.json` stay
    as today.
13. **Imported Claude Code plugins:** Cursor also loads Claude Code plugins as **Imported**. The adopter must
    not install `tightship` in both harnesses on the same machine, or pieces load twice. The install guide says
    so in one sentence.
14. **Probe notes stay in `.claude/skills/cursor-plugin/SKILL.md`** until task 1 copies any new facts into the
    skill or this spec; they are not canonical content.
15. **Confirmed 2026-10-16:** single epic #58; options in `~/.cursor/tightship/options.json`; contract on every
    `beforeSubmitPrompt` while enabled; status line in the CLI only; probe skill path unchanged.

## Acceptance Criteria

### Contract

| Command or action | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| `cargo xtask build` | Maintainer | Yes | Writes `dist/claude-code/` and `dist/cursor/` |
| `cargo xtask build --binaries <folder>` | Maintainer / release | Yes | Both plugins with four platform binaries each |
| `cargo xtask package <tag>` | Release | Yes | `tightship-claude-code.zip` and `tightship-cursor.zip`, and their catalogs |
| Copy `dist/cursor/` to `~/.cursor/plugins/local/tightship/` | Maintainer | Yes | Local plugin for probes and try-it |
| `agent --plugin-dir dist/cursor` | Maintainer / adopter | Yes | CLI loads the built plugin |
| `install.sh --harness cursor` or `install.ps1 -Harness cursor` | Adopter | Yes | Companions, then tightship, into `~/.cursor/plugins/local/` |

### Request

#### `harnesses/cursor/harness.toml`

Same keys as [epic #1](../canonical-source-and-build/build-the-plugin.md), with Cursor values:

| Table | Key | Value for Cursor |
| --- | --- | --- |
| `[plugin]` | `name` | `"tightship"` |
| `[plugin]` | `description` | Same string as Claude Code |
| `[plugin]` | `author` | `"ATNexusLab"` |
| `[plugin]` | `repository` | `"https://github.com/ATNexusLab/tightship"` |
| `[plugin]` | `license` | `"MIT"` |
| `[references]` | `skill` | `"{name}"` |
| `[references]` | `command` | `"/{name}"` |
| `[references]` | `agent` | `"{name}"` |
| `[references]` | `option` | `${user_config.{name}}` in piece bodies only; hooks read `options.json` |
| `[access]` | `read-only` | `Read, Grep, Glob, Shell` (Cursor tool names; verifier uses Shell, not Bash) |
| `[contract]` | `limit` | `10000` |
| `[neutrality]` | `names` | `["Cursor", "Composer"]` |
| `[neutrality]` | `paths` | `[".cursor/", "~/.cursor", "CURSOR_PLUGIN_ROOT", "CURSOR_PROJECT_DIR"]` |
| `[neutrality]` | `tools` | `SwitchMode`, `TodoWrite`, `WebFetch`, `WebSearch`, `AskQuestion`, `CallMcpTool`, `GetDynamicTools`, `$ARGUMENTS` |

`[[companions]]` lists the same three companions as Claude Code for the install guide and for parity in
`harness.toml`; the build does not write a `dependencies` key in Cursor's `plugin.json`.

`[capabilities]` includes every capability token used in `content/`, with Cursor-native text. At minimum the
entries of Claude Code's `[capabilities]`, plus:

| Capability | Renders to | Notes |
| --- | --- | --- |
| `plan-mode` | `Plan mode` | `SwitchMode` in Cursor |
| `plan-approval` | `` `SwitchMode` to Agent `` | No `ExitPlanMode` |
| `explore-agent` | `the **explore** subagent` | `subagent_type` `explore` |
| `general-agent` | `` `generalPurpose` subagent `` | Task tool |
| `review-command` | `` `code-reviewer` subagent `` | No `/code-review` slash command |
| `security-review-command` | `` `security-reviewer` subagent `` | |
| `harness-instructions-file` | `` `AGENTS.md` `` | |
| `area-instructions` | `Cursor loads rules for the files in scope` | |
| `instruction-files-search` | `` `find . -name AGENTS.md` `` | |
| `instruction-files-migration` | Paragraph aligned with bootstrap for `AGENTS.md` only | |
| `binary-invocation` | `` "${CURSOR_PLUGIN_ROOT}/bin/tightship" `` | Every vault and hook mention |

#### `dist/cursor/`

| Path | From |
| --- | --- |
| `.cursor-plugin/plugin.json` | `[plugin]`, workspace version, `variables` from `content/options.toml` |
| `skills/<name>/SKILL.md` | Canonical skills only |
| `commands/<name>.md` | Canonical commands |
| `agents/<name>.md` | Canonical subagents; YAML adds `readonly: true` for the verifier |
| `contract/` | Packed parts, as epic #4 |
| `hooks/hooks.json` | `version` 1; events below |
| `bin/tightship` + `bin/tightship.cmd` + platform binaries | Same layout as Claude Code |

`hooks/hooks.json` uses **camelCase** event names. Contract hooks use `sessionStart` once and
`beforeSubmitPrompt` for every prompt (rule 8). Memory hooks:

| Event | When | Command |
| --- | --- | --- |
| `sessionStart` | New conversation | `tightship hook start` (JSON output + env file via binary) |
| `beforeSubmitPrompt` | Every prompt, memory on | `tightship hook prompt` |
| `preCompact` | Before compaction | `tightship hook checkpoint` |

Option guards prepend a PowerShell test that reads `options.json`, not `CLAUDE_PLUGIN_OPTION_*`.

Success output ends with:

```text
Built dist/cursor/: 22 pieces (13 skills, 8 commands, 1 agent), <files> files.
Try it: copy dist/cursor to ~/.cursor/plugins/local/tightship and reload the window, or agent --plugin-dir dist/cursor
```

`<files>` is the count of every file under `dist/cursor/` after the build, as in epic #1.

### Response

Exit codes match epic #1: `0` on success, `1` on any diagnostic; failed build removes partial `dist/cursor/`.

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | Adds `harnesses/cursor/` and the renderer |
| Adopter | None from this project | Installs through Cursor's marketplace or local folder |

---

## Business Rules

### 1. Build authorization

N/A — local clone only.

### 2. Canonical content stays untouched

- **When:** the diff that adds Cursor lands.
- **Then:** `content/` has 0 changed lines ([NFR-FLEX-07](../../requirements/non-functional/flexibility.md#nfr-flex-07)).

### 3. Neutrality

- As epic #1, against `harnesses/cursor/harness.toml` only for Cursor terms.
- Shared canonical filenames and headings (`AGENTS.md`, "Task Breakdown") are not listed as Cursor-only terms
  because they appear in harness-neutral `content/`; Cursor-specific surface names stay in `[neutrality].names`,
  paths, and tools.

### 4. Companions before workflow (FR-INS-02.2)

- **When:** the adopter runs the unified installer for Cursor.
- **Then:** ponytail, i-have-adhd, and humanizer are in `~/.cursor/plugins/local/<name>/` before tightship is,
  and each shows **Local** under Customize → Plugins (BR-04). Without the installer,
  `docs/install/cursor/companions.md` gives the same steps by hand.

### 5. Options file (FR-INS-03, FR-INS-04, FR-INS-05, FR-INS-06)

- **When:** the adopter runs `tightship options set <name> <true|false>`, `tightship options set vault_folder
  <path>`, `tightship options list`, or `tightship options reset`.
- **Then:** `$HOME/.cursor/tightship/options.json` holds booleans and `vault_folder`; hooks read it through the
  binary only. `set` prints `Set <name> to <value>.` on stdout and exit `0`. `list` prints pretty-printed JSON
  and exit `0`. `reset` removes the file, prints `Removed ~/.cursor/tightship/options.json.`, and exit `0`.
  Uninstall removes `$HOME/.cursor/tightship/` whole; the guide lists the step.

### 6. Contract (FR-CTR-01 to FR-CTR-04)

- Same packing and gating as [epic #4](../contract/load-the-contract.md), with hook delivery per rules 7–8.
- FR-CTR-04 deferrals to companions: unchanged; companions are separate plugins.

### 7. Memory and handoff (FR-MEM-*, FR-MEM-06, FR-MEM-07)

- Same behavior as epics #5 and #6; only hook wiring and `TIGHTSHIP_*` env names differ. Vault path
  `~` expansion is [#60](https://github.com/ATNexusLab/tightship/issues/60), not this epic.

### 8. Verifier (FR-VER-01, FR-VER-02)

- Ships as `agents/adversarial-verifier.md` with `readonly: true`. Cursor does not enforce read-only
  (probed); the agent instructions and FR-VER-02's private-copy rule remain the enforcement.

### 9. Status line (FR-STL-01 to FR-STL-03)

- **Editor:** N/A for FR-STL-01.1, FR-STL-02.1, and FR-STL-03.1 ([NFR-FLEX-02.1](../../requirements/non-functional/flexibility.md#nfr-flex-02)).
- **CLI:** when the option is on and the adopter consented, `tightship status-line enable` writes
  `statusLine` into `~/.cursor/cli-config.json` and saves the previous value under
  `$HOME/.cursor/tightship/status-line.json`; `tightship status-line render` is the command Cursor runs.

### 10. NFR-COMP-01

- Same vault folder in `options.json` as in Claude Code's user settings; `tightship vault` commands behave
  identically across harnesses on one machine.

### 11. Persistence and audit

- **Build:** writes only `dist/cursor/`.
- **Adopter:** Cursor marketplace / local install; `options.json`; vault git remote unchanged from M1.

---

## Errors

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `HARNESS_INVALID` | `1` | Invalid `harnesses/cursor/harness.toml` | `harnesses/cursor/harness.toml: <reason>` |
| `CONTRACT_SECTION_TOO_LONG` | `1` | Part over `[contract] limit` | Same pattern as epic #4 with `cursor` in the path |
| `CAPABILITY_UNDEFINED` | `1` | Token with no `[capabilities]` entry | Same as epic #1 |
| `OPTIONS_UNKNOWN` | `1` | `tightship options set` names no option | `error: unknown option "<name>"` |
| `OPTIONS_INVALID_VALUE` | `1` | `set` with a non-boolean for a boolean option | `error: <name> must be true or false` |
| `OPTIONS_IO` | `1` | `options.json` cannot be read or written | `error: <io message>` |

---

## Side Effects

- **Persistence:** `dist/cursor/` replaced atomically via staging, as Claude Code.
- **Concurrency:** one build at a time per clone.
- **Transaction:** staging `dist/.cursor.tmp` then rename to `dist/cursor/`.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — Build both harnesses (happy path)

```gherkin
Given content/ and harnesses/claude-code/ and harnesses/cursor/ are valid
When the maintainer runs cargo xtask build
Then dist/claude-code/ and dist/cursor/ exist
And dist/cursor/commands/spec.md holds a rendered /spec reference
And dist/cursor/skills/grilling/SKILL.md holds no Claude Code literal
```

### Scenario 2 — Install companions then plugin (FR-INS-02.2)

```gherkin
Given an adopter runs the unified installer for Cursor
When the installer finishes
Then ponytail, i-have-adhd, and humanizer are in ~/.cursor/plugins/local/
And tightship's skills and commands appear in Customize
```

### Scenario 3 — Contract after compaction (FR-CTR-01.2, editor)

```gherkin
Given tightship is installed with the contract option on
And a new Agent conversation has started
When the adopter compacts the context
And sends another prompt
Then the agent's context still holds the contract from its first rule to its last
```

### Scenario 4 — Memory off (FR-CTR-03.1)

```gherkin
Given options.json has "memory": false
When sessionStart and beforeSubmitPrompt hooks run
Then no vault text is injected
And contract parts omit vault rules
```

### Scenario 5 — Same vault in Claude Code and Cursor (NFR-COMP-01.1)

```gherkin
Given the adopter set the same vault_folder in Claude Code and in options.json
When they write a note from Claude Code
Then a Cursor session with memory on shows that note at session start
```

### Scenario 6 — CLI loads plugin (NFR-FLEX-02.1, interactive)

```gherkin
Given dist/cursor is built
When the adopter runs agent --plugin-dir dist/cursor
And asks which slash commands are available
Then the answer lists /spec and the other workflow commands
```

### Scenario 7 — Verifier dispatches (FR-VER-01)

```gherkin
Given tightship is installed in Cursor
When the adopter invokes adversarial-verifier through Task
Then the subagent runs with the shipped prompt
```

### Scenario 8 — Option off leaves no contract text (FR-CTR-02.1)

```gherkin
Given options.json has "contract": false
When a new conversation starts
Then the agent's context holds no rule of the contract
```

### Scenario 9 — Set an option (FR-INS-04.1, happy path)

```gherkin
Given options.json does not exist
When the adopter runs tightship options set memory false
Then stdout is "Set memory to false."
And options.json holds "memory": false
```

### Scenario 10 — Unknown option (exception)

```gherkin
When the adopter runs tightship options set not_an_option true
Then stderr is "error: unknown option \"not_an_option\""
And the exit code is 1
```

---

## Architecture impact

- **Building blocks:** `harness-definitions` gains `harnesses/cursor/`; `build` gains a Cursor renderer;
  `claude-code-plugin` diagram gets a sibling **Cursor plugin** container (M2, #58).
- **Runtime:** contract and memory scenarios duplicate M1 paths with Cursor hook events and JSON stdout.
- **Deployment:** `docs/install/cursor/`; second zip in release pipeline; adopter prerequisite Cursor 3.25.6+.

## Data model impact

- **Options:** new file entity `$HOME/.cursor/tightship/options.json` (mirror of Claude Code user settings
  slice); documented in `docs/data-model/options.md`.
- **Status line:** CLI `~/.cursor/cli-config.json` `statusLine` documented next to Claude Code settings.

## Out of Scope

- **Cloud Agents** hook gaps listed in Cursor docs — not a harness of BR-01 for this epic.
- **Enforcing verifier read-only** in Cursor — platform limitation; instructions only.
- **Status line in the editor** — no API; FR-STL in the editor is excluded by NFR-FLEX-02.1.
- **Companion dependencies installed by Cursor** — no manifest field; the unified installer copies them (#85).
- **Codex and Antigravity CLI** — M3/M4 epics #59 and later.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on | Issue |
| --- | --- | --- | --- | --- | --- |
| 1 | Document Cursor hook probes in maintainer skill | Maintainer probes on the dev platform; update `.claude/skills/cursor-plugin/SKILL.md`; CI covers cross-platform Rust | Documented in the skill; limit stays 10000 unless a probe forces a change request | — | #65 |
| 2 | Write companion and marketplace install steps | `docs/install/cursor/companions.md`, rule 4 sentence in `install.md` | Scenario 2 | — | #66 |
| 3 | Add harnesses/cursor/harness.toml and capability entries | `harnesses/cursor/harness.toml` (`[plugin]`, `[dist]`, `[install]`, `[paths]`, `[capabilities]`, `[neutrality]` per spec table), `launcher-env.*`; neutrality check | Scenario 1 | — | #67 |
| 4 | Render dist/cursor pieces and manifest | `xtask/src/build/cursor.rs`, multi-harness `build.rs`, `bin/` launchers, `agents/adversarial-verifier.md` with `readonly: true`, tests in `xtask/tests/build.rs` | Scenarios 1 and 7 | 3 | #68 |
| 5 | Pack contract and emit Cursor hooks | `xtask` contract pack for `cursor`, `dist/cursor/hooks/hooks.json`, `tightship hook contract-part` JSON output, `xtask/tests/cursor_contract.rs` | Scenarios 3, 4, 8 | 4 | #69 |
| 6 | Options file and tightship options subcommands | `tightship/src/options.rs` (`$HOME/.cursor/tightship/options.json`), `tightship options` in `main.rs`, hook option guards in `xtask` (PowerShell reads JSON); manifest `variables` already in task 4 — not Antigravity `options.toml` | Scenarios 4, 8, 9, 10 | 4 | #70 |
| 7 | Wire memory and handoff hooks for Cursor | Cursor-specific hook wiring only (`tightship/src/hook/*`, `launcher-env` → `TIGHTSHIP_*`); reuse M1 and #63 paths — no second vault implementation | Scenario 5 | 5, 6 | #71 |
| 8 | Install guide and try-it line in AGENTS.md | `docs/install/cursor/`, README | Scenario 2, 6 | 2, 4 | #72 |
| 9 | CLI status line probe and enable/render path | `tightship/status-line`, docs, task 1 probe notes | Rule 9 | 6 | #73 |
| 10 | Package tightship-cursor.zip and catalog | `xtask/package.rs`, `.github/workflows/release.yml`, root catalog | Release contract in rule 12 | 4, 7, 8, 9 | #74 |
| 11 | Release checklist rows for Cursor | `docs/release/checklist.md`, NFR-FLEX-02.1 sign-off | Scenario 6 | 10 | #75 |
