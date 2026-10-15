# #5 — Keep curated notes in the adopter's vault

> **Status:** closed
> **Profile:** API
> **Module:** `tightship/`, `content/`, `harnesses/claude-code/`, `xtask/`
> **Epic:** #5
> **Requirements:** FR-MEM-01, FR-MEM-02, FR-MEM-03, FR-MEM-04, FR-MEM-05, FR-MEM-08, FR-MEM-09, FR-MEM-10,
> NFR-REL-01, NFR-PERF-01, NFR-PERF-02
> **Sprint:** 3

The plugin ships its first program, the `tightship` binary. When the memory option is on and the adopter has
set a vault folder, a session starts with the vault's pinned notes and the project's handoff. The agent gets
a checkpoint every 10 prompts and after each compaction. It searches, writes, and lints notes with
`tightship vault`, and `tightship vault init` creates a new vault from the template.

Terms from the SRS, as used here:

- **Vault**: a git repository of curated notes, shared across machines and harnesses
  ([glossary](../../requirements/glossary.md)). It lives in the folder the adopter chose
  ([FR-MEM-09](../../requirements/functional/memory.md#fr-mem-09)).
- **Note**: one curated fact in the vault: a title, a type, the line that states why it matters, and a body
  ([data model](../../data-model/README.md)).
- **Scope**: the vault folder a note belongs to: one project's name, or `all`.
- **Pinned note**: a note marked to be listed at every session start.
- **Handoff**: the one note per project that carries a session's state to the next session. Epic #6 releases
  and claims it; this epic only shows it at session start
  ([FR-MEM-01.2](../../requirements/functional/memory.md#fr-mem-01)).
- **Checkpoint**: a reminder, placed in the agent's context, to record in the vault what the session learned
  ([FR-MEM-02](../../requirements/functional/memory.md#fr-mem-02)).
- **Baseline**: commit `1f41699` of the maintainer's private configuration. Its memory layer is the script
  `hooks/vault.py` and the template `vault-template/AGENTS.md`.
- **Option**: a part of the plugin the adopter turns on or off; this spec reads `memory` by rule 4 of the
  [epic #3 spec](../options/choose-the-options.md): off only when the value is exactly `false`.

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-15. They rest on Claude Code's hooks and plugin references and on a
probe with a throwaway plugin on Claude Code 2.1.299:

1. **The branch `feat/5-memory` is cut from `develop`**, after epic #4's PR merged. An epic's branch is never
   cut from another epic's branch.
2. **The checkpoint for a compaction arrives right after it.** A `PreCompact` hook adds nothing to the agent's
   context, and blocking an automatic compaction fails the request. So the checkpoint comes from a
   `SessionStart` hook on the source `compact`, as in the baseline. FR-MEM-02 changes to match, by a change
   request.
3. **The adopter sets the vault's folder in the plugin's settings.** `content/options.toml` declares
   `vault_folder`, a setting of type `directory` with no default, next to the three on/off options. Claude
   Code passes it only to hooks. The probe showed that a plugin's `SessionStart` hook gets `CLAUDE_ENV_FILE`,
   and that an `export` written there reaches the plugin's `bin/` through the Bash tool. So the session-start
   hook exports `TIGHTSHIP_VAULT`, and `tightship vault` reads it. The probe also showed that a `directory`
   setting the adopter never set reaches the hook as an absent variable.
4. **Notes keep the baseline's format**, so an existing vault works without a migration. Two differences: the
   binary writes no `harness` field, because it names no harness, and it generates no `index.md` and no
   per-scope hub, which no requirement asks for and which made every session start write to the vault.
5. **The checkpoint's prompt counter lives in the harness's plugin data folder**, at an interval of 10 prompts,
   each counter file deleted after 7 days. ADR 0001's reach is amended to add that folder and the session's
   environment file.
6. **`tightship vault init <folder>` creates a vault** when the agent runs it at the adopter's request. No new
   command piece. The template's `AGENTS.md` is canonical content, written harness-neutral and embedded in the
   binary.
7. **The `tightship` crate starts here.** `cargo xtask build` compiles it for the machine that builds and
   places it under `bin/`, behind the `sh` launcher `bin/tightship`. Every other target arrives with epic #8.
8. **A search lists the project's scope first**, as FR-MEM-03.2 requires. The baseline put first the notes that
   match the most terms.
9. Three tasks: the binary, the folder, and `init`; then the session start and the checkpoints; then writing,
   searching, and linting.

## Acceptance Criteria

### Contract

| Command | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| `cargo xtask build` | Maintainer | Yes | Also compiles `tightship` and writes `bin/` and the memory hooks into `dist/claude-code/` |
| A session start, `/clear`, or a compaction, with memory on | Adopter, through Claude Code | No: it writes the session's environment file | Prints the memory context and exports `TIGHTSHIP_VAULT` |
| A resumed or forked session, with memory on | Adopter, through Claude Code | Yes | Exports `TIGHTSHIP_VAULT` and prints nothing |
| A compaction, with memory on | Adopter, through Claude Code | Yes | Prints the checkpoint |
| Each prompt, with memory on | Adopter, through Claude Code | No: it counts the prompt | Prints the checkpoint on every 10th prompt of the session |
| `tightship vault path` | Agent | Yes | Prints the vault's folder |
| `tightship vault init <folder>` | Agent, at the adopter's request | No | Creates a vault from the template |
| `tightship vault search [--everywhere] <term>...` | Agent | Yes | Lists the notes that match |
| `tightship vault new …` | Agent | No | Writes one note |
| `tightship vault lint` | Agent or adopter | Yes | Reports every note that breaks a rule |
| `tightship --help`, or `tightship vault --help` | Agent | Yes | Prints the usage |

### Request

#### `content/options.toml`

Each entry gains the key `type`:

| Key | Type | Required | Validation |
| --- | --- | --- | --- |
| `type` | string | Yes | `boolean` or `directory`; else `OPTION_TYPE_UNKNOWN` |

A `boolean` entry renders into `userConfig` with `"default": true`, as epic #3 set. A `directory` entry
renders with no `default`. The file gains:

```toml
[[options]]
name = "vault_folder"
type = "directory"
title = "Vault folder"
description = "Your memory vault, a git repository; memory needs it, and tightship vault init creates one"
```

#### `content/vault-template/AGENTS.md`

New. The rules a new vault starts with: the baseline's `vault-template/AGENTS.md`, with only these changes:

| Baseline text | In the shipped template |
| --- | --- |
| The title "ai-memory — cross-harness persistent memory" | "Vault — cross-harness memory" |
| "shared by all AI coding harnesses (Claude Code, Codex, Cursor) across machines (macOS, Linux, Windows). It syncs through a private git repo and lives at `~/ai-memory` on every machine — resolve that path, never hardcode it (`~/.claude/SETUP.md` has the wiring)." | "shared by every coding agent across machines. It syncs through a private git repo and lives in the folder each machine set; `tightship vault path` prints it." |
| "Harness scratch (`~/.claude/projects/<project>/memory/`, `~/.codex/memories/`)" | "A harness's own per-project memory" |
| The layout line for `index.md`, the Read protocol line on `vault.py context` regenerating, and the Never line on `index.md` | Removed: the binary generates no index |
| `python3 ~/.claude/hooks/vault.py <subcommand>`, and "(`python` on Windows; the session start prints the exact command for the machine)" | `tightship vault <subcommand>` |
| "`vault.py handoff release --title "..."` … `vault.py handoff claim` … In Claude Code, `/handoff release\|claim` writes the body and reads the repo; any other harness runs the two commands from the project's directory." | "The handoff command releases and claims it." Epic #6 builds that command. |
| The front matter line `harness: claude-code` | Removed |
| "id: mem-0042 # mem-NNNN" | "id: mem-0042 # mem- and at least four digits" |
| The Lint item "Sweep the harness scratch silos" | "Sweep each harness's own per-project memory" |

It holds no token: the binary embeds it as written. The neutrality, identity, and literal-reference checks
of epic #1 read it like every other file of `content/`.

#### `tightship vault new`

| Argument | Type | Required | Validation |
| --- | --- | --- | --- |
| `--scope` | string | Yes | 1 to 100 characters, with no `/`, no `\`, and no leading `.` |
| `--type` | string | Yes | `preference`, `decision`, `gotcha`, or `project` |
| `--title` | string | Yes | 1 to 120 characters |
| `--why` | string | Yes | At least 5 words, split on whitespace |
| `--source` | string | Yes | `user`, `session`, or `repo` |
| `--tags` | string | No | Comma-separated; each tag trimmed, empty ones dropped |
| `--pin` | flag | No | Marks the note pinned |
| `--created` | string | No | A date `YYYY-MM-DD`; today when absent |
| Body | stdin | Yes | Not empty after trimming; never read from a terminal |

```bash
tightship vault new --scope tightship --type gotcha --title "Hooks get CLAUDE_ENV_FILE" \
  --why "without it a plugin's bin cannot find a value only hooks receive" \
  --tags hooks,env --source session <<'EOF'
A plugin's SessionStart hook can export variables for the Bash tool.
EOF
```

#### `tightship vault search`

| Argument | Type | Required | Validation |
| --- | --- | --- | --- |
| `<term>` | string, repeated | Yes | At least one term that is not blank |
| `--everywhere` | flag | No | Also searches the scopes other than the project's and `all` |

#### `tightship vault init`

| Argument | Type | Required | Validation |
| --- | --- | --- | --- |
| `<folder>` | path | Yes | Relative to the working directory, or absolute; absent or an empty folder |

#### The hooks' input

The hook command lines, rendered by the build for Claude Code, hand the binary only `TIGHTSHIP_*` variables,
so the binary names no harness:

| Variable | From | Read by |
| --- | --- | --- |
| `TIGHTSHIP_VAULT` | The `vault_folder` setting; absent when the adopter never set it | Every `hook` subcommand; `vault` subcommands read it from the session's environment |
| `TIGHTSHIP_ENV_FILE` | The session's environment file; absent where the harness gives none | `hook start` and `hook export` |
| `TIGHTSHIP_DATA` | The plugin's data folder | `hook prompt` |

`hook prompt` reads the field `session_id` of the JSON object the harness writes on stdin. The project is the
working directory the hook runs in.

### Response

#### The note

`tightship vault new` writes `<vault>/notes/<scope>/<id>-<slug>.md`, where `<id>` is `mem-` and the highest
number in the vault plus one, at least four digits, and `<slug>` is the title folded to ASCII, lowercased, every
run of other characters turned into one `-`, trimmed of `-`, and cut at 60 characters. An empty slug leaves
the name `<id>.md`.

```markdown
---
id: mem-0632
type: gotcha
project: tightship
why: without it a plugin's bin cannot find a value only hooks receive
tags: [hooks, env]
source: session
created: 2026-10-15
updated: 2026-10-15
---

# Hooks get CLAUDE_ENV_FILE

A plugin's SessionStart hook can export variables for the Bash tool.
```

Fields are written in the order `id`, `type`, `project`, `why`, `tags`, `pinned`, `source`, `created`, and
`updated`. Each empty one is left out, and `pinned: true` is written only with `--pin`. Dates are the local
date of the machine. Stdout prints the path relative to the vault: `notes/tightship/mem-0632-hooks-get-claude-env-file.md`.

#### The memory context

`hook start` prints, at most 9,000 characters counted in UTF-16 code units:

```markdown
# Vault — curated cross-harness memory

Scope `<scope>` and `all`. Before non-trivial work, search them for the subsystem and the task:
`tightship vault search <term> <term>` — one line per match, its `why` stating
what not knowing it costs. Open one at `<vault>/<path>`. Rules: `<vault>/AGENTS.md`.
Listed below, only the pinned notes. A hint to verify against the repo, never truth.
Write with `tightship vault new --help`.

## Handoff — <scope> · <id> · <age>

<the handoff's body>

## Scope — <scope>

- [[<file stem>]] (<age>) — <why>

## Scope — all

- [[<file stem>]] (<age>) — <why>

[<n> more notes in these scopes — `tightship vault search <terms>`]
[other scopes: <scope>(<n>) <scope>(<n>) — `tightship vault search --everywhere <terms>`]
[<n> pinned notes cut at the 9000-character cap — `tightship vault lint` names the scope]
```

`<age>` is the `created` date, or `<created> → <updated>` when they differ. A section with nothing to list is
left out with its heading, and each bracketed line appears only when its count is above 0. The scopes in
"other scopes" are in alphabetical order.

#### The checkpoint

Printed by `hook checkpoint`, and by `hook prompt` on every 10th prompt:

```text
MEMORY CHECKPOINT — did anything this session become a durable fact?

Only these enter the vault:
1. A rule or preference the user stated — type `preference`.
2. A decision plus its why, one that outlives this task — type `decision`.
3. A trap that cost real time and will recur — type `gotcha`.
4. A project fact that lives in no repo, not the code, not git, not the docs — type `project`.

Everything else stays out: derivable from the repo, session state, task progress, whatever merely
seemed interesting. The test is the `why` field — if you cannot write in one line what not knowing
it would cost later, it does not qualify.

To write one (single step, then tell the user in one line what landed):
  tightship vault new --scope <project|all> --type <preference|decision|gotcha|project> \
    --title "..." --why "..." --tags a,b --source <user|session|repo> <<'EOF'
  <body>
  EOF

The note is then reached with `tightship vault search`, which reads the title, the `why`, the tags and the
body — so put in them the words someone hitting this again would type. Add `--pin` only for a
standing fact the work needs before it knows to search: pinned notes are listed at every session
start, a space that holds about thirty.

Nothing durable? Ignore this and carry on.
```

#### A search

```text
<shown> of <total> matching notes, paths under `<vault>/`:
- notes/<scope>/<file> — <why>
```

At most 10 lines. With no match: `no note matches <term> · <term> — try other terms, or add --everywhere for
the other scopes`, without the part after "terms" when `--everywhere` was given. Exit 0 in both cases.

#### A lint

One block per note with problems, its path relative to the vault, then one line per problem:

```text
notes/tightship/mem-0012-a-note.md
  - missing `why`
scope `tightship`: 3 pinned notes do not fit the session start — unpin them, move them to the repo, or delete them
12 notes, 1 with problems, 1 scopes over the session start
```

Exit 0 when no note has a problem and every scope fits; exit 1 otherwise.

#### What `tightship vault init` prints

Creates `<folder>/AGENTS.md` from the template and the empty folder `<folder>/notes/all/`, then runs `git init`
in `<folder>`. Prints:

```text
Created the vault at <absolute folder>. Commit it, push it to a private remote to share it across machines,
and set this folder as the plugin's vault folder.
```

#### What `tightship vault path` prints

Prints the vault's absolute folder and a newline.

#### The usage

```text
Usage: tightship vault path
       tightship vault init <folder>
       tightship vault search [--everywhere] <term>...
       tightship vault new --scope <project|all> --type <preference|decision|gotcha|project>
                           --title <title> --why <line> --source <user|session|repo>
                           [--tags <a,b>] [--pin] [--created <YYYY-MM-DD>] < body
       tightship vault lint
```

#### `dist/claude-code/`

| File | Holds |
| --- | --- |
| `bin/tightship` | The `sh` launcher: it runs `bin/<platform>/tightship` for `uname -s` and `uname -m`, with `<platform>` `macos`, `linux-x64`, `linux-arm64`, or `windows-x64` (`tightship.exe`, for `MINGW*`, `MSYS*`, and `CYGWIN*`) |
| `bin/<platform>/tightship` | The binary for the machine that ran the build |
| `hooks/hooks.json` | The contract's group, then the memory hooks below |
| `.claude-plugin/plugin.json` | `userConfig` gains `vault_folder`, `"type": "directory"`, with no `default` |

The memory hooks, written when `content/options.toml` declares `memory`. Each line opens with the test
`[ "$CLAUDE_PLUGIN_OPTION_MEMORY" = false ] ||`, left out of the table:

| Event | Matcher | Command, after the test |
| --- | --- | --- |
| `SessionStart` | `startup\|clear\|compact` | `TIGHTSHIP_VAULT="$CLAUDE_PLUGIN_OPTION_VAULT_FOLDER" TIGHTSHIP_ENV_FILE="$CLAUDE_ENV_FILE" "${CLAUDE_PLUGIN_ROOT}/bin/tightship" hook start` |
| `SessionStart` | `resume\|fork` | `TIGHTSHIP_VAULT="$CLAUDE_PLUGIN_OPTION_VAULT_FOLDER" TIGHTSHIP_ENV_FILE="$CLAUDE_ENV_FILE" "${CLAUDE_PLUGIN_ROOT}/bin/tightship" hook export` |
| `SessionStart` | `compact` | `TIGHTSHIP_VAULT="$CLAUDE_PLUGIN_OPTION_VAULT_FOLDER" "${CLAUDE_PLUGIN_ROOT}/bin/tightship" hook checkpoint` |
| `UserPromptSubmit` | none | `TIGHTSHIP_VAULT="$CLAUDE_PLUGIN_OPTION_VAULT_FOLDER" TIGHTSHIP_DATA="$CLAUDE_PLUGIN_DATA" "${CLAUDE_PLUGIN_ROOT}/bin/tightship" hook prompt` |

With the baseline's contract and the memory option declared, the build's success output reads:

```text
Built dist/claude-code/: 22 pieces (13 skills, 8 commands, 1 agent), 35 files.
Try it: claude --plugin-dir dist/claude-code
```

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | Edits the template and the binary, and runs the build |
| Adopter | None from this project | Owns the vault and sets the memory option and the vault folder in their own Claude Code configuration |
| Agent | The Bash tool of the adopter's session | Runs `tightship vault`; the binary checks no permission of its own |

---

## Business Rules

### 1. Authorization

N/A. The binary runs in the adopter's own session, on the adopter's own vault, with no service of this project
behind it. Who may run it is whoever may run the Bash tool there.

### 2. The vault is the folder the adopter set

- **When:** every `hook` and `vault` subcommand, before it reads or writes a note.
- A `hook` subcommand reads `TIGHTSHIP_VAULT` from its command line; a `vault` subcommand reads it from the
  session's environment, where `hook start` or `hook export` put it. Nothing else names a vault: there is no
  default folder (FR-MEM-09).
- The value must be an absolute path to an existing folder. `hook start` and `hook export` write
  `export TIGHTSHIP_VAULT='<folder>'` to the end of `TIGHTSHIP_ENV_FILE` only when it is, each `'` in the
  folder written as `'\''`, and write nothing when `TIGHTSHIP_ENV_FILE` is absent.
- A `vault` subcommand without a usable value fails with `VAULT_NOT_SET`, `VAULT_NOT_ABSOLUTE`, or
  `VAULT_MISSING`.

### 3. The session starts with the notes that matter to the project

- **When:** `hook start`, on the sources `startup`, `clear`, and `compact`. Not on `resume` and `fork`, whose
  context already holds the earlier print.
- **The project's scope:** the name of the git repository's top folder for the working directory, or of the
  working directory when it is in no repository, lowercased, its leading dots removed. When `notes/` holds
  a folder of that name, that is the scope. Otherwise, the longest folder name other than `all` that is a
  prefix of the name followed by `-` or `_`. Otherwise, the name itself.
- **Notes in reach:** the notes whose `project` is the scope or `all`. A note is a `.md` file under `notes/`
  that opens with a `---` line and closes its front matter with another. Any other file is not a note and is
  left out.
- **The handoff:** of the notes in reach of type `handoff` and in the project's scope, the one with the
  latest `updated`, then the highest `id`. When the header and the handoff do not fit in 9,000 characters, the
  handoff's section holds only the line `The handoff is <n> characters, more than the session start holds;
  read it at <vault>/<path>.`.
- **Pinned notes:** the notes in reach, other than the handoff, with `pinned: true`, the project's scope first
  and then `all`, each by `id`. They are listed while the text stays within 9,000 characters, keeping 200 for
  the bracketed lines. Each note that does not fit is counted in the line about the cap.
- The rest of the text is the Response's memory context, and Claude Code adds it to the context
  (FR-MEM-01.1, FR-MEM-01.3).

### 4. A session without a usable vault says so and goes on

- **When:** `hook start`, with the memory option on.
- `TIGHTSHIP_VAULT` absent or empty prints, and exits 0:
  > "# Vault\n\nMemory is on and no vault folder is set, so this session has no vault. When the user asks
  > about memory, tell them to create one with `tightship vault init <folder>`, set that folder as the
  > plugin's vault folder, and start a new session, or to turn the memory option off."
- A value that is not absolute prints, and exits 0:
  > "# Vault\n\nMemory is on and the vault folder `<value>` is not an absolute path, so this session has no
  > vault. When the user asks about memory, tell them to set the plugin's vault folder to an absolute path."
- A folder that does not exist prints, and exits 0:
  > "# Vault\n\nMemory is on and the vault folder `<folder>` does not exist, so this session has no vault.
  > When the user asks about memory, tell them to create it with `tightship vault init <folder>`, or to set
  > another folder as the plugin's vault folder."
- A folder the binary cannot read fails with `VAULT_UNREADABLE`, and a note it cannot read with
  `NOTE_UNREADABLE`. Claude Code shows the error, and the session starts (NFR-REL-01).

### 5. Checkpoints

- **Every 10th prompt:** `hook prompt` adds 1 to the count kept in `<TIGHTSHIP_DATA>/checkpoints/<session>`,
  where `<session>` is `session_id` with each character other than ASCII letters, digits, `_`, and `-` turned
  into `-`. When the count is a multiple of 10, it prints the checkpoint (FR-MEM-02.1). A missing
  `session_id` counts under `unknown`. A file not modified for 7 days is deleted on the next prompt.
- **After a compaction:** `hook checkpoint` prints the checkpoint on the source `compact`, after the
  compaction's summary (FR-MEM-02.2, as the change request words it).
- With `TIGHTSHIP_VAULT` absent or empty, neither prints: the session has no vault to write to.
- A counter that cannot be read as a number starts again from 0. A data folder the binary cannot write fails
  with `DATA_UNWRITABLE`, and the prompt goes ahead.

### 6. Memory off runs nothing

- **When:** any memory hook, with `CLAUDE_PLUGIN_OPTION_MEMORY` exactly `false`.
- The hook's `sh` test stops it before the binary starts: no context, no checkpoint, no counter, no export
  (FR-MEM-10.1). `TIGHTSHIP_VAULT` stays out of the session's environment, so `tightship vault` fails with
  `VAULT_NOT_SET`.

### 7. A search finds the notes by their words

- **When:** `tightship vault search`.
- Each term and each note are folded: case removed, runs of whitespace made one space. A note matches when its
  `why`, its tags, or its body, title line included, holds at least one term.
- Without `--everywhere`, only the notes in reach of rule 3 are searched. With it, every note.
- Order: the project's scope, then `all`, then the other scopes (FR-MEM-03.2). Within each, the most terms
  matched, then the most occurrences, then `id`.
- The first 10 are shown, in the Response's form (FR-MEM-03.1).

### 8. A note is written in one step

- **When:** `tightship vault new`.
- Every argument is checked as the Request says, in the order of its table, and the first that fails is
  reported. Nothing is written then.
- The note is written as the Response says, `notes/<scope>/` created when absent (FR-MEM-04.1). A file
  already at that path fails with `NOTE_EXISTS`.
- Two machines that write without pulling can give two notes the same `id`. The lint reports it, and the
  template tells the agent to pull before writing.

### 9. The lint checks every note against the template's rules

- **When:** `tightship vault lint`.
- For each note, in path order, each problem is reported in this order and with this text:

| Rule | Line |
| --- | --- |
| Each of `id`, `type`, `project`, `why`, `source`, `created`, `updated` is present and not empty | ``missing `<field>` `` |
| `type` is `project`, `gotcha`, `decision`, `preference`, or `handoff` | ``type `<value>` is not one of project, gotcha, decision, preference, handoff`` |
| `source` is `user`, `session`, or `repo` | ``source `<value>` is not one of user, session, repo`` |
| `id` is `mem-` and four or more digits | ``id `<value>` is not mem- and four or more digits`` |
| No earlier note, in path order, has the same `id` | ``id `<value>` already used by <file name of the earlier note>`` |
| `project` is the name of the note's folder | ``project `<value>` disagrees with folder `<folder>` `` |
| `why` has at least 5 words | `why names a topic instead of stating what ignoring it costs` |
| Each `[[<target>]]` outside code is the stem of a `.md` file of the vault, outside `.git/` | ``[[<target>]] points at no note`` |
| `pinned` is `true` or absent | ``pinned `<value>` is neither `true` nor absent`` |

- Then, for each scope in alphabetical order, whether its pinned notes fit rule 3's session start.
- The summary line closes the report (FR-MEM-05.1, FR-MEM-05.2).

### 10. A new vault starts from the template

- **When:** `tightship vault init <folder>`.
- With git absent from the `PATH`, it fails with `INIT_NO_GIT` before creating anything.
- A path that exists and is not an empty folder fails with `INIT_NOT_EMPTY`, and nothing changes
  (FR-MEM-08.2).
- Otherwise it creates the folder and its parents, `AGENTS.md`, and `notes/all/`, and runs `git init`
  (FR-MEM-08.1). The adopter commits and adds the remote.
- A `git init` that fails reports `INIT_GIT_FAILED` and leaves what it created.

### 11. A failure never blocks the session

- The binary never exits with 2, the code that makes Claude Code block a prompt or a compaction. A usage
  error exits 1, and so does a panic, which the binary reports as `INTERNAL`.
- A launcher that finds no binary for the machine exits 1 with
  `error: tightship has no binary for <uname -s> <uname -m>`. Claude Code shows it, and the session goes on.

### 12. Each hook stays within its time budget

- `hook start` takes at most 500 ms at the 95th percentile, over 20 session starts with every option on and a
  vault of 1,000 notes, together with the contract's hooks (NFR-PERF-01).
- `hook prompt` takes at most 100 ms at the 95th percentile, over 100 prompts (NFR-PERF-02).
- No hook uses the network.

### 13. The binary carries no identity

- The build compiles `tightship` with every path under the maintainer's home folder remapped, so the
  binary holds no home path. The identity scan of epic #1 reads `bin/` like every other file of `dist/`.

### 14. Persistence and audit

- **Files changed:** in the vault, the note `vault new` writes and the files `vault init` creates. In the
  plugin's data folder, `checkpoints/<session>`. In the session's environment file, one `export` line per
  session start or resume. The build writes `bin/` and `hooks/hooks.json` into `dist/claude-code/`.
- **Audit:** N/A. The vault is a git repository: its history records every note.
- **Events/integrations fired:** N/A. Nothing in this epic uses the network; pulling and pushing the vault is
  the adopter's, and epic #6's for a handoff.

---

## Errors

Every message goes to stderr, prefixed by `error:` and a space. The `vault` subcommands exit 1, and the `hook`
subcommands exit 1 too, which Claude Code reports without blocking.

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `VAULT_NOT_SET` | `1` | Rule 2 | "`` no vault is set for this session; turn memory on, set the plugin's vault folder, and start a new session ``" |
| `VAULT_NOT_ABSOLUTE` | `1` | Rule 2 | "`` the vault folder <value> is not an absolute path ``" |
| `VAULT_MISSING` | `1` | Rule 2 | "`` the vault folder <folder> does not exist; create it with `tightship vault init <folder>` ``" |
| `VAULT_UNREADABLE` | `1` | Rules 2 and 4 | "`` cannot read the vault folder <folder>: <the system's message> ``" |
| `NOTE_UNREADABLE` | `1` | Rules 3, 7, and 9 | "`` cannot read <path relative to the vault>: <the system's message> ``" |
| `SEARCH_NO_TERM` | `1` | Rule 7 | "`` search needs at least one term ``" |
| `NOTE_SCOPE_INVALID` | `1` | Rule 8 | "`` --scope must be 1 to 100 characters, with no / or \ and no leading dot ``" |
| `NOTE_TYPE_INVALID` | `1` | Rule 8 | "`` --type must be one of preference, decision, gotcha, project ``" |
| `NOTE_TITLE_INVALID` | `1` | Rule 8 | "`` --title must be 1 to 120 characters ``" |
| `NOTE_WHY_SHORT` | `1` | Rule 8 | "`` --why must state in a full line what not knowing this would cost later, not name a topic (got <n> words) ``" |
| `NOTE_SOURCE_INVALID` | `1` | Rule 8 | "`` --source must be one of user, session, repo ``" |
| `NOTE_CREATED_INVALID` | `1` | Rule 8 | "`` --created must be a date as YYYY-MM-DD ``" |
| `NOTE_BODY_MISSING` | `1` | Rule 8 | "`` the note's body must arrive on stdin ``" |
| `NOTE_EXISTS` | `1` | Rule 8 | "`` <path relative to the vault> already exists ``" |
| `INIT_NO_GIT` | `1` | Rule 10 | "`` git is not installed; a vault is a git repository, so nothing was created ``" |
| `INIT_NOT_EMPTY` | `1` | Rule 10 | "`` <folder> already exists and is not an empty folder; nothing changed ``" |
| `INIT_GIT_FAILED` | `1` | Rule 10 | "`` git init failed in <folder>: <git's message> ``" |
| `DATA_UNWRITABLE` | `1` | Rule 5 | "`` cannot write the checkpoint counter <path>: <the system's message> ``" |
| `USAGE` | `1` | An unknown subcommand, an unknown argument, a flag with no value, or a required flag missing | "`unknown subcommand <name>`", "`unknown argument <argument>`", "`<flag> needs a value`", or "`missing <flag>`", then a blank line and the usage |
| `INTERNAL` | `1` | Rule 11, a panic | "`tightship stopped on an internal fault: <the panic's message>`" |
| `LINT_FAILED` | `1` | Rule 9 | No stderr: the report on stdout is the message |
| `OPTION_TYPE_UNKNOWN` | `1` | The build, `content/options.toml` | "`content/options.toml: the type of "<name>" must be boolean or directory`" |

`<the system's message>` is the operating system's text for the error.

## Side Effects

- **Persistence:** a note is written whole by one write call. `vault init` writes into a folder it checked was
  absent or empty.
- **Concurrency:** two `vault new` at once in one vault can pick the same `id`; the second write then finds no
  file at its path, since slugs differ, and the lint reports the duplicate `id`. Two prompts of one session do
  not run their hooks at once. Two sessions keep separate counters.
- **Transaction:** none across files. The build writes `bin/` with the rest of `dist/.claude-code.tmp/`, published
  by the same rename as in epic #1.

---

## Acceptance Scenarios (Gherkin)

Scenarios 1 to 22 and 29 to 31 run as tests over vaults created in a temporary folder, never over files
tracked in this repository, which the scan of epic #1 would reject. Scenarios 23 to 28 run in sessions of an
isolated Claude Code configuration.

### Scenario 1 — Build the binary and the memory hooks (happy path)

```gherkin
Given content/options.toml declares contract, memory, status_line, and vault_folder
When the maintainer runs `cargo xtask build`
Then it exits 0 and reports 35 files
And dist/claude-code/bin/tightship runs the binary built for this machine
And plugin.json's userConfig holds vault_folder with "type": "directory" and no default
And hooks/hooks.json holds the contract's group and the four memory hooks of the Response
And `claude plugin validate dist/claude-code/.claude-plugin/plugin.json` reports "Validation passed" with no warning
And the identity scan finds no term in dist/claude-code/bin/
```

### Scenario 2 — An unknown option type (exception)

```gherkin
Given content/options.toml declares an option with type "string"
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: content/options.toml: the type of \"<name>\" must be boolean or directory"
```

### Scenario 3 — Create a vault (FR-MEM-08.1, happy path)

```gherkin
Given no folder at /tmp/<test>/vault
When the agent runs `tightship vault init /tmp/<test>/vault`
Then it exits 0 and prints "Created the vault at /tmp/<test>/vault. …"
And the folder holds AGENTS.md equal to content/vault-template/AGENTS.md, an empty notes/all/, and .git/
And it holds no note
```

### Scenario 4 — A vault already there (FR-MEM-08.2, exception)

```gherkin
Given /tmp/<test>/vault holds AGENTS.md
When the agent runs `tightship vault init /tmp/<test>/vault`
Then it exits 1
And stderr holds "error: /tmp/<test>/vault already exists and is not an empty folder; nothing changed"
And AGENTS.md is unchanged
```

### Scenario 5 — Git missing (exception)

```gherkin
Given a PATH with no git
When the agent runs `tightship vault init /tmp/<test>/vault`
Then it exits 1 with "error: git is not installed; a vault is a git repository, so nothing was created"
And /tmp/<test>/vault does not exist
```

### Scenario 6 — The folder the adopter chose (FR-MEM-09.1)

```gherkin
Given TIGHTSHIP_VAULT is /tmp/<test>/elsewhere, a vault
And another vault exists at /tmp/<test>/home/ai-memory, the baseline's default under HOME=/tmp/<test>/home
When the agent runs `tightship vault path`, then `tightship vault new …`
Then path prints "/tmp/<test>/elsewhere"
And the note is written under /tmp/<test>/elsewhere/notes/ and nothing under /tmp/<test>/home changes
```

### Scenario 7 — No vault set (exception)

```gherkin
Given TIGHTSHIP_VAULT is absent
When the agent runs `tightship vault search hooks`
Then it exits 1
And stderr holds "error: no vault is set for this session; turn memory on, set the plugin's vault folder, and start
  a new session"
```

### Scenario 8 — A vault folder that is gone (exception)

```gherkin
Given TIGHTSHIP_VAULT is /tmp/<test>/gone, which does not exist
When the agent runs `tightship vault path`
Then it exits 1 with "error: the vault folder /tmp/<test>/gone does not exist; create it with `tightship vault init
  /tmp/<test>/gone`"
```

### Scenario 9 — The session start lists the project's pinned notes (FR-MEM-01.1, FR-MEM-01.3, happy path)

```gherkin
Given a vault with pinned notes in the scopes "tightship", "all", and "other", and an unpinned note in "tightship"
And the working directory is a git repository whose top folder is "tightship"
And TIGHTSHIP_ENV_FILE is a file in the test's folder
When `tightship hook start` runs
Then it exits 0 and prints the memory context with scope `tightship`
And it lists the pinned note of "tightship" under "## Scope — tightship", then the one of "all" under "## Scope — all"
And it does not list the note of "other", and prints "[other scopes: other(1) — …]"
And it prints "[1 more notes in these scopes — `tightship vault search <terms>`]"
And the environment file ends with "export TIGHTSHIP_VAULT='<vault>'"
```

### Scenario 10 — The handoff at session start (FR-MEM-01.2)

```gherkin
Given a vault with a note of type handoff in the scope "tightship"
When `tightship hook start` runs in the project "tightship"
Then the memory context holds "## Handoff — tightship · <id> · <age>" followed by the handoff's body
And the handoff is not listed among the pinned notes
```

### Scenario 11 — Pinned notes past the cap (alternative)

```gherkin
Given a scope "tightship" with pinned notes whose lines add up to 12,000 characters
When `tightship hook start` runs in the project "tightship"
Then the print is at most 9,000 UTF-16 code units
And it ends with "[<n> pinned notes cut at the 9000-character cap — `tightship vault lint` names the scope]"
```

### Scenario 12 — A project folder named after a scope prefix (alternative)

```gherkin
Given a vault with the scope folders "tightship" and "all"
When `tightship hook start` runs in a repository whose top folder is "tightship-docs"
Then the scope is "tightship"
```

### Scenario 13 — No vault, a relative vault, a missing vault (NFR-REL-01.1, exception)

```gherkin
Given TIGHTSHIP_VAULT is, in turn, absent, "vault", and /tmp/<test>/gone
When `tightship hook start` runs
Then each run exits 0 and prints rule 4's message for that case
And the environment file receives no export
```

### Scenario 14 — A malformed note (NFR-REL-01.1, alternative)

```gherkin
Given a vault where one file under notes/ has no front matter, and one note has no closing --- line
When `tightship hook start` runs
Then it exits 0 and lists the other pinned notes
```

### Scenario 15 — A resumed session gets the export only (alternative)

```gherkin
Given a vault set in TIGHTSHIP_VAULT
When `tightship hook export` runs
Then it prints nothing
And the environment file ends with "export TIGHTSHIP_VAULT='<vault>'"
```

### Scenario 16 — The checkpoint every 10 prompts (FR-MEM-02.1)

```gherkin
Given a vault set in TIGHTSHIP_VAULT and TIGHTSHIP_DATA set to an empty folder
When `tightship hook prompt` runs 20 times with {"session_id":"abc"} on stdin
Then runs 10 and 20 print the checkpoint, and the other 18 print nothing
And <TIGHTSHIP_DATA>/checkpoints/abc holds "20"
And a counter file of another session, modified 8 days ago, is deleted
```

### Scenario 17 — The checkpoint after a compaction (FR-MEM-02.2)

```gherkin
Given a vault set in TIGHTSHIP_VAULT
When `tightship hook checkpoint` runs
Then it prints the checkpoint and exits 0
```

### Scenario 18 — Memory off runs nothing (FR-MEM-10.1)

```gherkin
Given the four memory hook lines from the built hooks.json
When each runs under sh with CLAUDE_PLUGIN_OPTION_MEMORY=false
Then each prints nothing and exits 0, and no checkpoint counter or environment file line is written
And with CLAUDE_PLUGIN_OPTION_MEMORY absent, the hook start line prints the memory context
```

### Scenario 19 — Search (FR-MEM-03.1, FR-MEM-03.2, happy path)

```gherkin
Given a note in "all" whose title holds "launcher" twice and whose why holds "uname"
And a note in "tightship" whose body holds "launcher" once
When the agent runs `tightship vault search launcher uname` in the project "tightship"
Then it prints "2 of 2 matching notes, paths under `<vault>/`:"
And the note of "tightship" comes first, each line with its path and its why
```

### Scenario 20 — Search with no match (alternative)

```gherkin
When the agent runs `tightship vault search zeppelin`
Then it exits 0 and prints "no note matches zeppelin — try other terms, or add --everywhere for the other scopes"
```

### Scenario 21 — Write a note (FR-MEM-04.1, happy path, and exceptions)

```gherkin
Given a vault whose highest id is mem-0631
When the agent runs the Request's `tightship vault new` example
Then it prints "notes/tightship/mem-0632-hooks-get-claude-env-file.md"
And the file holds the Response's note, with today's local date
When the agent runs it again with --why "about hooks"
Then it exits 1 with "error: --why must state in a full line what not knowing this would cost later, not name a
  topic (got 2 words)" and writes nothing
When the agent runs it with --scope "../x"
Then it exits 1 with "error: --scope must be 1 to 100 characters, with no / or \\ and no leading dot"
When the agent runs it with no stdin
Then it exits 1 with "error: the note's body must arrive on stdin"
```

### Scenario 22 — Lint (FR-MEM-05.1, FR-MEM-05.2)

```gherkin
Given a vault where every note follows rule 9
When the agent runs `tightship vault lint`
Then it exits 0 and prints "<n> notes, 0 with problems, 0 scopes over the session start"
Given one note has no why line, and two notes share id mem-0007
When the agent runs `tightship vault lint`
Then it exits 1
And the report names the first note and "missing `why`", and the second of the pair and "id `mem-0007` already
  used by <file>"
```

### Scenario 23 — A session starts with the vault (FR-MEM-01.1, happy path)

```gherkin
Given the plugin is installed with memory on and the vault folder set to a vault of pinned notes
When a session starts in a project of that vault
Then the agent's context holds the memory context with that project's pinned notes
And `tightship vault path`, run through the Bash tool, prints the vault's folder
```

### Scenario 24 — A resumed session still finds the vault (alternative)

```gherkin
Given a session from scenario 23, ended
When the adopter resumes it
Then the agent's context holds the memory context once
And `tightship vault path`, run through the Bash tool, prints the vault's folder
```

### Scenario 25 — The checkpoint reaches the agent (FR-MEM-02.1, FR-MEM-02.2)

```gherkin
Given a session from scenario 23
When the adopter sends 10 prompts, then runs /compact
Then the agent's context holds the checkpoint after the 10th prompt, and again after the compaction
```

### Scenario 26 — Memory off (FR-MEM-10.1)

```gherkin
Given the plugin is installed with memory set to false and the vault folder set
When a session starts and the adopter sends 10 prompts
Then the agent's context holds no memory context and no checkpoint
And `tightship vault path`, run through the Bash tool, fails with "error: no vault is set for this session; …"
```

### Scenario 27 — A missing vault does not stop the session (NFR-REL-01.1, exception)

```gherkin
Given the plugin is installed with memory on and the vault folder set to a folder that was deleted
When a session starts and the adopter sends a prompt
Then the session starts and answers
And the agent's context holds rule 4's message for a missing folder
```

### Scenario 28 — The time budgets (NFR-PERF-01.1, NFR-PERF-02.1)

```gherkin
Given a vault of 1,000 notes and every option on
When the plugin's SessionStart hooks run 20 times, and its UserPromptSubmit hook runs 100 times
Then the 95th percentile of the session start's hooks, together, is 500 ms or less
And the 95th percentile of the prompt hook is 100 ms or less
```

### Scenario 29 — Arguments of `new` and `search` rejected (exception)

```gherkin
Scenario Outline: a bad argument writes nothing
  Given a vault set in TIGHTSHIP_VAULT
  When the agent runs <command>
  Then it exits 1 with "error: <message>"
  And the vault is unchanged

  Examples:
    | command                                                       | message                                                       |
    | `tightship vault search "  "`                                 | search needs at least one term                                |
    | `tightship vault new` with --type handoff                     | --type must be one of preference, decision, gotcha, project   |
    | `tightship vault new` with --title of 121 characters          | --title must be 1 to 120 characters                           |
    | `tightship vault new` with --source chat                      | --source must be one of user, session, repo                   |
    | `tightship vault new` with --created 15/10/2026               | --created must be a date as YYYY-MM-DD                        |
    | `tightship vault new` whose file name already exists          | notes/tightship/mem-0632-hooks-get-claude-env-file.md already exists |
```

### Scenario 30 — Usage, a relative vault, and a failing git (exception)

```gherkin
Given TIGHTSHIP_VAULT is "vault"
When the agent runs `tightship vault path`
Then it exits 1 with "error: the vault folder vault is not an absolute path"
When the agent runs `tightship vault fly`
Then it exits 1, and stderr holds "error: unknown subcommand fly", a blank line, and the usage
And no subcommand of the binary exits with 2, a usage error included
When the agent runs `tightship vault init /tmp/<test>/vault` with a git on the PATH that exits 1
Then it exits 1 with "error: git init failed in /tmp/<test>/vault: <git's message>"
When bin/tightship runs on a machine whose platform folder is absent
Then it exits 1 with "error: tightship has no binary for <uname -s> <uname -m>"
```

### Scenario 31 — An unreadable vault, an unreadable note, an unwritable data folder (NFR-REL-01, exception)

```gherkin
Given a vault folder the binary cannot read
When `tightship hook start` runs
Then it exits 1 with "error: cannot read the vault folder <folder>: <the system's message>"
Given a vault with one note the binary cannot read
When `tightship hook start` runs
Then it exits 1 with "error: cannot read notes/<scope>/<file>: <the system's message>"
Given TIGHTSHIP_DATA is a file, not a folder
When `tightship hook prompt` runs
Then it exits 1 with "error: cannot write the checkpoint counter <path>: <the system's message>"
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md), marked M1, #5:

- **Building blocks:**
  - tightship binary: built, no longer "not built yet", with its subcommands `vault` and `hook` and the
    launcher.
  - Canonical content gains `vault-template/AGENTS.md`, and `options.toml` gains the `type` key and
    `vault_folder`.
  - Build compiles the binary for the machine that builds, with home paths remapped, and writes the memory
    hooks.
  - Claude Code plugin gains `bin/` and its memory hooks on `SessionStart` and `UserPromptSubmit`.
- **Runtime:** three new scenarios: "Session start: memory", "A prompt: the checkpoint", and "The agent
  reaches the vault", through `TIGHTSHIP_VAULT` in the session's environment.
- **Concepts:** a new page, memory, holds the vault folder, the scope, what the session start lists and its
  cap, and the checkpoint. Options names the `directory` type and `vault_folder`. Security adds the plugin's
  data folder and the session's environment file to what the plugin writes.
- **Deployment:** the adopter's machine runs the binary from this epic, built for the maintainer's machine
  only until epic #8.
- **Diagrams:** the containers diagram adds the vault template to canonical content, and `bin/` and the
  memory hooks to the Claude Code plugin. The tightship binary stays dashed until task 1 builds it.
- **ADR 0001** is amended: the plugin also writes the counter of the checkpoint in the harness's plugin data
  folder, and one line per session in the session's environment file.

## Data model impact

Applied to [docs/data-model/](../../data-model/README.md), marked M1, #5:

- A new page, vault, holds the physical model: the folder layout (`AGENTS.md`, `notes/<scope>/`), a note's
  file name and front matter with each field's type, obligatoriness, and domain, and what makes a file a note.
- Options gains `vault_folder`, a `directory` setting of the memory layer that Claude Code keeps like the
  others.
- Access patterns: the checkpoint reads and writes the counter of its session in the plugin's data folder,
  and arrives after the compaction instead of before.
- The conceptual model is unchanged.

## Out of Scope

- **Releasing and claiming a handoff**, the handoff command, and pulling and pushing the vault: epic #6. This
  epic only shows a handoff at session start.
- **FR-MEM-10.2**, the handoff command with memory off: epic #6 verifies it, since it builds that command.
  This epic covers FR-MEM-10.1.
- **The binary for every target**, and the release archive: epic #8. Until then, `dist/claude-code/` carries
  the binary of the machine that built it.
- **`index.md` and the per-scope hubs** the baseline generated: no requirement asks for them. A vault that
  already holds them keeps them, and the lint does not read them as notes.
- **Committing, publishing, or adding a remote to a new vault:** the adopter's, with their own git identity and
  host.
- **A search index:** every search reads every note, which takes well under the session start's budget on
  1,000 notes. A vault of tens of thousands of notes is not an M1 target.
- **Notes in other harnesses:** M2 to M4, each with its own hooks around the same binary.
- **Memory in a subagent:** the hooks reach the main session only, as the contract's do.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- |
| 1 | Ship the tightship binary with the vault folder and `vault init` | `Cargo.toml` (member), `tightship/Cargo.toml`, `tightship/src/main.rs`, `tightship/src/vault.rs` (folder, `path`), `tightship/src/init.rs`, `tightship/launcher.sh`, `tightship/tests/vault.rs`, `content/vault-template/AGENTS.md`, `content/options.toml`, `xtask/src/options.rs` (`type`), `xtask/src/content.rs` (`vault-template/`), `xtask/src/claude_code.rs` (`userConfig` types, `bin/`), `xtask/src/main.rs` (compile), `xtask/tests/build.rs` and `xtask/tests/common/mod.rs`, about 15 files | Scenarios 1 to 8 and 30 by tests | — |
| 2 | Place the vault's notes and checkpoints in the session | `tightship/src/note.rs` (read a note), `tightship/src/scope.rs`, `tightship/src/start.rs` (memory context, export), `tightship/src/checkpoint.rs`, `tightship/src/main.rs`, `tightship/tests/hooks.rs`, `xtask/src/claude_code.rs` (memory hooks), `xtask/tests/build.rs`, about 8 files | Scenarios 9 to 18 and 31 by tests; 23 to 28 in sessions of an isolated Claude Code configuration, 28 measured on a vault of 1,000 generated notes | 1 |
| 3 | Write, search, and lint notes | `tightship/src/new.rs`, `tightship/src/search.rs`, `tightship/src/lint.rs`, `tightship/src/note.rs` (write a note), `tightship/src/main.rs`, `tightship/tests/notes.rs`, about 6 files | Scenarios 19 to 22 and 29 by tests | 2 |
