# #6 — Hand a session off through the vault

> **Status:** published
> **Profile:** API
> **Module:** `tightship/`, `content/`, `harnesses/claude-code/`, `xtask/`
> **Epic:** #6
> **Requirements:** FR-MEM-06, FR-MEM-07, FR-MEM-10 (FR-MEM-10.2), FR-CTR-03 (FR-CTR-03.2)
> **Sprint:** 4

An adopter who stops midway runs the command `handoff release`. The agent writes the session's state, and
`tightship vault handoff release` puts it in the vault as the project's only handoff, which the sync then
pushes. On the next machine, `handoff claim` syncs, prints the handoff, removes it from the vault, and has the
agent compare it with the repository before it shows the next step. The command knows memory is off through a
new token, `{{option:<name>}}`. The closeout command gets back the memory item it had in the baseline.

Terms from the SRS, as used here:

- **Handoff**: the one vault note per project that carries a session's state to the next session, on any
  machine or harness ([glossary](../../requirements/glossary.md)). Its type is `handoff`, and the session start
  of epic #5 already shows it ([FR-MEM-01.2](../../requirements/functional/memory.md#fr-mem-01)).
- **Release** a handoff: write it to the vault, replacing the previous one
  ([FR-MEM-06](../../requirements/functional/memory.md#fr-mem-06)).
- **Claim** a handoff: move it from the vault into the session
  ([FR-MEM-07](../../requirements/functional/memory.md#fr-mem-07)).
- **Vault**, **note**, **scope**, and the project's scope: as the
  [epic #5 spec](../memory/keep-the-vault.md) defines them. The scope is the name of the project's repository
  folder, matched against the vault's scope folders (its rule 3).
- **Sync**: `tightship vault sync` and the command `sync` of epic #5
  ([FR-MEM-11](../../requirements/functional/memory.md#fr-mem-11)).
- **Option**: a part of the plugin the adopter turns on or off. It is off only when its value is exactly
  `false` (rule 4 of the [epic #3 spec](../options/choose-the-options.md)).
- **Baseline**: commit `1f41699` of the maintainer's private configuration. Its handoff is the command
  `commands/handoff.md` and the subcommand `vault.py handoff release|claim`.

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-15:

1. **The branch `feat/6-handoff` is cut from `develop`**, after epic #5's PR merged.
2. **The binary writes and removes handoffs, and never runs git.** `tightship vault handoff release` and
   `tightship vault handoff claim` work on the files only. The baseline pulled, committed, and pushed inside
   `vault.py handoff`. Here the command runs `tightship vault sync` around them, so the vault has one git path,
   and a conflict is resolved the way the command `sync` resolves one.
3. **A claim syncs first, and stops when that sync fails.** Without the sync, a handoff released on another
   machine is not there yet. A claim made offline could show a local handoff that is stale.
4. **The command reads the memory option through a token.** No hook runs with memory off, so the binary cannot
   tell memory off from a vault folder never set. The command's body holds `{{option:memory}}`, which Claude
   Code renders as `${user_config.memory}` and fills in when the command runs. A value of exactly `false`
   means off. An option never set reaches the body as the literal text, which is on, as rule 4 of epic #3
   requires. This amends that rule, which said no piece reads an option, and ADR 0001's token kinds.
5. **The binary checks that the body has its five parts.** A release whose body lacks one fails, so
   FR-MEM-06.1 holds without trusting the agent.
6. **A release removes every handoff of the scope, and a claim prints and removes every one.** Two machines
   that release before syncing leave two handoffs after the next sync. The baseline printed only the latest
   and removed both, so one was lost without a trace.
7. **The agent compares the handoff with the repository** (FR-MEM-07.2). It reads the commits that **Repo
   state** names and checks each one with git. The project's repository is only read: no commit, push, stash,
   or checkout.
8. **The closeout command regains its memory item**, the baseline's §5, worded for `tightship vault`. Its
   mention in the contract goes into the gated Memory section. The dev loop's Closeout step keeps its wording,
   because rule 4 of the [epic #4 spec](../contract/load-the-contract.md) keeps every memory rule inside the
   gated section (FR-CTR-03.1). The session start gets back the baseline's line on claiming the handoff.
9. Two tasks: the binary, then the command, the token, and the closeout.

## Acceptance Criteria

### Contract

| Command | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| The command `handoff release` | Adopter | No | The agent writes the body, releases it, and syncs the vault |
| The command `handoff claim` | Adopter | No | The agent syncs, claims, syncs again, compares with the repository, and shows the next step |
| `tightship vault handoff release --title <title>` with the body on stdin | Agent, through the command | No | Writes the scope's handoff and removes the scope's earlier ones |
| `tightship vault handoff claim` | Agent, through the command | No | Prints the scope's handoffs and removes them |
| A session start with a handoff in the scope | Adopter, through Claude Code | Yes | The handoff's section also says how to claim it |
| `cargo xtask build` | Maintainer | Yes | Also renders `{{option:<name>}}`, and writes the command `handoff` |

### Request

#### `tightship vault handoff release`

| Argument | Type | Required | Validation |
| --- | --- | --- | --- |
| `--title` | string | Yes | 1 to 120 characters, once each run of whitespace, line breaks included, is one space; else `NOTE_TITLE_INVALID` |
| Body | stdin | Yes | Not empty after trimming, never read from a terminal, else `NOTE_BODY_MISSING`. It holds the five labels of the table below, each at the start of a line, else `HANDOFF_BODY_INCOMPLETE` |

The five parts of the body, in this order, written in English like every note:

| Label | Holds |
| --- | --- |
| `**Goal:**` | What the work is for, with the issue or plan it follows |
| `**Done:**` | What is finished and verified |
| `**Next step:**` | The one action the next session starts with |
| `**Open decisions:**` | What waits for the adopter, or `none` |
| `**Repo state:**` | Branch `<name>` · clean, or the dirty files · each unpushed commit as `<short hash> <subject>` |

```bash
tightship vault handoff release --title "Handoff spec drafted, waiting for the values table" <<'EOF'
**Goal:** close the spec of epic #6, issue #6
**Done:** the assumptions are confirmed and the draft is written
**Next step:** confirm the table of proposed values, then run the closed-spec gate
**Open decisions:** none
**Repo state:** branch `feat/6-handoff` · dirty: docs/specs/handoff/hand-off-the-session.md · no unpushed commit
EOF
```

#### `tightship vault handoff claim`

No argument.

Both read the vault from `TIGHTSHIP_VAULT` and the project's scope from the working directory, as rules 2
and 3 of epic #5 say.

#### `content/commands/handoff.md`

New. A command with the name `handoff`, the description "Release this session's work as a handoff, or claim
the one waiting — to switch machine, OS or harness mid-task", and the argument hint `release | claim`. Its body:

```markdown
Run the handoff step for this project: {{capability:arguments}}

A handoff is the one vault note that holds a session's state: one per project scope, replaced on release
and removed on claim. `tightship vault handoff` writes and removes it, `tightship vault sync` carries it
between machines, and this command writes its text and reads the repository.

Memory is `{{option:memory}}` for this plugin. When that reads exactly `false`, tell the user "Memory is off,
so this project has no vault to hand off through. Turn the memory option on and start a new session to use
it." and stop. When the argument is neither `release` nor `claim`, tell the user "Run the handoff command with
`release` or `claim`." and stop.

The project's repository is only read, never changed: no commit, push, stash, or checkout. Its approval
stays with the user.

## release

1. Read the repository: `git branch --show-current`, `git status -sb`, and the commits not on the remote,
   `git log --oneline @{upstream}..`, or every commit of the branch when it tracks no remote branch.
2. Write the body from the session, in English like every vault note:

   **Goal:** <what the work is for, with the issue or plan it follows>
   **Done:** <what is finished and verified>
   **Next step:** <the one action the next session starts with>
   **Open decisions:** <what waits for the user, or none>
   **Repo state:** branch `<name>` · <clean, or the dirty files> · <each unpushed commit as its short hash and subject>

   Anything only on this machine — a dirty file, an unpushed commit, a running server — is named in
   **Repo state** with the warning that the other machine does not have it.
3. From the project's directory, run `tightship vault handoff release --title "<what the work is, and where
   it stopped>"` with the body on stdin. When it fails, show its message and stop.
4. Run `tightship vault sync`. When it fails, follow `{{command:sync}}` from its step 2.
5. Tell the user, in one line: "Handoff released: <the path release printed> — <the sync's last line>."

## claim

1. Run `tightship vault sync`. When it fails, follow `{{command:sync}}` from its step 2; when that ends
   without a sync, stop and claim nothing, since the remote may hold a newer handoff.
2. From the project's directory, run `tightship vault handoff claim`. When it prints
   `no handoff for <scope>`, tell the user "No handoff waits for <scope>; nothing changed." and stop.
3. Run `tightship vault sync` again, as in step 1, so the other machines see the handoff is gone.
4. Compare each handoff with the repository: run `git fetch`, then check every commit of **Repo state** with
   `git cat-file -e <hash>^{commit}`, and the branch and the working tree against it. Name each difference:
   - "Commit <hash> is not in this repository; it stayed on the machine that released the handoff."
   - "The handoff was released on branch <name>, and this checkout is on <name>."
   - "The handoff names uncommitted changes to <files>; they stayed on the other machine."
5. Show the next step and the open decisions of each handoff, and wait for the user before acting on them.
```

`<scope>` is the scope the binary names. The body names the binary's commands and no harness.

#### `content/commands/closeout.md`

The description becomes "Check what the delivery gate does not — surfaces, docs, traceability, security debt,
memory — before the commit". A section is added after §4:

```markdown
## 5. Memory

Only with a vault: when `tightship vault path` fails, this item is `n/a — no vault`. Facts are written to the
vault as they appear, not swept up here. What this step checks is that nothing durable from this session
went unwritten:

- Walk the session for the four things that qualify — a rule the user stated · a decision plus its why
  that outlives the task · a trap that cost real time and will recur · a project fact that lives in no
  repo. Anything found gets written now with `tightship vault new`, per the gate in the vault's `AGENTS.md`.
- Anything durable that landed in the harness's own per-project memory is a write that went to the wrong
  place: move it into the vault and delete it there.
- Derivable from the repo, its history, or its docs → it does not go in.
```

The Output block gains the line `- Memory:        [promoted <note> / nothing durable / n/a — no vault]`.

#### `content/contract.md`

The Memory section gains, after the bullet on the harness's per-project `memory/` folder:

```markdown
- **`{{command:closeout}}` also covers memory:** it checks that nothing durable from the session went
  unwritten.
```

#### The token `{{option:<name>}}`

| Part | Validation |
| --- | --- |
| `<name>` | 1 to 64 lowercase ASCII letters, digits, and `_`, starting with a letter, the domain of an option's `name` in epic #3; else `TOKEN_MALFORMED` |
| The option | A `boolean` entry of `content/options.toml`; else `TOKEN_MISSING_OPTION` |
| Where | The body of a skill, a command, or an agent; in `content/contract.md` or `content/vault-template/AGENTS.md`, `TOKEN_OPTION_OUTSIDE_PIECE` |

`harnesses/claude-code/harness.toml` gains, in `[references]`, the key `option = "${user_config.{name}}"`, which
renders the token, and, in `[neutrality] paths`, the term `${user_config`, so a literal in `content/` fails
with `HARNESS_TERM`.

### Response

#### What `tightship vault handoff release` writes and prints

It writes `<vault>/notes/<scope>/<id>-<slug>.md`, `<id>` and `<slug>` made as `tightship vault new` makes them:

```markdown
---
id: mem-0641
type: handoff
project: tightship
why: without it the next session, on another machine or harness, starts blind on work already in flight
source: session
created: 2026-10-15
updated: 2026-10-15
---

# Handoff spec drafted, waiting for the values table

**Goal:** close the spec of epic #6, issue #6
…
```

Then it removes every other note of type `handoff` in the scope, and prints, paths relative to the vault:

```text
notes/tightship/mem-0641-handoff-spec-drafted-waiting-for-the-values-table.md
replaced notes/tightship/mem-0630-contract-migrated-review-pending.md
```

One `replaced` line per note removed, in path order, and none when the scope had no handoff. Exit 0.

#### What `tightship vault handoff claim` prints

Each handoff of the scope, the latest `updated` first, then the highest `id`, separated by one blank line:

```markdown
# Handoff — tightship · mem-0641 · 2026-10-15

# Handoff spec drafted, waiting for the values table

**Goal:** close the spec of epic #6, issue #6
…
```

`<age>` is the one of epic #5: the `created` date, or `<created> → <updated>` when they differ. Then each is
removed, and it exits 0. With no handoff in the scope, it prints `no handoff for <scope>`, changes nothing,
and exits 0.

#### The session start

The handoff's section of epic #5's memory context gains one line between its heading and the body:

```markdown
## Handoff — <scope> · <id> · <age>

Claim it with the handoff command once you pick the work up.

<the handoff's body>
```

The line counts toward the 9,000 characters, and appears also when the body is replaced by the line on its
size.

#### The usage

The binary's usage gains two lines after `tightship vault sync`:

```text
       tightship vault handoff release --title <title> < body
       tightship vault handoff claim
```

#### `dist/claude-code/`

`skills/handoff/SKILL.md` holds the command with `{{option:memory}}` rendered as `${user_config.memory}` and
`{{command:sync}}` as `/tightship:sync`. With the baseline's contract and every option declared, the build's
success output reads:

```text
Built dist/claude-code/: 24 pieces (13 skills, 10 commands, 1 agent), 37 files.
Try it: claude --plugin-dir dist/claude-code
```

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | Edits the command, the token, and the binary, and runs the build |
| Adopter | None from this project | Owns the vault and the project's repository, and runs the command |
| Agent | The Bash tool of the adopter's session | Runs `tightship vault handoff`, `tightship vault sync`, and git's read commands; the binary checks no permission of its own |

---

## Business Rules

### 1. Authorization

N/A. The binary and the command run in the adopter's own session, on the adopter's vault and repository, with
no service of this project behind them.

### 2. Memory off leaves everything as it was

- **When:** the command `handoff`, before any step, with the memory option exactly `false`.
- The agent tells the adopter, and runs nothing:
  > "Memory is off, so this project has no vault to hand off through. Turn the memory option on and start a
  > new session to use it."
- The option never set renders as the literal `${user_config.memory}`, which is not `false`: memory is on
  (FR-MEM-10.2).
- With memory on and no usable vault, `tightship vault handoff` fails with the errors of epic #5's rule 2, and
  the command shows that message and stops.

### 3. The command needs `release` or `claim`

- **When:** the command `handoff` with any other argument, or none.
- The agent tells the adopter, and runs nothing:
  > "Run the handoff command with `release` or `claim`."

### 4. A release leaves exactly one handoff in the scope

- **When:** `tightship vault handoff release`.
- The title, then the body, are checked as the Request says, and the first that fails is reported. Nothing is
  written or removed then.
- The new note is written first. Only then is each other note of type `handoff` in the scope removed. A note
  that cannot be removed fails with `HANDOFF_UNREMOVABLE`, and the new note stays (FR-MEM-06.1).
- Only the project's scope is touched, never `all` or another scope.

### 5. A release never changes the project's repository

- **When:** the command `handoff release`, and `tightship vault handoff release`.
- The binary runs no git command. The command runs only `git branch --show-current`, `git status -sb`, and
  `git log` in the project's repository (FR-MEM-06.2).

### 6. The handoff travels by the sync

- **When:** the command, after a release, and before and after a claim.
- `tightship vault sync` records the vault's changes, takes in the remote's, and pushes, as epic #5's rule 15
  says. A vault with no remote keeps the handoff local, and the sync's last line, `kept local — the vault has
  no git remote`, says so.
- A sync that fails after a release leaves the handoff written and not pushed. The agent follows the command
  `sync` from its step 2.
- A sync that fails before a claim ends the claim, which removes nothing.

### 7. A claim moves every handoff of the scope into the session

- **When:** `tightship vault handoff claim`.
- Every note of type `handoff` in the scope is printed as the Response says, then removed (FR-MEM-07.1). A note
  that cannot be removed fails with `HANDOFF_UNREMOVABLE`, after the print.
- With none, it prints `no handoff for <scope>` and changes nothing; the agent says
  "No handoff waits for `<scope>`; nothing changed." (FR-MEM-07.3).

### 8. A claim names what the repository lacks

- **When:** the command `handoff claim`, after the second sync.
- The agent runs `git fetch` in the project's repository, then checks each commit of **Repo state** with
  `git cat-file -e <hash>^{commit}`, the current branch, and the working tree, and names each difference with
  the Request's sentences (FR-MEM-07.2).
- It shows the next step and the open decisions, and does not act on them before the adopter answers.

### 9. The option token renders the harness's value

- **When:** `cargo xtask build`, while it renders the pieces.
- `{{option:<name>}}` is checked as the Request says, and renders as `[references] option` with `{name}`
  replaced.
- The kinds of token become `skill`, `command`, `agent`, `capability`, and `option`; `TOKEN_UNKNOWN_KIND` lists
  them in this order.

### 10. The closeout checks memory again

- **When:** a session with the contract and memory on.
- The Memory part of the contract holds the closeout line of the Request, and no part outside it names
  memory's commands (FR-CTR-03.1, FR-CTR-03.2).
- The command `closeout` runs its §5 and reports the line `Memory:`. With no vault it records
  `n/a — no vault`.

### 11. Persistence and audit

- **Files changed:** in the vault, the handoff written and the handoffs removed, then the sync's commits. The
  project's repository: nothing; `git fetch` on a claim updates only its remote-tracking branches. The build
  writes `skills/handoff/` into `dist/claude-code/`.
- **Audit:** N/A. The vault is a git repository: the sync's commits record every release and claim.
- **Events/integrations fired:** only the sync, to the vault's own remote, and `git fetch` on a claim, to the
  project's remote, both through the adopter's git.

---

## Errors

Every message goes to stderr, prefixed by `error:` and a space. Exit 1, never 2, as epic #5's rule 11 says.

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `NOTE_TITLE_INVALID` | `1` | Rule 4, epic #5's | "`` --title must be 1 to 120 characters ``" |
| `NOTE_BODY_MISSING` | `1` | Rule 4, epic #5's | "`` the note's body must arrive on stdin ``" |
| `HANDOFF_BODY_INCOMPLETE` | `1` | Rule 4 | "`` the handoff's body has no <label>, <label>; each of **Goal:**, **Done:**, **Next step:**, **Open decisions:**, and **Repo state:** starts a line of it ``" |
| `NOTE_EXISTS` | `1` | Rule 4, epic #5's | "`` <path relative to the vault> already exists ``" |
| `NOTE_UNWRITABLE` | `1` | Rule 4, epic #5's | "`` cannot write <path relative to the vault>: <the system's message> ``" |
| `NOTE_UNREADABLE` | `1` | Rules 4 and 7, epic #5's | "`` cannot read <path relative to the vault>: <the system's message> ``" |
| `HANDOFF_UNREMOVABLE` | `1` | Rules 4 and 7 | "`` cannot remove <path relative to the vault>: <the system's message> ``" |
| `VAULT_NOT_SET`, `VAULT_NOT_ABSOLUTE`, `VAULT_MISSING`, `VAULT_UNREADABLE` | `1` | Rule 2, epic #5's | Epic #5's messages |
| `USAGE` | `1` | An unknown `handoff` subcommand, an unknown argument, or `--title` missing on release | Epic #5's messages, then a blank line and the usage |
| `TOKEN_MALFORMED` | `1` | Rule 9, epic #1's | "`<path>:<line>: "<text>" is not a token; write {{<kind>:<name>}}`" |
| `TOKEN_MISSING_OPTION` | `1` | Rule 9 | "`<path>:<line>: {{option:<name>}} names no boolean option in content/options.toml`" |
| `TOKEN_OPTION_OUTSIDE_PIECE` | `1` | Rule 9 | "`<path>:<line>: {{option:<name>}} works only in a skill, a command, or an agent`" |
| `HARNESS_TERM` | `1` | Rule 9, epic #1's | "`<path>:<line>: "${user_config" belongs to claude-code; use a {{capability:…}} token defined in harnesses/claude-code/harness.toml`" |

`<label>` is each missing label, in the order of the Request's table.

## Side Effects

- **Persistence:** a handoff is written whole by one write call, before any earlier one is removed.
- **Concurrency:** two machines that release before syncing each remove only their own scope's handoffs; the
  next sync brings both in, renumbered when their ids collide, and the next claim prints and removes both. Two
  releases at once on one machine are not a case: one session runs one command at a time.
- **Transaction:** none across files. A failed removal leaves the new handoff and the ones not yet removed.

---

## Acceptance Scenarios (Gherkin)

Scenarios 1 to 13 run as tests over vaults and repositories created in a temporary folder. Scenarios 14 to 20
run in sessions of an isolated Claude Code configuration, with a vault cloned twice from a bare remote,
A and B.

### Scenario 1 — Release replaces the scope's handoff (FR-MEM-06.1, happy path)

```gherkin
Given a vault whose highest id is mem-0640, with the handoff notes/tightship/mem-0630-old.md
And a handoff notes/other/mem-0620-x.md in the scope "other"
When the agent runs the Request's `tightship vault handoff release` in the project "tightship"
Then it exits 0 and prints the new path, then "replaced notes/tightship/mem-0630-old.md"
And notes/tightship/ holds exactly one note of type handoff, mem-0641, with the Response's front matter and the
  five labels
And notes/other/mem-0620-x.md is unchanged
And `tightship vault lint` exits 0
```

### Scenario 2 — Release leaves the project's repository as it was (FR-MEM-06.2)

```gherkin
Given the project is a git repository with a dirty file, a stash entry, and a commit its remote lacks
When the agent runs `tightship vault handoff release` in it
Then `git rev-parse HEAD`, `git for-each-ref`, `git stash list`, and `git status --porcelain` print what they
  printed before
```

### Scenario 3 — A body without its five parts (exception)

```gherkin
Given a vault with the handoff notes/tightship/mem-0630-old.md
When the agent releases a body that lacks "**Done:**" and "**Repo state:**"
Then it exits 1 with "error: the handoff's body has no **Done:**, **Repo state:**; each of **Goal:**, **Done:**,
  **Next step:**, **Open decisions:**, and **Repo state:** starts a line of it"
And mem-0630-old.md is still there, and no note was written
```

### Scenario 4 — A bad title or no body (exception)

```gherkin
Scenario Outline: a bad release writes nothing
  Given a vault set in TIGHTSHIP_VAULT
  When the agent runs <command>
  Then it exits 1 with "error: <message>"
  And the vault is unchanged

  Examples:
    | command                                                     | message                                      |
    | release with --title of 121 characters                     | --title must be 1 to 120 characters          |
    | release with --title "   "                                  | --title must be 1 to 120 characters          |
    | release with no stdin                                       | the note's body must arrive on stdin         |
```

### Scenario 5 — Claim (FR-MEM-07.1, happy path)

```gherkin
Given a vault with the handoff notes/tightship/mem-0641-a.md, created 2026-10-14 and updated 2026-10-15
When the agent runs `tightship vault handoff claim` in the project "tightship"
Then it exits 0 and prints "# Handoff — tightship · mem-0641 · 2026-10-14 → 2026-10-15", a blank line, and the
  body
And mem-0641-a.md no longer exists
When the agent runs it again
Then it prints "no handoff for tightship"
```

### Scenario 6 — Nothing to claim (FR-MEM-07.3, exception)

```gherkin
Given a vault with no handoff in the scope "tightship", committed and clean
When the agent runs `tightship vault handoff claim` in the project "tightship"
Then it exits 0 and prints "no handoff for tightship"
And `git -C <vault> status --porcelain` prints nothing
```

### Scenario 7 — Two handoffs after two machines released (alternative)

```gherkin
Given the scope "tightship" holds the handoffs mem-0641, updated 2026-10-14, and mem-0642, updated 2026-10-15
When the agent runs `tightship vault handoff claim`
Then it prints mem-0642's section first, then mem-0641's
And neither note exists any more
```

### Scenario 8 — A handoff that cannot be removed (exception)

```gherkin
Given the handoff notes/tightship/mem-0641-a.md, in a folder the binary cannot write
When the agent runs `tightship vault handoff claim`
Then it prints the handoff, then exits 1 with "error: cannot remove notes/tightship/mem-0641-a.md: <the system's
  message>"
```

### Scenario 9 — The session start says how to claim (alternative)

```gherkin
Given a vault with a handoff in the scope "tightship"
When `tightship hook start` runs in the project "tightship"
Then the line after "## Handoff — tightship · <id> · <age>" and a blank line is "Claim it with the handoff command
  once you pick the work up."
```

### Scenario 10 — No vault, and a wrong subcommand (exception)

```gherkin
Given TIGHTSHIP_VAULT is absent
When the agent runs `tightship vault handoff claim`
Then it exits 1 with "error: no vault is set for this session; turn memory on, set the plugin's vault folder, and
  start a new session"
When the agent runs `tightship vault handoff fly`
Then it exits 1, and stderr holds "error: unknown subcommand fly", a blank line, and the usage
```

### Scenario 11 — Build the command with the option token (happy path)

```gherkin
Given content/commands/handoff.md as the Request writes it
When the maintainer runs `cargo xtask build`
Then it exits 0 and reports "24 pieces (13 skills, 10 commands, 1 agent), 37 files"
And dist/claude-code/skills/handoff/SKILL.md holds "Memory is `${user_config.memory}` for this plugin"
And it holds "/tightship:sync" and no "{{"
```

### Scenario 12 — The option token misused (exception)

```gherkin
Scenario Outline: the build rejects the token
  Given <file> holds <text> on line 3
  When the maintainer runs `cargo xtask build`
  Then it exits 1 with "error: <message>"

  Examples:
    | file                        | text                     | message                                                                                   |
    | content/commands/x.md       | {{option:nope}}          | content/commands/x.md:3: {{option:nope}} names no boolean option in content/options.toml  |
    | content/commands/x.md       | {{option:vault_folder}}  | content/commands/x.md:3: {{option:vault_folder}} names no boolean option in content/options.toml |
    | content/contract.md         | {{option:memory}}        | content/contract.md:3: {{option:memory}} works only in a skill, a command, or an agent    |
    | content/commands/x.md       | ${user_config.memory}    | content/commands/x.md:3: "${user_config" belongs to claude-code; use a {{capability:…}} token defined in harnesses/claude-code/harness.toml |
```

### Scenario 13 — The closeout's memory item is back (FR-CTR-03.1, FR-CTR-03.2)

```gherkin
When the maintainer runs `cargo xtask build`
Then dist/claude-code/contract/memory.md holds "also covers memory"
And no dist/claude-code/contract/part-*.md holds "tightship vault"
And dist/claude-code/skills/closeout/SKILL.md holds "## 5. Memory" and "- Memory:"
```

### Scenario 14 — Hand off from one machine to the other (FR-MEM-06.1, FR-MEM-07.1, happy path)

```gherkin
Given a session in a project whose vault folder is clone A, and a second configuration whose vault folder is
  clone B
When the adopter runs the command `handoff release` in the first
Then the agent replies "Handoff released: notes/<scope>/<file> — synced: 1 pushed, 0 pulled."
And the bare remote holds that handoff
When the adopter runs the command `handoff claim` in the second, in a clone of the same project
Then the agent shows the handoff's next step and open decisions, and waits
And neither clone B nor the bare remote holds a handoff for the scope
```

### Scenario 15 — Release leaves the project's repository as it was (FR-MEM-06.2)

```gherkin
Given the project's repository has a dirty file and an unpushed commit
When the adopter runs the command `handoff release`
Then the handoff's **Repo state** names the dirty file and the unpushed commit
And `git rev-parse HEAD`, `git for-each-ref`, `git stash list`, and `git status --porcelain` print what they
  printed before
```

### Scenario 16 — A commit the other machine lacks (FR-MEM-07.2, alternative)

```gherkin
Given a handoff whose **Repo state** names the unpushed commit abc1234
And the claiming clone does not have abc1234
When the adopter runs the command `handoff claim`
Then the agent says "Commit abc1234 is not in this repository; it stayed on the machine that released the
  handoff."
```

### Scenario 17 — Nothing to claim (FR-MEM-07.3, exception)

```gherkin
Given no handoff for the scope in the vault or its remote
When the adopter runs the command `handoff claim`
Then the agent says "No handoff waits for <scope>; nothing changed."
And the vault's notes are unchanged
```

### Scenario 18 — Memory off (FR-MEM-10.2, exception)

```gherkin
Given the plugin installed with memory set to false
When the adopter runs the command `handoff release`, then `handoff claim`
Then the agent says each time "Memory is off, so this project has no vault to hand off through. Turn the memory
  option on and start a new session to use it."
And the vault and the project's repository are unchanged
Given the plugin installed with memory never set, and a vault folder set
When the adopter runs the command `handoff claim`
Then the agent runs `tightship vault sync`
```

### Scenario 19 — No argument (exception)

```gherkin
When the adopter runs the command `handoff` with no argument
Then the agent says "Run the handoff command with `release` or `claim`." and runs nothing
```

### Scenario 20 — A claim whose first sync fails (exception)

```gherkin
Given a handoff in clone B for the scope, and a vault remote whose path no longer exists
When the adopter runs the command `handoff claim`
Then the agent shows "error: git fetch failed in <vault>: <git's message>" and what it needs from the adopter
And it does not run `tightship vault handoff claim`, and the handoff is still in clone B
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md), marked M1, #6:

- **Building blocks:**
  - tightship binary: `tightship vault` gains `handoff release|claim`.
  - Canonical content gains the command `handoff`, and the closeout command its memory item.
  - Harness definitions: `[references]` gains `option`, and `[neutrality] paths` gains `${user_config`.
  - Build renders the token `{{option:<name>}}`.
- **Runtime:** two new scenarios, "The adopter releases a handoff" and "The adopter claims a handoff", each
  through the command, the binary, and the sync.
- **Concepts:** memory gains a section on the handoff. Tokens gains the row `{{option:memory}}` →
  `${user_config.memory}`. Options says a skill, a command, or an agent reads an option only through that
  token, and treats the literal as on. Security: the handoff reaches the network only through the sync, and a
  claim runs `git fetch` in the project's repository.
- **Deployment:** N/A — nothing new is installed; the command ships with the plugin.
- **Diagrams:** N/A — no container, external system, or relation changes; the command is one more piece of
  canonical content.
- **ADR 0001** is amended: the token kinds gain `option`, whose value is the harness's own rendering of an
  option.

## Data model impact

Applied to [docs/data-model/](../../data-model/README.md), marked M1, #6:

- Vault gains a section, a handoff: its front matter, fixed `why`, and the five parts of its body. The row of
  `type` says `handoff` is written by `tightship vault handoff release`.
- The README's invariant "A scope holds at most one handoff" becomes "Releasing a handoff removes the scope's
  others", since two machines can hold one each until a claim.
- Access patterns: a release writes one handoff and removes the scope's others; a claim reads and removes every
  handoff of the scope.
- The conceptual model is unchanged.

## Out of Scope

- **Pushing or pulling a handoff without the sync:** the vault has one git path, epic #5's.
- **Merging two handoffs into one:** the claim prints both; the agent and the adopter read them.
- **Running any write command in the project's repository**, such as committing the dirty files before a
  release: the adopter's, with their approval.
- **A handoff with memory off**, kept outside the vault: FR-MEM-10 runs no memory function then.
- **Other harnesses' handoff command:** M2 to M4, each rendering `{{option:<name>}}` its own way.
- **The option token in hooks or in the contract:** a hook reads `CLAUDE_PLUGIN_OPTION_<NAME>`, and the contract
  is gated by sections (epic #4).
- **Showing the handoff on a resumed session:** epic #5's rule 3 prints the context on `startup`, `clear`, and
  `compact` only.

## Task Breakdown

| # | Issue | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- | --- |
| 1 | #32 | Release and claim the handoff in the binary | `tightship/src/vault/handoff.rs`, `tightship/src/vault.rs` (module), `tightship/src/vault/note.rs` (write a handoff), `tightship/src/vault/context.rs` (claim line), `tightship/src/failure.rs`, `tightship/src/main.rs` (subcommand, usage), `tightship/tests/handoff.rs`, `tightship/tests/hook.rs`, about 8 files | Scenarios 1 to 10 by tests | — |
| 2 | #33 | Ship the handoff command, the option token, and the closeout's memory item | `content/commands/handoff.md`, `content/commands/closeout.md`, `content/contract.md`, `harnesses/claude-code/harness.toml`, `xtask/src/build/tokens.rs`, `xtask/src/build/references.rs`, `xtask/src/build/harness.rs`, `xtask/tests/build.rs`, `xtask/tests/contract.rs`, about 9 files | Scenarios 11 to 13 by tests; 14 to 20 in sessions of an isolated Claude Code configuration | 1 |
