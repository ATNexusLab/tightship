# #4 — Load the contract into every session

> **Status:** closed
> **Profile:** API
> **Module:** `content/`, `harnesses/claude-code/`, `xtask/`
> **Epic:** #4
> **Requirements:** FR-CTR-01, FR-CTR-02, FR-CTR-03, FR-CTR-04, BR-05
> **Sprint:** 3

The plugin carries the baseline's contract as canonical content, and the build renders it into hooks that
put it in the agent's context when a session starts, after `/clear`, and after each compaction. The contract
option turns every one of those hooks off, and the memory option turns off the part that speaks of the vault.

Terms from the SRS, as used here:

- **Contract**: the always-on text that tells the agent how to work in every session
  ([glossary](../../requirements/glossary.md), [FR-CTR-01](../../requirements/functional/contract.md#fr-ctr-01)).
- **Baseline**: commit `1f41699` of the maintainer's private configuration, the pre-plugin workflow. Its
  contract is the file `CLAUDE.md` there: 335 lines, 13 sections, about 25,000 characters
  ([FR-CTR-04](../../requirements/functional/contract.md#fr-ctr-04)).
- **Option**: a part of the plugin the adopter turns on or off; this spec reads `contract` and `memory` by
  rule 4 of the [epic #3 spec](../options/choose-the-options.md): off only when the value is exactly `false`.
- **Companion plugins**: ponytail, i-have-adhd, and humanizer, whose rules the workflow never restates
  ([BR-05](../../requirements/business-rules.md#br-05)).

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-15, after reading Claude Code's hooks reference and a probe with a
throwaway plugin on Claude Code 2.1.299:

1. **The contract reaches the session through `SessionStart` hooks.** Claude Code caps each hook's injected
   text at 10,000 characters, with no setting to raise it; over the cap, the agent sees only a file path and
   a preview of 2,000 characters. So the build packs the contract's sections, in order, into parts of at most
   10,000 characters, and each part is one hook. Hooks of one event run in parallel, and the probe showed
   their text arriving in the order they finish, so each part stands on its own under its own title.
2. **The hooks fire on `startup`, `clear`, and `compact`.** The probe showed a compaction firing the hook
   again with `source` `compact`, and a resumed or forked session already holding the earlier copy, which a
   new run would duplicate.
3. **Each hook is one `sh` command line that prints a file the build wrote.** No program ships for it, and the
   `tightship` crate still starts with epic #5. ADR 0001 is amended: a hook command line that only prints a
   file the plugin ships is not a program. It needs what the `sh` launcher needs: Git for Windows on Windows.
4. **The canonical contract is one file, `content/contract.md`.** Its front matter names each section an
   option turns off. Such a section becomes a hook of its own, which also tests that option. The text never
   holds a conditional; the two mentions of the vault outside the Memory section move into it.
5. **The Memory section names the commands epic #5 builds:** `tightship vault search`, `tightship vault new`,
   and `tightship vault path`, in place of the baseline's `vault.py`. Epic #5's spec takes these names.
6. **What leaves the baseline as personal** is the four items of rule 8, each with its reason. Every other rule
   is carried, with harness names turned into capability tokens and piece names into piece tokens.
7. Two tasks: the build renders a contract into hooks, then the baseline's contract is migrated.

## Acceptance Criteria

### Contract

The build keeps its command and its exit codes; its success line counts the new files.

| Command | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| `cargo xtask build` | Maintainer | Yes | Also writes `contract/` and `hooks/hooks.json` into `dist/claude-code/` |
| A session start, `/clear`, or a compaction, with the plugin enabled | Adopter, through Claude Code | Yes | Each contract hook prints its part, or nothing when its options are off |

### Request

#### `content/contract.md`

New, optional. Markdown, in English ([BR-03](../../requirements/business-rules.md#br-03)), linted like every
other `.md`:

- An optional TOML front matter between `+++` lines, with one optional table, `[gated]`, mapping a section's
  heading text to the name of the option that turns it off.
- One level-1 title, then a preamble, then the sections, each opening with a level-2 heading. Headings of
  level 3 and deeper belong to their section.

```markdown
+++
[gated]
Memory = "memory"
+++

# Global Working Contract

Always-on instructions for **every** project. …

## How I work

- **I am the orchestrator.** …

## Memory

The vault is the adopter's git repository of notes; `tightship vault path` resolves its location. …
```

| Element | Type | Required | Validation |
| --- | --- | --- | --- |
| Front matter | TOML | No | Only the key `gated`; anything else fails with `FRONT_MATTER_INVALID` |
| `[gated]` key | string | No | The text of a level-2 heading of the file, exactly; else `CONTRACT_SECTION_UNKNOWN` |
| `[gated]` value | string | — | The `name` of an option in `content/options.toml`; else `CONTRACT_OPTION_UNKNOWN` |
| Title | level-1 heading | Yes | The first heading of the file; else `CONTRACT_TITLE_MISSING` |
| Section | level-2 heading and the text up to the next one | No | Rendered with its part's title, at most the harness's limit; else `CONTRACT_SECTION_TOO_LONG` |

It is canonical content: token resolution, the literal-reference check, the neutrality check, and the
identity scan of epic #1 read it like every other file of `content/`.

#### `harnesses/claude-code/harness.toml`

Gains a table and the capabilities rule 8 needs:

```toml
[contract]
limit = 10000
```

| Key | Type | Required | Validation |
| --- | --- | --- | --- |
| `contract.limit` | integer | Yes, when `content/contract.md` exists | 1 to 1,000,000; the most characters one hook may inject |

### Response

#### `dist/claude-code/contract/`

One file per part, each the part's title line, a blank line, then its sections joined by one blank line:

| File | Holds | Title line |
| --- | --- | --- |
| `part-<n>.md`, `<n>` from 1 | The preamble in part 1, then every section not gated, packed in file order | `# <title> (<n>/<m>)`, `<m>` being the number of these parts |
| `<option>.md` | The sections gated by that option, in file order | `# <title> (<section heading>)`, headings joined by a comma and a space |

A part's length is counted in UTF-16 code units, the way Claude Code counts a hook's text, title line
included. Packing is greedy: a section joins the current part while the part stays within the limit, and
starts the next part otherwise.

#### `dist/claude-code/hooks/hooks.json`

One `SessionStart` group, one hook per file of `contract/`, parts first, then the gated files in the order
`[gated]` first names their option:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|clear|compact",
        "hooks": [
          {
            "type": "command",
            "command": "[ \"$CLAUDE_PLUGIN_OPTION_CONTRACT\" = false ] || cat \"${CLAUDE_PLUGIN_ROOT}/contract/part-1.md\""
          },
          {
            "type": "command",
            "command": "[ \"$CLAUDE_PLUGIN_OPTION_CONTRACT\" = false ] || [ \"$CLAUDE_PLUGIN_OPTION_MEMORY\" = false ] || cat \"${CLAUDE_PLUGIN_ROOT}/contract/memory.md\""
          }
        ]
      }
    ]
  }
}
```

The test on `CLAUDE_PLUGIN_OPTION_CONTRACT` is present when `content/options.toml` declares `contract`, and a
gated file adds one test for its option, the name uppercased. With no `content/contract.md`, the build writes
neither `contract/` nor `hooks/hooks.json`.

With the baseline's contract migrated, the build writes 3 parts and `memory.md`, and its success output reads:

```text
Built dist/claude-code/: 22 pieces (13 skills, 8 commands, 1 agent), 33 files.
Try it: claude --plugin-dir dist/claude-code
```

#### What the agent's context holds

| Contract | Memory | Context after a session start, `/clear`, or a compaction |
| --- | --- | --- |
| On | On | Every part and `memory.md`, each in one system reminder |
| On | Off | Every part; no line that names the vault |
| Off | Any | Nothing from the contract |

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | Edits `content/contract.md` and runs the build |
| Adopter | None from this project | Turns the options on or off in their own Claude Code configuration |

---

## Business Rules

### 1. Authorization

N/A. The build runs on a local clone, and the hooks run in the adopter's own session, with no service of this
project behind either.

### 2. The contract is read and checked

- **When:** while `build` reads `content/`, before it renders anything.
- The front matter, the title, `[gated]`, and every section are checked as the Request table says. Every
  error is reported together with the others, as rule 10 of epic #1 sets, and none leaves `dist/claude-code/`
  behind.

### 3. Sections are packed into parts within the harness's limit

- **When:** `cargo xtask build` renders the Claude Code plugin.
- Tokens are resolved first; lengths are counted on the rendered text.
- The preamble opens part 1. Each section not gated follows in file order, by the greedy packing of the
  Response.
- A section that exceeds the limit on its own, with its part's title line, fails with
  `CONTRACT_SECTION_TOO_LONG`.

### 4. A gated section leaves with its option

- **When:** a hook runs.
- A gated file's hook prints nothing when its option's value is exactly `false`, and prints the file
  otherwise, an absent value included (rule 4 of epic #3).
- No section outside a gated file names the vault or a vault command.

### 5. The contract option turns every hook off

- **When:** a hook runs.
- With `CLAUDE_PLUGIN_OPTION_CONTRACT` exactly `false`, every contract hook prints nothing and exits 0. The
  skills, commands, and verifier load and run as before (FR-CTR-02.2).

### 6. The contract is placed again after each compaction

- The hooks fire on the sources `startup`, `clear`, and `compact`.
- On `resume` and `fork`, they do not fire: the session already holds the copy placed before.

### 7. A hook that fails never stops the session

- A hook whose file is missing, or that Claude Code runs under PowerShell for lack of Git Bash, exits non-zero.
  Claude Code shows the error and starts the session with the other parts
  ([NFR-REL-01](../../requirements/non-functional/reliability.md#nfr-rel-01)).

### 8. The contract is the baseline's, minus what is personal

- `content/contract.md` holds every rule of the baseline's `CLAUDE.md`, with only these changes:
  - Every literal reference to a piece replaced by its token, such as `/requirements` by
    `{{command:requirements}}` and `` `testing-contract` `` by `{{skill:testing-contract}}`.
  - Every harness term replaced by a capability token. Reused: `area-instructions`, `explore-agent`,
    `plan-mode`, `review-command`, `security-review-command`. New in `harness.toml`: `skill-tool`, rendered
    "the Skill tool", and `instruction-files-precedence`, rendered "A `CLAUDE.md` or `CLAUDE.local.md` in a
    directory, or any directory above it, stops Claude Code from reading the `AGENTS.md` there".
  - The Git rule ends at "goes to the repo's `AGENTS.md`", and the Memory section gains "A reason worth
    keeping that belongs in no repo goes to the vault." The Closeout step of the dev loop reads "surface gates,
    docs, traceability, and security debt", since its memory item belongs to epic #5.
  - The `---` lines between sections removed; the build separates sections itself.
  - Lines past 110 columns rewrapped, outside tables, and the formatting markdownlint requires.
- Left out as personal, each replaced as shown:

| Baseline text | Why it is personal | In the shipped contract |
| --- | --- | --- |
| "Personal, always-on instructions" | Describes the maintainer's own file | "Always-on instructions" |
| "The vault is `ai-memory`, shared across harnesses and machines through a private git repo" | Names the maintainer's vault and where it is hosted | "The vault is the adopter's git repository of notes, shared across harnesses and machines" |
| `vault.py path`, `vault.py search`, `vault.py new` | The maintainer's script, which the plugin does not ship | `tightship vault path`, `tightship vault search`, `tightship vault new` (epic #5) |
| "wait for the "vai"" | The maintainer's word in Portuguese | "wait for a go-ahead" |

### 9. The contract restates no companion's rule

- No rule of `content/contract.md` states what ponytail, i-have-adhd, or humanizer already provides. The
  contract names a companion only to defer to it or to override one of its defaults, as the baseline does.

### 10. Persistence and audit

- **Files changed:** `dist/claude-code/contract/` and `dist/claude-code/hooks/hooks.json`, by the build.
  Nothing on the adopter's machine: the hooks only print.
- **Audit:** N/A. Git records every change to `content/contract.md`.
- **Events/integrations fired:** N/A. Neither the build nor the hooks use the network.

---

## Errors

Every message goes to stderr, prefixed by `error:` and a space, as in epic #1.

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `FRONT_MATTER_INVALID` | `1` | Rule 2, a key other than `gated` | "`content/contract.md: front matter: unknown key "<key>"`" |
| `CONTRACT_TITLE_MISSING` | `1` | Rule 2 | "`content/contract.md: the contract opens with no # title`" |
| `CONTRACT_SECTION_UNKNOWN` | `1` | Rule 2 | "`content/contract.md: front matter: [gated] names the section "<heading>", which the file does not have`" |
| `CONTRACT_OPTION_UNKNOWN` | `1` | Rule 2 | "`content/contract.md: front matter: the section "<heading>" is gated by "<option>", which content/options.toml does not declare`" |
| `CONTRACT_SECTION_TOO_LONG` | `1` | Rule 3 | "`content/contract.md:<line>: the section "<heading>" renders to <count> characters with its title; claude-code takes at most <limit> per hook`" |
| `CONTRACT_LIMIT_MISSING` | `1` | Rule 3 | "`harnesses/claude-code/harness.toml: content/contract.md exists, so [contract] limit is required`" |

`<line>` is the line of the section's level-2 heading. `<count>` and `<limit>` are plain integers.

## Side Effects

- **Persistence:** the build replaces `dist/claude-code/` whole, as in epic #1.
- **Concurrency:** the hooks run in parallel and write nothing, so their order changes only the order of the
  parts in the context.
- **Transaction:** `contract/` and `hooks/hooks.json` are written into `dist/.claude-code.tmp/` with the rest
  of the plugin and published by the same rename.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — Build the contract (happy path)

```gherkin
Given content/contract.md holds a title, a preamble, sections A, B, and C, and a section Memory
And its front matter gates Memory by "memory"
And content/options.toml declares contract and memory
When the maintainer runs `cargo xtask build`
Then it exits 0
And dist/claude-code/contract/ holds part-1.md and memory.md
And part-1.md opens with "# <title> (1/1)" and holds the preamble, then A, B, and C, in that order
And memory.md opens with "# <title> (Memory)"
And dist/claude-code/hooks/hooks.json holds one SessionStart group with matcher "startup|clear|compact"
  and two hooks, the second also testing CLAUDE_PLUGIN_OPTION_MEMORY
And `claude plugin validate dist/claude-code` reports "Validation passed" with no warning
```

### Scenario 2 — Sections overflow into a second part (alternative)

```gherkin
Given the harness's limit is 10000
And content/contract.md holds sections A of 6,000 characters and B of 5,000, none gated
When the maintainer runs `cargo xtask build`
Then part-1.md holds the preamble and A, and opens with "(1/2)"
And part-2.md holds B, and opens with "(2/2)"
And neither file exceeds 10,000 UTF-16 code units
```

### Scenario 3 — A section too long (exception)

```gherkin
Given content/contract.md holds a section "Code I write" of 10,001 characters
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/contract.md:<line>: the section \"Code I write\" renders to <count> characters
  with its title; claude-code takes at most 10000 per hook"
And dist/claude-code/ does not exist
```

### Scenario 4 — A gated section the file does not have (exception)

```gherkin
Given content/contract.md gates "Memories" by "memory" and has no section "Memories"
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/contract.md: front matter: [gated] names the section \"Memories\", which the
  file does not have"
```

### Scenario 5 — A gate on an undeclared option (exception)

```gherkin
Given content/contract.md gates "Memory" by "vault"
And content/options.toml declares no option "vault"
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/contract.md: front matter: the section \"Memory\" is gated by \"vault\", which
  content/options.toml does not declare"
```

### Scenario 6 — A contract with no title (exception)

```gherkin
Given content/contract.md opens with "## How I work"
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/contract.md: the contract opens with no # title"
```

### Scenario 7 — An unknown front matter key (exception)

```gherkin
Given content/contract.md's front matter holds the key "order"
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/contract.md: front matter: unknown key \"order\""
```

### Scenario 8 — A harness with no limit (exception)

```gherkin
Given content/contract.md exists
And harnesses/claude-code/harness.toml has no [contract] table
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: harnesses/claude-code/harness.toml: content/contract.md exists, so [contract] limit is
  required"
```

### Scenario 9 — No contract (alternative)

```gherkin
Given content/ has no contract.md
When the maintainer runs `cargo xtask build`
Then it exits 0
And dist/claude-code/ holds neither contract/ nor hooks/hooks.json
```

### Scenario 10 — The hooks honor the options (FR-CTR-02.1, FR-CTR-03.1)

```gherkin
Given the plugin built from scenario 1
When each hook command runs under sh with CLAUDE_PLUGIN_ROOT set to dist/claude-code
Then with no option variable set, each prints its file and exits 0
And with CLAUDE_PLUGIN_OPTION_MEMORY=false, the part hook prints part-1.md and the memory hook prints nothing
And with CLAUDE_PLUGIN_OPTION_CONTRACT=false, every hook prints nothing and exits 0
```

### Scenario 11 — The whole contract at session start (FR-CTR-01.1, happy path)

```gherkin
Given the baseline's contract is migrated and the plugin is loaded with every option on
When a session starts
Then the agent's context holds parts 1 to 3 and memory.md
And every one of the 13 section headings of content/contract.md appears in it
And the build reported 33 files
```

### Scenario 12 — The contract survives compaction (FR-CTR-01.2)

```gherkin
Given a session with the contract on
When the context is compacted with /compact
Then the hooks run with source "compact"
And the agent's context afterward holds every one of the 13 section headings
```

### Scenario 13 — A resumed session holds one copy (alternative)

```gherkin
Given a session with the contract on, ended
When the adopter resumes it
Then the contract hooks do not run
And the agent's context holds the contract once
```

### Scenario 14 — The contract turned off (FR-CTR-02.1, FR-CTR-02.2)

```gherkin
Given the plugin is installed with contract set to false
When a session starts
Then the agent's context holds none of the 13 section headings
And when the adopter invokes /tightship:spec, the command's skill loads and runs
```

### Scenario 15 — Memory turned off (FR-CTR-03.1)

```gherkin
Given the plugin is installed with contract on and memory set to false
When a session starts
Then the agent's context holds parts 1 to 3 and not memory.md
And no line of parts 1 to 3 holds "vault" or "tightship vault"
```

### Scenario 16 — Memory turned on (FR-CTR-03.2)

```gherkin
Given the plugin is installed with contract and memory on
When a session starts
Then memory.md is in the agent's context
And it holds every rule of the baseline's Memory section, as rule 8 changes them
And it holds "A reason worth keeping that belongs in no repo goes to the vault."
```

### Scenario 17 — Every baseline rule is carried over (FR-CTR-04.1)

```gherkin
Given the baseline's CLAUDE.md at commit 1f41699
When each of its rules, bullets, table rows, and numbered steps is looked up in content/contract.md
Then each one is found, with only the changes of rule 8
Or it is one of the four rows of rule 8's personal table
```

### Scenario 18 — No companion's rule is restated (BR-05)

```gherkin
Given the skills of ponytail, i-have-adhd, and humanizer at the versions the adopter installs
When each rule of content/contract.md is compared with them
Then none states what a companion already provides
And each mention of a companion defers to it or overrides one of its defaults
```

### Scenario 19 — A broken hook does not stop the session (NFR-REL-01, exception)

```gherkin
Given an installed copy of the plugin whose contract/part-2.md was deleted
When a session starts
Then the session starts
And the agent's context holds parts 1 and 3 and memory.md
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md), marked M1, #4:

- **Building blocks:**
  - Canonical content gains `contract.md`, the one authored copy of the contract.
  - Harness definitions gain `[contract] limit` and the capabilities `skill-tool` and
    `instruction-files-precedence`.
  - Build packs the contract into parts and gates them by option.
  - Claude Code plugin gains `contract/` and `hooks/hooks.json`: its first hooks, plain `sh` lines.
- **Runtime:** the first scenario, "Session start: the contract", replaces the line saying epic #5 brings it.
- **Concepts:** a new page, contract, holds how the contract reaches a session in parts, its gates, and its
  limit. Options names the contract's hooks as the first readers of an option.
- **Diagrams:** the containers diagram adds the contract to canonical content and the contract hooks to the
  Claude Code plugin.
- **ADR 0001** is amended: a hook command line that only prints a file the plugin ships is not a program.

## Data model impact

N/A — the contract holds no data of the adopter, and the options it reads keep the model epic #3 set.

## Out of Scope

- **What each skill and command does without the contract** (FR-CTR-05): epic #8, in the install guide.
- **The vault commands the Memory section names:** epic #5 builds `tightship vault path`, `search`, and `new`.
  Until it lands, a development build's Memory section names commands that do not exist yet; nothing ships
  before epic #8.
- **The contract in subagents:** a `SessionStart` hook reaches the main session only. The verifier works from
  its own instructions, and the baseline's subagent read nothing more than them.
- **The contract in Cursor, Antigravity CLI, and Codex:** M2 to M4, each against its own limit. OQ-06 stays
  open for them.
- **Changes to the baseline after `1f41699`:** the baseline is fixed by the SRS; a new rule goes through
  change control.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- |
| 1 | Render the contract into session-start hooks | `xtask/src/contract.rs` (new: read, check, pack), `xtask/src/content.rs` (accept `contract.md`), `xtask/src/claude_code.rs` (`contract/`, `hooks/hooks.json`), `xtask/src/harness.rs` and `harnesses/claude-code/harness.toml` (`[contract] limit`), `xtask/src/main.rs`, `xtask/tests/build.rs` and its fixture in `xtask/tests/common/`, about 8 files | Scenarios 1 to 10 by tests, the hook lines of scenario 10 run under `sh` in the test | — |
| 2 | Migrate the baseline's contract | `content/contract.md` (new), `harnesses/claude-code/harness.toml` (2 capabilities), about 2 files | Scenarios 11 to 16 and 19 in sessions of an isolated Claude Code configuration; 17 and 18 by a comparison with the baseline and the companions, read at review | 1 |
