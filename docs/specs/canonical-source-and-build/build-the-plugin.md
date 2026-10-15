# #1 — Build the Claude Code plugin from the canonical content

> **Status:** published
> **Profile:** API
> **Module:** `content/`, `harnesses/claude-code/`, `xtask/`
> **Epic:** #1
> **Requirements:** NFR-MNT-01, NFR-FLEX-06, NFR-SEC-01, NFR-SEC-02, BR-02, BR-03, FR-SKL-03
> **Sprint:** 1

The maintainer edits one file per workflow piece, and one command turns the whole set into the Claude Code
plugin, with the identity and secret scans passing. The stack, the format of the canonical content, the
per-harness places, and the plugin's reach are decided in
[ADR 0001](../../decisions/0001-rust-build-and-tokenized-canonical-content.md), which this spec carries.

Terms from the SRS, as used here:

- **Canonical content** — the single authored copy of every piece, from which each harness's plugin is built
  ([NFR-MNT-01](../../requirements/non-functional/maintainability.md#nfr-mnt-01)).
- **Piece** — one skill, command, or subagent of the workflow.
- **Baseline** — commit `1f41699` of the maintainer's private configuration, the pre-plugin workflow
  ([overview](../../requirements/overview.md#assumptions-and-dependencies)).
- **Per-harness place** — the folder where a harness's names, paths, and tools may appear
  ([NFR-FLEX-06](../../requirements/non-functional/flexibility.md#nfr-flex-06)): `harnesses/<harness>/`.
- **Shipped file** — every file under `dist/<harness>/`, the plugin a harness installs
  ([NFR-SEC-01](../../requirements/non-functional/security.md#nfr-sec-01)).

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-14:

1. Rust for the build and for every program the plugin ships, in one Cargo workspace; the build is the
   `xtask` crate.
2. Canonical content is one Markdown file per piece under `content/`, with TOML front matter and tokens for
   every reference to another piece or to a harness capability.
3. The plugin is named `tightship`, which settles the SRS's open question OQ-01.
4. The generated plugin goes to `dist/`, which git ignores; a release ships it as an archive. The release is
   epic #8.
5. The identity scan reads its terms from the machine and from an untracked file, never from the repository.
   The secret scan is gitleaks over the whole history.
6. This epic migrates the 13 baseline skills, 8 of its 9 commands, and its subagent. The `handoff` command
   comes with epic #6, and the memory item of `closeout` comes with epic #5, because both need the memory
   script.
7. The plugin's reach is the one in ADR 0001, which settles the SRS's open question
   OQ-03.

## Acceptance Criteria

### Contract

The maintainer runs two commands at the repository root. Neither takes an argument.

| Command | Who | Idempotent | Reads | Writes |
| --- | --- | --- | --- | --- |
| `cargo xtask build` | Maintainer | Yes | `content/`, `harnesses/`, `.identity-terms`, git config | `dist/claude-code/` only |
| `cargo xtask scan` | Maintainer | Yes | The git history, the working tree | Nothing |

`cargo xtask` is a Cargo alias for `cargo run --package xtask --`, set in `.cargo/config.toml`.

### Request

#### Canonical layout

| Path | Holds | Required |
| --- | --- | --- |
| `content/skills/<name>/SKILL.md` | One skill | Yes, one per skill |
| `content/skills/<name>/<file>` | A file the skill points to: a `.md` file is rendered like a body, any other file is copied byte for byte | No |
| `content/commands/<name>.md` | One command | Yes, one per command |
| `content/agents/<name>.md` | One subagent | Yes, one per subagent |
| `harnesses/claude-code/harness.toml` | Everything the build knows about Claude Code | Yes |
| `.identity-terms` | The maintainer's identity terms, one per line; ignored by git | Yes, on each machine that builds |

`<name>` is 1 to 64 characters of lowercase letters, digits, and single hyphens, starting with a letter.
Skills, commands, and subagents share one namespace: a name belongs to one piece.

#### Front matter

Each `SKILL.md`, command, and subagent file opens with TOML between two `+++` lines.

| Key | Type | Required | Validation |
| --- | --- | --- | --- |
| `name` | string | Yes | Equals the skill's folder name, or the command's or subagent's file name without `.md` |
| `description` | string | Yes | 1 to 1,024 characters, one line |
| `argument-hint` | string | No; commands only | 1 to 200 characters, one line |
| `access` | string | Yes; subagents only | `"read-only"` |

Any other key fails the build. The body after the closing `+++` is Markdown.

```markdown
+++
name = "spec"
description = "Map a feature into a closed, unambiguous spec — ready to become an epic"
argument-hint = "<epic issue or feature>"
+++

Produce the design document **before** any implementation, for: {{capability:arguments}}
```

#### Tokens

A token is `{{<kind>:<name>}}`, written anywhere in a body or a supporting `.md` file, code spans and code
blocks included.

| Kind | `<name>` names | Example |
| --- | --- | --- |
| `skill` | A folder in `content/skills/` | `{{skill:grilling}}` |
| `command` | A file in `content/commands/` | `{{command:spec}}` |
| `agent` | A file in `content/agents/` | `{{agent:adversarial-verifier}}` |
| `capability` | A key of `[capabilities]` in every `harness.toml` | `{{capability:plan-mode}}` |

#### `harnesses/claude-code/harness.toml`

| Table | Key | Value for Claude Code |
| --- | --- | --- |
| `[plugin]` | `name` | `"tightship"` |
| `[plugin]` | `description` | `"Skills, a command per step of the development loop, and a read-only verifier"` |
| `[plugin]` | `author` | `"ATNexusLab"` |
| `[plugin]` | `repository` | `"https://github.com/ATNexusLab/tightship"` |
| `[references]` | `skill` | `"{plugin}:{name}"` |
| `[references]` | `command` | `"/{plugin}:{name}"` |
| `[references]` | `agent` | `"{plugin}:{name}"` |
| `[access]` | `read-only` | `"Read, Grep, Glob, Bash"` |
| `[neutrality]` | `names` | `["Claude Code", "Claude", "Anthropic"]` |
| `[neutrality]` | `paths` | `[".claude/", "~/.claude", "CLAUDE.md", "CLAUDE.local.md", "CLAUDE_PLUGIN_ROOT", "CLAUDE_PLUGIN_DATA", "CLAUDE_PROJECT_DIR"]` |
| `[neutrality]` | `tools` | `["Skill tool", "Agent tool", "Task tool", "AskUserQuestion", "ExitPlanMode", "EnterPlanMode", "TodoWrite", "WebFetch", "WebSearch", "Explore", "general-purpose", "/code-review", "/security-review", "$ARGUMENTS"]` |

Every key is required. `[neutrality]` values are arrays of strings; every other value is a string. The
maintainer writes the file by hand.

`[capabilities]`, the text each capability token renders to in Claude Code:

| Capability | Renders to | Used by |
| --- | --- | --- |
| `arguments` | `$ARGUMENTS` | Every command that takes arguments |
| `plan-mode` | `plan mode` | `grilling` |
| `plan-approval` | `` `ExitPlanMode` `` | `grilling` |
| `explore-agent` | `the **Explore** agent` | `bootstrap` |
| `general-agent` | `` `general-purpose` subagent `` | `audit` |
| `review-command` | `` `/code-review` `` | `audit` |
| `security-review-command` | `` `/security-review` `` | `audit` |
| `harness-instructions-file` | `` `CLAUDE.md` `` | `bootstrap` |
| `area-instructions` | `Claude Code loads an area's file when it works there` | `bootstrap` |
| `instruction-files-search` | `` `find . -name AGENTS.md -o -name CLAUDE.md -o -name CLAUDE.local.md` `` | `bootstrap` |
| `instruction-files-migration` | The baseline's paragraph, from "A `CLAUDE.md` or `CLAUDE.local.md` in a directory" to "before touching it." | `bootstrap` |

### Response

#### `dist/claude-code/`

| Path | From |
| --- | --- |
| `.claude-plugin/plugin.json` | `[plugin]` of `harness.toml`, plus `version` from `[workspace.package]` of the root `Cargo.toml` |
| `skills/<name>/SKILL.md` | A canonical skill, or a canonical command: Claude Code documents `commands/` as legacy and runs a skill as a command |
| `skills/<name>/<file>` | A skill's supporting file: a `.md` file with its tokens replaced, any other file byte for byte |
| `agents/<name>.md` | A canonical subagent |

Each generated Markdown file opens with YAML front matter between `---` lines: `name`, `description`,
`argument-hint` when present, and `tools` for a subagent, taken from `[access]`. Each value is written as a
JSON string, which YAML reads as a double-quoted scalar. The body is the canonical body with every token
replaced as `[references]` and `[capabilities]` say.

```json
{
  "name": "tightship",
  "version": "0.1.0",
  "description": "Skills, a command per step of the development loop, and a read-only verifier",
  "author": { "name": "ATNexusLab" },
  "repository": "https://github.com/ATNexusLab/tightship"
}
```

#### Output and exit codes

`cargo xtask build`, on success, prints each step before it starts and the result at the end:

```text
> Reading content/
> Checking references and harness neutrality
> Rendering claude-code into dist/claude-code/
> Scanning dist/claude-code/ for identity terms
Built dist/claude-code/: 22 pieces (13 skills, 8 commands, 1 agent), 27 files.
Try it: claude --plugin-dir dist/claude-code
```

`cargo xtask scan`, on success:

```text
> Scanning the git history for secrets (gitleaks 8.30.1)
> Looking for vault notes in the working tree
No secret in 12 commits. No vault note.
```

The numbers in both outputs are the counts of that run.

| Exit | When |
| --- | --- |
| `0` | Every check passed; `build` wrote `dist/claude-code/` |
| `1` | At least one error of the table in **Errors**; `build` leaves no `dist/claude-code/` behind |

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | The only one who runs the build ([overview](../../requirements/overview.md#stakeholders)). An adopter never runs it. |

---

## Business Rules

### 1. Authorization

N/A — both commands run locally on a clone, with no service behind them. Whoever can clone the public
repository can run them; only a push needs the maintainer's write access.

### 2. One authored copy

- `content/` holds the only authored copy of every piece. `dist/` is rebuilt from it on every run: the build
  deletes `dist/claude-code/` first, then writes it whole.
- Two runs over the same `content/`, `harnesses/`, and workspace version write byte-identical files.
- `dist/` and `.identity-terms` are in `.gitignore`.

### 3. Every token resolves

- **When:** during the reference check of `build`, over every body and supporting `.md` file in `content/`.
- A token whose kind is not `skill`, `command`, `agent`, or `capability` fails with `TOKEN_UNKNOWN_KIND`.
- A token that names no piece of its kind fails with `TOKEN_MISSING_PIECE`.
- A `capability` token missing from `[capabilities]` of any `harness.toml` fails with `CAPABILITY_UNDEFINED`.
- Text with `{{` and `}}` that does not match `{{<kind>:<name>}}` fails with `TOKEN_MALFORMED`.

### 4. No literal reference

- **When:** during the reference check of `build`, over every body and supporting `.md` file in `content/`,
  after tokens are set aside.
- These forms of a known piece's name fail with `LITERAL_REFERENCE`:
  - `/<command>` not preceded by a letter, digit, `/`, `.`, or `-`, and not followed by a letter, digit,
    `/`, or `-`.
  - `` `<skill>` `` or `` `<agent>` ``, the name alone between backticks.
  - `commands/<command>.md`, `skills/<skill>/`, or `agents/<agent>.md`.

### 5. Harness neutrality

- **When:** during the neutrality check of `build`, over every file under `content/`, supporting files
  included.
- Every term in `[neutrality]` of every `harness.toml` is matched case-sensitively. A term that starts and
  ends with a letter or digit matches only as a whole word. A match fails with `HARNESS_TERM`.
- `harnesses/` is never scanned for neutrality.

### 6. Front matter is valid

- **When:** while `build` reads `content/`.
- A file with no `+++` block, a TOML syntax error, an unknown key, a missing required key, or a value outside
  its domain fails with `FRONT_MATTER_INVALID`.
- A `name` that differs from the file or folder name fails with `NAME_MISMATCH`.
- A name used by two pieces fails with `NAME_TAKEN`.

### 7. No identity ships

- **When:** the last step of `build`, over every file under `dist/claude-code/`.
- The terms are: git's `user.name` and `user.email`, the operating system's user name, the home folder's path
  written with `/` and with `\`, and every non-blank line of `.identity-terms`.
- Matching ignores case. A term that starts and ends with a letter or digit matches only as a whole word.
- A match fails with `IDENTITY_FOUND`, which names the file and line and never prints the term.
- A file or folder whose name holds a term fails with `IDENTITY_FOUND` on the folder that holds it, so the
  name is never printed either.
- A missing `.identity-terms` fails with `IDENTITY_TERMS_MISSING`. An empty `user.name` or `user.email` fails
  with `GIT_IDENTITY_MISSING`.

### 8. No secret and no vault note in the repository

- **When:** `scan`.
- gitleaks 8.30.1 runs over every commit of every branch, with its default rules and redacted output. Any
  finding fails with `SECRET_FOUND`.
- gitleaks absent from the `PATH` fails with `GITLEAKS_MISSING`. Any other version fails with
  `GITLEAKS_VERSION`.
- A tracked file whose front matter has an `id` matching `mem-` followed by digits is a vault note and fails
  with `VAULT_NOTE_FOUND`.

### 9. Content is migrated from the baseline

- Each of the 22 pieces is the baseline's file with only these changes:
  - YAML front matter rewritten as TOML, with `tools` replaced by `access = "read-only"`, and `name` added
    to each command.
  - Every literal reference and harness term replaced by its token.
  - The memory item of `closeout` left out: its section `## 5. Memory`, the `- Memory:` line of its output
    block, and `memory` in its description.
  - Lines past 110 columns rewrapped, outside tables and code blocks.
  - The formatting markdownlint requires: blank lines around headings, lists, tables and code blocks, a
    space inside table pipes, no trailing spaces, and `text` on a code block with no language.
  - `Cursor-based` in `api-design` lowercased to `cursor-based`.
  - `general-purpose generator` in `security-audit` reworded to `general random generator`, since
    `general-purpose` is a harness term.
- The pieces stay in English ([BR-03](../../requirements/business-rules.md#br-03)), as the baseline writes
  them.
- The 4 supporting files are `spec-writing/templates.md`, migrated like a body, and
  `project-docs/docs-theme.css`, `project-docs/image-viewer.css`, and `project-docs/image-viewer.js`, copied
  byte for byte.

| Kind | Pieces |
| --- | --- |
| Skills (13) | `api-design`, `architecture-reading`, `database-design`, `frontend-architecture`, `grilling`, `hexagonal-architecture`, `performance-analysis`, `project-docs`, `requirements-engineering`, `security-audit`, `spec-writing`, `technical-writing`, `testing-contract` |
| Commands (8) | `audit`, `bootstrap`, `closeout`, `epic`, `pr-review`, `requirements`, `spec`, `sprint` |
| Subagent (1) | `adversarial-verifier` |

### 10. Errors are reported together

- `build` runs every check over every file before it stops, and prints each error on its own line, sorted by
  path and line. Rendering and the identity scan run only when the checks before them found no error.
- `scan` runs both of its checks and prints every error.

### 11. Persistence and audit

- **Files changed:** `dist/claude-code/` by `build`; nothing by `scan`.
- **Audit:** N/A — a local command on the maintainer's machine. Git records every change to `content/` and
  `harnesses/`.
- **Events/integrations fired:** none. Neither command uses the network, except what gitleaks reads from the
  local clone.

---

## Errors

Every message goes to stderr, prefixed by `error:` and a space. `<path>` is relative to the repository root.

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `TOKEN_UNKNOWN_KIND` | `1` | Rule 3 | "`<path>:<line>: unknown token kind "<kind>" in {{<kind>:<name>}}`" |
| `TOKEN_MISSING_PIECE` | `1` | Rule 3 | "`<path>:<line>: {{<kind>:<name>}} names no <kind> in content/`" |
| `CAPABILITY_UNDEFINED` | `1` | Rule 3 | "`<path>:<line>: capability "<name>" has no entry in harnesses/<harness>/harness.toml`" |
| `TOKEN_MALFORMED` | `1` | Rule 3 | "`<path>:<line>: "<text>" is not a token; write {{<kind>:<name>}}`" |
| `LITERAL_REFERENCE` | `1` | Rule 4 | "`<path>:<line>: "<text>" refers to the <kind> "<name>"; write {{<kind>:<name>}}`" |
| `HARNESS_TERM` | `1` | Rule 5 | "`<path>:<line>: "<term>" belongs to <harness>; use a {{capability:…}} token defined in harnesses/<harness>/harness.toml`" |
| `FRONT_MATTER_INVALID` | `1` | Rule 6 | "`<path>: front matter: <reason>`" |
| `NAME_MISMATCH` | `1` | Rule 6 | "`<path>: name "<name>" differs from its file name "<file name>"`" |
| `NAME_TAKEN` | `1` | Rule 6 | "`<path>: the name "<name>" already belongs to <other path>`" |
| `IDENTITY_TERMS_MISSING` | `1` | Rule 7 | "`.identity-terms is missing; create it with one line per name, email address, user name, and handle of the maintainer`" |
| `GIT_IDENTITY_MISSING` | `1` | Rule 7 | "`git config user.name or user.email is empty`" |
| `IDENTITY_FOUND` | `1` | Rule 7 | "`<path>:<line>: holds an identity term of the maintainer`" |
| `IDENTITY_FOUND` | `1` | Rule 7 | "`<folder>: holds a file or folder named with an identity term of the maintainer`" |
| `GITLEAKS_MISSING` | `1` | Rule 8 | "`gitleaks is not on the PATH; install 8.30.1 from https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1`" |
| `GITLEAKS_VERSION` | `1` | Rule 8 | "`gitleaks <found> found; 8.30.1 is required`" |
| `SECRET_FOUND` | `1` | Rule 8 | "`gitleaks found <count> secrets in the history; its redacted report is above`" |
| `VAULT_NOTE_FOUND` | `1` | Rule 8 | "`<path> is a vault note; vault notes never enter this repository`" |

`<reason>` is the TOML parser's message, or one of: `no +++ block`, `unknown key "<key>"`,
`missing key "<key>"`, `"<key>" is outside its domain`.

## Side Effects

- **Persistence:** `build` replaces `dist/claude-code/` whole on success and removes it on failure; `scan`
  writes nothing.
- **Concurrency:** N/A — one maintainer runs one build at a time on a local clone. A file edited during a run
  is read in whichever state the run found it.
- **Transaction:** the whole of `dist/claude-code/` is the unit. `build` writes into `dist/.claude-code.tmp/`
  and renames it to `dist/claude-code/` only after the identity scan passes.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — Build the plugin (happy path)

```gherkin
Given content/ holds the 22 migrated pieces with every token resolvable
And .identity-terms exists and git config holds user.name and user.email
When the maintainer runs `cargo xtask build`
Then it exits 0 and prints the success output
And dist/claude-code/ holds .claude-plugin/plugin.json named "tightship"
And dist/claude-code/skills/ holds 21 folders and dist/claude-code/agents/ holds 1 file
And `claude plugin validate dist/claude-code/.claude-plugin/plugin.json` reports "Validation passed"
```

### Scenario 2 — One edit reaches the plugin (NFR-MNT-01.1)

```gherkin
Given a built dist/claude-code/
When the maintainer changes one sentence in content/skills/grilling/SKILL.md
And runs `cargo xtask build`
Then dist/claude-code/skills/grilling/SKILL.md holds the new sentence
And no other file under dist/claude-code/ changed
```

### Scenario 3 — Same input, same output

```gherkin
Given a built dist/claude-code/
When the maintainer runs `cargo xtask build` again without changing anything
Then every file under dist/claude-code/ is byte-identical to the previous run
```

### Scenario 4 — References render under the plugin's names (FR-SKL-03.1)

```gherkin
Given content/commands/spec.md holds {{skill:spec-writing}} and {{command:epic}}
When the maintainer runs `cargo xtask build`
Then dist/claude-code/skills/spec/SKILL.md holds "tightship:spec-writing" and "/tightship:epic"
And dist/claude-code/skills/spec-writing/ and dist/claude-code/skills/epic/ exist
```

### Scenario 5 — A capability renders the harness's text

```gherkin
Given content/skills/grilling/SKILL.md holds {{capability:plan-mode}}
When the maintainer runs `cargo xtask build`
Then dist/claude-code/skills/grilling/SKILL.md holds "plan mode" where the token was
```

### Scenario 6 — A token names a missing piece (exception)

```gherkin
Given content/commands/spec.md holds {{skill:spec-writer}} on line 12
And no folder content/skills/spec-writer/ exists
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/commands/spec.md:12: {{skill:spec-writer}} names no skill in content/"
And dist/claude-code/ does not exist
```

### Scenario 7 — Unknown token kind (exception)

```gherkin
Given a body holds {{script:vault}} on line 4
When the maintainer runs `cargo xtask build`
Then it exits 1 with "unknown token kind \"script\" in {{script:vault}}" for that line
```

### Scenario 8 — Undefined capability (exception)

```gherkin
Given a body holds {{capability:todo-list}} and harness.toml has no "todo-list" capability
When the maintainer runs `cargo xtask build`
Then it exits 1 with "capability \"todo-list\" has no entry in harnesses/claude-code/harness.toml"
```

### Scenario 9 — Malformed token (exception)

```gherkin
Given a body holds "{{skill grilling}}" on line 7
When the maintainer runs `cargo xtask build`
Then it exits 1 with "\"{{skill grilling}}\" is not a token; write {{<kind>:<name>}}" for that line
```

### Scenario 10 — Literal reference (exception)

```gherkin
Given content/commands/sprint.md holds "`/spec`" on line 30
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/commands/sprint.md:30: \"/spec\" refers to the command \"spec\"; write {{command:spec}}"
```

### Scenario 11 — A path like docs/specs/ is not a reference (alternative)

```gherkin
Given a body holds "docs/specs/<epic>/" and "docs/requirements/"
When the maintainer runs `cargo xtask build`
Then neither is reported as a literal reference
```

### Scenario 12 — Harness term in canonical content (NFR-FLEX-06.1, exception)

```gherkin
Given content/skills/grilling/SKILL.md holds "ExitPlanMode" on line 60
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds the HARNESS_TERM message for content/skills/grilling/SKILL.md:60, term "ExitPlanMode",
  harness "claude-code"
```

### Scenario 13 — A word that contains a term is not a match (alternative)

```gherkin
Given a body holds "cursor-based pagination" and "Explored"
When the maintainer runs `cargo xtask build`
Then no harness term is reported
```

### Scenario 14 — Invalid front matter (exception)

```gherkin
Given content/agents/adversarial-verifier.md has front matter with tools = "Read"
When the maintainer runs `cargo xtask build`
Then it exits 1 with "content/agents/adversarial-verifier.md: front matter: unknown key \"tools\""
```

### Scenario 15 — Name differs from its file (exception)

```gherkin
Given content/commands/spec.md has name = "specs"
When the maintainer runs `cargo xtask build`
Then it exits 1 with "content/commands/spec.md: name \"specs\" differs from its file name \"spec\""
```

### Scenario 16 — Name taken by two pieces (exception)

```gherkin
Given content/skills/audit/SKILL.md and content/commands/audit.md both exist
When the maintainer runs `cargo xtask build`
Then it exits 1 with "the name \"audit\" already belongs to" naming the other file
```

### Scenario 17 — Every error in one run (alternative)

```gherkin
Given one body holds {{skill:missing}} and another holds "ExitPlanMode"
When the maintainer runs `cargo xtask build`
Then stderr holds both errors, sorted by path and line
And it exits 1
```

### Scenario 18 — Identity term in a shipped file (NFR-SEC-01.1, exception)

```gherkin
Given .identity-terms lists a handle
And a canonical body holds that handle on line 3
When the maintainer runs `cargo xtask build`
Then it exits 1 with "<path>:3: holds an identity term of the maintainer"
And stderr does not hold the handle
And dist/claude-code/ does not exist
```

### Scenario 19 — Identity terms file missing (exception)

```gherkin
Given no .identity-terms at the repository root
When the maintainer runs `cargo xtask build`
Then it exits 1 with the IDENTITY_TERMS_MISSING message
```

### Scenario 20 — Git identity missing (exception)

```gherkin
Given git config user.email is empty for the repository
When the maintainer runs `cargo xtask build`
Then it exits 1 with "git config user.name or user.email is empty"
```

### Scenario 21 — Scan a clean repository (NFR-SEC-02.1, NFR-SEC-02.2, happy path)

```gherkin
Given gitleaks 8.30.1 is on the PATH
And the history holds no secret and the tree holds no vault note
When the maintainer runs `cargo xtask scan`
Then it exits 0 and prints "No secret in <n> commits. No vault note."
```

### Scenario 22 — Secret in the history (exception)

```gherkin
Given a commit on any branch added a file holding a GitHub token
When the maintainer runs `cargo xtask scan`
Then it exits 1 with "gitleaks found 1 secrets in the history; its redacted report is above"
And the report does not print the token
```

### Scenario 23 — gitleaks missing or another version (exception)

```gherkin
Given gitleaks is not on the PATH
When the maintainer runs `cargo xtask scan`
Then it exits 1 with "gitleaks is not on the PATH; install 8.30.1 from https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1"
```

```gherkin
Given gitleaks 8.29.1 is on the PATH
When the maintainer runs `cargo xtask scan`
Then it exits 1 with "gitleaks 8.29.1 found; 8.30.1 is required"
```

### Scenario 24 — Vault note in the tree (NFR-SEC-02.2, exception)

```gherkin
Given a tracked file docs/note.md whose front matter holds id: mem-0001
When the maintainer runs `cargo xtask scan`
Then it exits 1 with "docs/note.md is a vault note; vault notes never enter this repository"
```

### Scenario 25 — The migrated pieces are the baseline's (BR-02, BR-03)

```gherkin
Given the 22 baseline files at commit 1f41699
When each is compared with its canonical file, with the changes of rule 9 undone
Then the texts are equal
And the generated plugin holds the 13 skills, the 8 commands, and the subagent
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md), marked M1:

- **Solution strategy** — Rust, tokenized canonical content, per-harness places, and a generated plugin,
  linking ADR 0001.
- **Building blocks**, with a new container diagram (`diagrams/containers.drawio.svg`):
  - Canonical content `[Markdown + TOML]`, built in this epic.
  - Harness definitions `[TOML]`, built in this epic.
  - Build `[Rust, xtask]`, built in this epic.
  - Claude Code plugin `[generated directory]`, built in this epic.
  - `tightship` binary `[Rust]`, dashed, starting in epic #5.
  - Release pipeline `[GitHub Actions]`, dashed, epic #8.
- **Deployment** — how the plugin is packaged: an archive per harness with the binary for each target and the
  `sh` launcher; the adopter's prerequisites.
- **Crosscutting concepts** — references and capabilities as tokens; security: identity and secret scans,
  and the plugin's reach.
- **Architecture decisions** — links ADR 0001.

## Data model impact

N/A — this epic reads and writes no vault note and no option state. The README's line that waits on the M1
ADR for file formats now points at the epics that define them: options in #3, notes and handoffs in #5 and #6.

## Out of Scope

- **The `tightship` binary, hooks, and the vault script** — epic #5, Memory, where the first hook appears.
- **The `handoff` command** — epic #6, with the memory script it calls.
- **The memory item of `closeout`** — epic #5, for the same reason.
- **The contract** — epic #4.
- **The status line** — epic #7.
- **The vault template, and the part of NFR-SEC-02.2 that checks it holds rules only** — epic #5.
- **Installing from a catalog, the companion plugins, and checking each skill and command loads in a
  session** — epic #2.
- **The options** — epic #3.
- **The release: GitHub Actions, the archive, the catalog's pointer to it, and the macOS signing test** —
  epic #8.
- **The repository's license** — the plugin manifest leaves `license` out until epic #8 chooses one, since an
  adopter installs only from a release.
- **Harnesses other than Claude Code** — M2 to M4 ([BR-01](../../requirements/business-rules.md#br-01)).
- **Continuous integration for the gates** — the gates run on the maintainer's machine until epic #8 adds
  GitHub Actions.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on | Issue |
| --- | --- | --- | --- | --- | --- |
| 1 | Build a canonical piece into the Claude Code plugin | Cargo workspace (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`), `xtask/` (content reading, front matter, tokens, references, neutrality, Claude Code rendering, tests over fixtures), `harnesses/claude-code/harness.toml`, `.gitignore`, `content/skills/grilling/SKILL.md`, `AGENTS.md` (stack, gates, commands), about 18 files | Scenarios 2, 3 and 5 verified on `grilling`; 4 and 6 to 17 verified by tests over fixtures | — | #9 |
| 2 | Scan for identity, secrets, and vault notes | `xtask/` (identity scan as the last step of `build`, the `scan` command, tests), `AGENTS.md` (the `scan` gate), GitHub secret scanning and push protection turned on after the maintainer approves, about 6 files | Scenarios 18 to 24 verified | 1 | #10 |
| 3 | Migrate the 22 baseline pieces | `content/skills/` (12 more `SKILL.md` and 4 supporting files), `content/commands/` (8 files), `content/agents/adversarial-verifier.md`, `harness.toml` capabilities, about 27 files. Skills and commands cite each other, so neither half builds alone; the review reads the comparison with the baseline of scenario 25 | Scenarios 1, 4 and 25 verified on the 22 pieces | 2 | #11 |
