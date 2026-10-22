# Curate the vault

> **Status:** closed
> **Profile:** API
> **Module:** `tightship/`, `content/vault-template/`, `xtask/`
> **Epic:** —
> **Requirements:** FR-MEM-01, FR-MEM-02, FR-MEM-12, FR-MEM-13
> **Sprint:** — (outside any sprint, by the maintainer's decision on 2026-10-22)

The vault stops growing on its own. The checkpoint comes only after a compaction, so the agent is no longer
pushed to write a note every 10 prompts. Each sync removes the notes the agent wrote on its own once 30 days
pass without a revision, and keeps the plugin's rules in a block of the vault's `AGENTS.md` that follows the
installed template. A session in a project lists the notes waiting to move into its repository, so they get
moved.

Terms from the SRS, as used here:

- **Vault**, **note**, **scope**, **pinned note**, **handoff**, **checkpoint**: as in the
  [epic #5 spec](../memory/keep-the-vault.md) and the [glossary](../../requirements/glossary.md).
- **Debt**: a note tagged `to-repo`, a fact that belongs in a repository and waits in the vault until a
  session in that repository moves it ([FR-MEM-01.4](../../requirements/functional/memory.md#fr-mem-01)).
- **Lifetime**: the days a note that can expire stays in the vault after its `updated` date
  ([FR-MEM-12](../../requirements/functional/memory.md#fr-mem-12)).
- **Rules block**: the delimited part of the vault's `AGENTS.md` that the plugin owns
  ([FR-MEM-13](../../requirements/functional/memory.md#fr-mem-13)).

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-22, in change request
[#118](https://github.com/ATNexusLab/tightship/issues/118):

1. **The work lands on the branch `fix/open-issues`**, with no sprint, by the maintainer's decision.
2. **One lifetime, 30 days, for every note that can expire.** A note can expire when its `type` is `gotcha` or
   `project`, its `source` is not `user`, and it is not pinned. `decision`, `preference`, and `handoff` never
   expire. The lifetime is a constant of the binary, not an option.
3. **The sync expires notes after the rebase and before the push**, in its own commit, and also in a vault
   with no remote. A note is past its lifetime when more than 30 days separate its `updated` date from the
   machine's local date. A note whose `updated` is missing or is not a date never expires; the lint reports it.
4. **A note that a live note links to expires all the same**, and each `[[link]]` to it becomes plain text,
   so the lint stays clean.
5. **A debt expires like any other note.** The exceptions of FR-MEM-12 are a closed list.
6. **Only the project's scope lists debts**: a note in `all` names no repository.
7. **The rules block is the whole template**, between two marker lines. `vault init` writes it, and the sync
   rewrites it when it is missing or differs. A vault with no markers gets the block at the top of its
   `AGENTS.md`, and the text that was there stays below it, unchanged.
8. **`tightship hook prompt` and every harness's prompt hook go.** Cursor keeps `beforeSubmitPrompt` for the
   contract's parts. Antigravity CLI is left with no checkpoint, as
   [NFR-FLEX-03.1](../../requirements/non-functional/flexibility.md#nfr-flex-03) already records.

## Acceptance Criteria

### Contract

| Command | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| A session start, `/clear`, or a compaction, with memory on | Adopter, through the harness | No: it writes the session's environment file | Prints the memory context, now with the project's debts |
| A compaction, with memory on | Adopter, through the harness | Yes | Prints the checkpoint, as before |
| Each prompt, with memory on | Adopter, through the harness | Yes | Runs no memory hook |
| `tightship vault init <folder>` | Agent, at the adopter's request | No | Creates a vault whose `AGENTS.md` is the rules block |
| `tightship vault sync` | Agent, through the command `sync` | No: it commits and pushes | Also removes the expired notes and updates the rules block |
| `tightship hook prompt` | — | — | Removed: a `USAGE` error, like any unknown subcommand |

### Request

No command gains an argument. The inputs are the vault's files.

#### A note, as the expiry reads it

| Field | Type | Required | Validation |
| --- | --- | --- | --- |
| `type` | string | Yes | Expires only as `gotcha` or `project` |
| `source` | string | Yes | Never expires as `user` |
| `pinned` | string | No | Never expires as `true` |
| `updated` | date | Yes | `YYYY-MM-DD`; anything else, or absent, never expires |
| `tags` | list of strings | No | Holds `to-repo` for a debt |

#### A debt's destination

The last line of the body that starts with `**Moves to:**`. Its rest, trimmed, is the destination. A debt
with no such line is listed with its `why`.

#### The vault's `AGENTS.md`

```text
<!-- tightship:rules:start -->
<content/vault-template/AGENTS.md, as the installed binary embeds it>
<!-- tightship:rules:end -->
<the adopter's own text, if any>
```

Each marker is a whole line. The file holds one start line, then one end line after it, or neither.

#### `content/vault-template/AGENTS.md`

The template gains, after the `to-repo` section:

```markdown
### Lifetime

A note of type `gotcha` or `project` that the user did not state and that is not pinned is removed by
`tightship vault sync` 30 days after its `updated` date. Rewriting it in place with a new `updated` keeps it
30 more days. A `[[link]]` to a removed note becomes plain text. `git log` keeps what was removed.
```

The `to-repo` section gains the line "Session start lists the project's debts, each with where it moves.",
and the template opens with:

```markdown
This block belongs to the plugin: `tightship vault sync` rewrites it to the installed template. Your own
rules go below it, outside the markers.
```

### Response

#### Session start

The memory context, as the [epic #5 spec](../memory/keep-the-vault.md) sets it, with two changes. The
header's line "Listed below, only the pinned notes." reads "Listed below, the project's debts and the pinned
notes.". A section follows the handoff and comes before the pinned notes, when the project has debts:

```text
## Debts — move into this repo

Move each into the file it names, in this work's commit, and delete the note.

- [[mem-0451-the-build-reads-identity-terms]] → AGENTS.md
- [[mem-0460-cursor-cli-skips-hooks]] — without it the next Cursor change breaks the CLI again
```

Debts are ordered by `id`, each listed once, and a pinned debt is listed only here. They share the
9,000-character budget with the pinned notes, ahead of them. Past the budget, the rest are cut and counted in
the line `[<n> debts cut at the 9000-character cap — tightship vault search to-repo]`.

#### `tightship vault sync`

After the renumbering and before the push, two steps, each printed only when it changes something:

```text
> Removing expired notes
expired notes/tightship/mem-0312-cargo-test-flakes-on-windows.md (updated 2026-09-01)
> Updating the vault's rules
```

The last line reads `synced: <n> pushed, <n> pulled, <n> expired`. In a vault with no remote, the two steps
run after the local commit, and the last line reads `kept local — the vault has no git remote; <n> expired`.

| Exit | When |
| --- | --- |
| `0` | Synced, or kept local |
| `1` | A row of **Errors** |

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Adopter | Their own vault, through the command `sync` | The sync never runs from a hook (epic #5, rule 15) |
| Agent | The adopter's shell | The binary reads and writes only the vault's folder |

---

## Business Rules

### 1. Authorization

The binary runs as the adopter, on the vault they set. Nothing checks a permission; memory off runs no hook
and no sync, as before ([FR-MEM-10](../../requirements/functional/memory.md#fr-mem-10)).

### 2. The checkpoint comes only after a compaction

No harness runs a memory hook on a prompt. The plugins of Claude Code, Cursor, and Antigravity CLI declare no
`tightship hook prompt`, and the binary has no such subcommand. A compaction prints the checkpoint as before
([FR-MEM-02.2](../../requirements/functional/memory.md#fr-mem-02)).

### 3. The session start lists the project's debts

A note is a debt when its scope is the project's and its `tags` hold `to-repo`. The section, its order, and its
cut follow **Response**. A project with no debts prints no section.

### 4. A note expires after 30 days without a revision

At each sync, a note is removed when it can expire (decision 2) and more than 30 days separate its `updated`
from the local date. A note whose `updated` was rewritten keeps its place for 30 more days
([FR-MEM-12.3](../../requirements/functional/memory.md#fr-mem-12)). The removal is a commit
`vault: expire notes`, so the vault's history keeps the note
([FR-MEM-12.1](../../requirements/functional/memory.md#fr-mem-12)).

### 5. A link to an expired note becomes plain text

In every note that stays, `[[<stem>]]` and `[[<stem>#<heading>]]` become the expired note's title, and
`[[<stem>|<text>]]` becomes `<text>`. A note changed this way keeps its `updated`: the link is not a revision
of the fact. The change is part of the commit `vault: expire notes`.

### 6. The rules block follows the installed template

At each sync, the binary compares the text between the markers with the template it embeds:

- No `AGENTS.md`: the file becomes the block.
- No markers: the block goes at the top, then a blank line, then the file's text as it was.
- A block that differs: the text between the markers is replaced; the text outside stays byte for byte
  ([FR-MEM-13.2](../../requirements/functional/memory.md#fr-mem-13)).
- A block that matches: nothing changes and nothing is printed.

A change is a commit `vault: update the rules`. `vault init` writes `AGENTS.md` as the block alone.

### 7. A sync that stops leaves the vault as git left it

A failure in either step stops the sync before the push, with exit 1 and its message. The commits already made
stay local, and the next sync pushes them.

### 8. Persistence and audit

- **Files changed:** the expired notes are removed, the notes that linked to them rewritten, the vault's
  `AGENTS.md` written.
- **Audit:** the vault's git history; each step is its own commit.
- **Events and integrations:** the push to the vault's remote, as before.

---

## Errors

Every message goes to stderr, prefixed by `error:` and a space, as in the [epic #5 spec](../memory/keep-the-vault.md).

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `RULES_BLOCK_BROKEN` | `1` | Rule 6: a start line with no end line after it, an end line with no start line before it, or more than one of either | "`` the vault's AGENTS.md must hold one <!-- tightship:rules:start --> line followed by one <!-- tightship:rules:end --> line; fix it by hand, then run `tightship vault sync` again ``" |
| `NOTE_UNREADABLE` | `1` | Rules 4 to 6: a note or `AGENTS.md` cannot be read | "`` cannot read <path relative to the vault>: <the system's message> ``" |
| `NOTE_UNWRITABLE` | `1` | Rules 5 and 6: a note or `AGENTS.md` cannot be written | "`` cannot write <path relative to the vault>: <the system's message> ``" |
| `HANDOFF_UNREMOVABLE` | `1` | Rule 4: an expired note cannot be removed | "`` cannot remove <path relative to the vault>: <the system's message> ``" |
| `SYNC_GIT_FAILED` | `1` | Rules 4 and 6: a commit fails | "`` git commit failed in <vault>: <git's message> ``" |
| `USAGE` | `1` | `tightship hook prompt` | "`unknown subcommand prompt`", then a blank line and the usage |

`HANDOFF_UNREMOVABLE` is renamed `NOTE_UNREMOVABLE` in the code, since it now removes notes of any type; its
message does not change.

## Side Effects

- **Persistence:** each note is removed or rewritten whole; `AGENTS.md` is written whole by one write call.
- **Concurrency:** two machines that expire the same note both commit its removal; the rebase of the one that
  pushes last finds the removal already there. Two machines that update the rules block to the same template
  write the same text, which the rebase accepts. Two installed versions with different templates take turns
  rewriting the block until both machines run the same version.
- **Transaction:** none across files. A step that fails halfway leaves its files changed and not committed;
  the next sync's first commit records them.

---

## Acceptance Scenarios (Gherkin)

Scenarios 1 to 13 run as tests over vaults created in a temporary folder, a sync's remote being a bare
repository in the same folder. Scenario 14 runs in a session of an isolated configuration of each harness.

### Scenario 1 — No harness counts prompts (FR-MEM-02, happy path)

```gherkin
Given the content with the memory option declared
When the maintainer runs `cargo xtask build`
Then no hook file of `dist/claude-code/`, `dist/cursor/`, or `dist/antigravity-cli/` runs `tightship hook prompt`
And `dist/cursor/hooks/hooks.json` keeps `beforeSubmitPrompt` with the contract's parts
And the Claude Code plugin still runs `tightship hook checkpoint` on `SessionStart` with the matcher `compact`
```

### Scenario 2 — The prompt subcommand is gone (exception)

```gherkin
Given the binary
When a harness left from an older install runs `tightship hook prompt`
Then it exits 1 and stderr starts with "error: unknown subcommand prompt"
```

### Scenario 3 — The session start lists the project's debts (FR-MEM-01.4, happy path)

```gherkin
Given memory is on, the project's scope is `tightship`
And the vault holds notes/tightship/mem-0002-a.md tagged `to-repo` with a last line "**Moves to:** AGENTS.md"
And notes/tightship/mem-0003-b.md tagged `to-repo`, pinned, with no **Moves to:** line
And notes/all/mem-0004-c.md tagged `to-repo`
When a session starts in the project
Then the context holds "## Debts — move into this repo"
And the line "- [[mem-0002-a]] → AGENTS.md", then the line "- [[mem-0003-b]] — <its why>"
And mem-0003-b is not listed among the pinned notes
And mem-0004-c is not listed
```

### Scenario 4 — A project with no debts (alternative)

```gherkin
Given memory is on and no note of the project's scope is tagged `to-repo`
When a session starts
Then the context holds no "## Debts" section
```

### Scenario 5 — Debts past the cap (alternative)

```gherkin
Given the project's scope holds 400 debts with a `why` of 100 characters each
When a session starts
Then the memory context is at most 9,000 characters
And it ends with "[<n> debts cut at the 9000-character cap — tightship vault search to-repo]"
```

### Scenario 6 — A note expires (FR-MEM-12.1, happy path)

```gherkin
Given a vault with a remote, and the local date 2026-10-22
And notes/all/mem-0002-a.md of type `gotcha`, source `session`, updated 2026-09-21
When the agent runs `tightship vault sync`
Then it prints "expired notes/all/mem-0002-a.md (updated 2026-09-21)"
And it ends with "synced: 1 pushed, 0 pulled, 1 expired"
And neither the vault nor the remote holds mem-0002-a.md
And the commit "vault: expire notes" in the remote's history removes it
```

### Scenario 7 — The notes that never expire (FR-MEM-12.2, alternative)

```gherkin
Given the local date 2026-10-22, and six notes updated 2025-01-01:
  a `gotcha` with source `user`, a pinned `project`, a `decision`, a `preference`, a `handoff`,
  and a `gotcha` with `updated: soon`
When the agent runs `tightship vault sync`
Then the vault still holds all six
And it ends with "0 expired"
```

### Scenario 8 — A note within its lifetime (alternative)

```gherkin
Given the local date 2026-10-22
And a `gotcha` with source `session` updated 2026-09-22, which is 30 days before
When the agent runs `tightship vault sync`
Then the vault still holds it
```

### Scenario 9 — A note rewritten in place (FR-MEM-12.3, alternative)

```gherkin
Given a `project` note with source `repo`, created 2026-08-01 and updated 2026-10-10
And the local date 2026-10-22, past the date its created version would have expired
When the agent runs `tightship vault sync`
Then the vault still holds it
```

### Scenario 10 — Links to an expired note (alternative)

```gherkin
Given an expired note notes/all/mem-0002-a.md titled "Cargo test flakes on Windows"
And a live note whose body holds "[[mem-0002-a]]", "[[mem-0002-a#fix]]", and "[[mem-0002-a|the flake]]"
When the agent runs `tightship vault sync`
Then the live note's body holds "Cargo test flakes on Windows" twice and "the flake" once, and no "[[mem-0002-a"
And its `updated` is unchanged
And `tightship vault lint` reports no link that points at no note
```

### Scenario 11 — The rules reach an older vault (FR-MEM-13.1, FR-MEM-13.2, happy path)

```gherkin
Given a vault whose AGENTS.md holds "# My rules" and no markers
When the agent runs `tightship vault sync`
Then it prints "> Updating the vault's rules"
And AGENTS.md opens with the start marker, the template, and the end marker, then a blank line and "# My rules"
And the commit "vault: update the rules" records it
When the agent runs `tightship vault sync` again
Then it does not print "> Updating the vault's rules"
And AGENTS.md is unchanged
```

### Scenario 12 — An older block, and a vault with no AGENTS.md (alternative)

```gherkin
Given a vault whose block holds an older template, with "# My rules" above the start marker and below the end marker
When the agent runs `tightship vault sync`
Then the block holds the installed template
And both "# My rules" lines are where they were, byte for byte
Given a vault with no AGENTS.md
When the agent runs `tightship vault sync`
Then AGENTS.md is the block alone
And `tightship vault init <folder>` writes the same AGENTS.md in a new vault
```

### Scenario 13 — A broken block, and a vault with no remote (exception, alternative)

```gherkin
Given a vault whose AGENTS.md holds the start marker and no end marker
When the agent runs `tightship vault sync`
Then it exits 1 with the message of RULES_BLOCK_BROKEN
And the remote is unchanged
Given a vault with no remote, an expired note, and AGENTS.md with no markers
When the agent runs `tightship vault sync`
Then it ends with "kept local — the vault has no git remote; 1 expired"
And the commits "vault: expire notes" and "vault: update the rules" are in its history
```

### Scenario 14 — Each harness, in a session (FR-MEM-02.2, FR-MEM-01.4)

```gherkin
Given each harness, Claude Code, Cursor, and Antigravity CLI, with the built plugin and memory on
And a project with one debt
When a session starts and the adopter sends 10 prompts
Then the agent's context holds the debt and no checkpoint
When the session's context is compacted, in Claude Code and Cursor
Then the context that follows holds the checkpoint
```

---

## Architecture impact

- [Runtime](../../architecture/README.md#runtime): **A prompt: the checkpoint** is removed. **Session start:
  memory** also lists the project's debts. **The adopter syncs the vault** also expires notes and updates the
  rules block.
- [Memory concept](../../architecture/concepts/memory.md): **What the session start lists** adds the debts;
  **The checkpoint** comes only after a compaction, with no prompt counter; **The sync** adds the expiry and the
  rules block.
- [Security concept](../../architecture/concepts/security.md): the plugin's data folder no longer holds the
  checkpoint's prompt counter.
- Building blocks: [Claude Code plugin](../../architecture/building-blocks/claude-code-plugin.md),
  [Cursor plugin](../../architecture/building-blocks/cursor-plugin.md), and
  [Antigravity CLI plugin](../../architecture/building-blocks/antigravity-cli-plugin.md) lose the prompt hook;
  the [tightship binary](../../architecture/building-blocks/tightship-binary.md) loses `hook prompt`; the
  [building blocks index](../../architecture/building-blocks/README.md) no longer runs a hook on each prompt.

## Data model impact

- [Vault](../../data-model/vault.md): the layout's `AGENTS.md` is the rules block and the adopter's text; a
  new section, **A note's lifetime**; **Outside the vault** loses the prompt count of the checkpoint.
- [Access patterns](../../data-model/access-patterns.md): **Session start** also reads the project's debts;
  **Checkpoint** runs only after each compaction and reads nothing; **Sync** also removes expired notes,
  rewrites the links to them, and writes `AGENTS.md`.

## Out of Scope

- **Triaging the maintainer's current vault**: a pass by hand over the adopter's own notes, after this spec's
  work ships. 306 of its 323 notes that can expire are under 30 days old, so the expiry alone clears them only
  from late November 2026.
- **A prune command, or a lifetime the adopter sets**: no requirement asks for either.
- **The `checkpoints/` folder older installs left in the plugin's data folder**: it holds one small file per
  session, read by nothing, and uninstalling the plugin removes it.
- **Moving the old text of an existing vault's `AGENTS.md`**: it stays below the block, and the adopter deletes
  what the block replaces.
- **Expiry from anything but the sync**: a hook never writes the vault's history or uses the network.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- |
| 1 | Place the checkpoint only after a compaction | `tightship/src/hook/prompt.rs` (deleted), `tightship/src/hook.rs`, `tightship/src/main.rs`, `tightship/tests/hook.rs` (2 prompt tests removed), `xtask/src/build/claude_code.rs`, `xtask/src/build/cursor.rs`, `xtask/src/build/antigravity_cli.rs`, `xtask/tests/contract.rs`, `xtask/tests/cursor_contract.rs`, `xtask/tests/antigravity_cli_contract.rs`, `xtask/tests/install.rs`, `docs/architecture/` (runtime, memory and security concepts, 4 building blocks), `docs/data-model/vault.md`, `docs/data-model/access-patterns.md`, about 19 files | Scenarios 1 and 2 by tests; 14's checkpoint part in sessions | — |
| 2 | List the project's debts at session start | `tightship/src/vault/context.rs`, `tightship/src/vault/note.rs` (tags, destination), `tightship/tests/hook.rs`, `docs/architecture/` (runtime, memory concept), `docs/data-model/access-patterns.md`, about 6 files | Scenarios 3 to 5 by tests; 14's debt part in sessions | 1 |
| 3 | Expire notes at each sync | `tightship/src/vault/expire.rs` (new), `tightship/src/vault/sync.rs`, `tightship/src/vault.rs`, `tightship/src/vault/note.rs` (title), `tightship/src/failure.rs` (`NoteUnremovable`), `tightship/src/vault/handoff.rs` (the rename), `tightship/tests/sync.rs`, `content/vault-template/AGENTS.md` (Lifetime), `docs/architecture/concepts/memory.md`, `docs/data-model/vault.md`, `docs/data-model/access-patterns.md`, about 11 files | Scenarios 6 to 10, and 13's expiry part, by tests | 1 |
| 4 | Keep the vault's rules block at each sync | `tightship/src/vault/rules.rs` (new), `tightship/src/vault/sync.rs`, `tightship/src/vault/init.rs`, `tightship/src/vault.rs`, `tightship/src/failure.rs` (`RulesBlockBroken`), `tightship/tests/sync.rs`, `tightship/tests/vault.rs`, `content/vault-template/AGENTS.md` (the opening lines, the debts line), `docs/architecture/concepts/memory.md`, `docs/data-model/vault.md`, about 10 files | Scenarios 11 and 12, and 13's block part, by tests | 3 |
