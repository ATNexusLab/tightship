# Support every surface of each harness

> **Status:** closed
> **Profile:** API
> **Module:** `docs/guides/install/`, `docs/guides/release/checklist.md`, `docs/system/architecture/deployment.md`
> **Epic:** [#134](https://github.com/ATNexusLab/tightship/issues/134)
> **Requirements:** NFR-FLEX-01, NFR-FLEX-02, NFR-FLEX-03, NFR-FLEX-08, FR-INS-08; new: NFR-FLEX-09
> **Sprint:** 4

Each harness runs on more than one surface on the adopter's machine: a CLI, a desktop app, an IDE, Zed.
The plugin is built and checked on some of them only, so an adopter on the others cannot tell whether it
installs there or which of its pieces run. One support matrix answers that for every local surface, each
install guide says how the plugin gets to each of its harness's surfaces, and the release checklist runs on
every one of them.

Terms, as used here:

- **Harness**: a coding agent tightship ships a plugin for: Claude Code, Cursor, and Antigravity CLI
  ([NFR-FLEX-01](../../../product/requirements/non-functional/flexibility.md#nfr-flex-01) to
  [NFR-FLEX-03](../../../product/requirements/non-functional/flexibility.md#nfr-flex-03)). Codex has no
  plugin yet.
- **Surface**: one way to run a harness on the adopter's machine: its CLI, its desktop app, its IDE, or Zed.
  Each row of the matrix is one surface.
- **Web surface**: a harness session opened from a browser or run on the vendor's machines: claude.ai,
  claude.ai/code, Cursor's Cloud Agents, Antigravity's Remote Agents. Not supported.
- **Piece**: what the plugin delivers to a session: the contract, the skills, the commands, the hooks, the
  status line, and the vault
  ([FR-SKL-01](../../../product/requirements/functional/skills.md#fr-skl-01)).

## Decisions this spec rests on

Confirmed by the maintainer on 2026-11-17, in the grilling of
[#134](https://github.com/ATNexusLab/tightship/issues/134):

1. **Codex leaves this epic and the sprint.** Its rows wait in the backlog with
   [#142](https://github.com/ATNexusLab/tightship/issues/142); NFR-FLEX-04 and OQ-04, OQ-06, and OQ-08 do
   not change.
2. **SRS 4.6.0 adds NFR-FLEX-09**, in this spec's commit, as the spec of #158 added NFR-FLEX-08.
3. **Only local surfaces are supported.** tightship is a software engineering workflow that works on a
   checkout on the adopter's machine; a web surface has no such checkout and no `tightship` binary. The
   matrix names the web surfaces once, as not supported, and has no row for them.
4. **Every surface is checked in a live session, piece by piece.** The Zed agent panel rows come from
   [#163](https://github.com/ATNexusLab/tightship/issues/163), which this epic does not redo.
5. **A piece that does not load on a surface is recorded, not fixed.** A fix is its own issue.
6. **The release checklist runs a desktop app or IDE on macOS only**; a CLI keeps running on macOS, Linux,
   and Windows.
7. **Profile API**: there is no screen; the contract is the text of the docs below.

## Acceptance Criteria

### Contract

| Doc | Change | Read by |
| --- | --- | --- |
| `docs/guides/install/README.md` and `.pt-BR.md` | New: the support matrix | The adopter |
| `docs/guides/install/<harness>/install.md` and `.pt-BR.md` | The section `In Zed` (`No Zed`) becomes `Surfaces` (`Superfícies`), one paragraph per surface | The adopter |
| `docs/guides/install/<harness>/README.md` and `.pt-BR.md` | One line under `Before you start` links the matrix | The adopter |
| `docs/guides/release/checklist.md` | A section `Surfaces`, one row per surface | The maintainer before a release |
| `docs/system/architecture/deployment.md` | The surfaces each harness runs on, on the adopter's machine | The maintainer |

### Request

What each row of the matrix comes from:

| Input | Type | Required | Source | Rule |
| --- | --- | --- | --- | --- |
| The surface | name | Yes | The list in [Response](#the-surfaces) | One row each, in that order |
| How the plugin gets there | text | Yes | The install guide of the harness | The installer, the harness's own command, or the same install as another surface |
| Each piece's result | one of three values | Yes | A live session on that surface, after a fresh install | Rule 3 |
| The reason a piece does not load | text, at most 120 characters | When the result is `No` | The live session, or the vendor's docs, linked | Rule 4 |

### Response

#### The surfaces

| Harness | Surface | How the plugin gets there |
| --- | --- | --- |
| Claude Code | CLI | The installer, or `claude plugin install` |
| Claude Code | Desktop app | The same install as the CLI: the app's local sessions read `~/.claude` |
| Claude Code | Zed's terminal | The same install as the CLI |
| Claude Code | Zed's agent panel | The `claude-acp` registry entry ([#163](https://github.com/ATNexusLab/tightship/issues/163)) |
| Cursor | Editor | The installer, or a copy into `~/.cursor/plugins/local/` |
| Cursor | `agent` CLI | The same install as the editor |
| Cursor | Zed's terminal | The same install as the editor |
| Cursor | Zed's agent panel | The `cursor` registry entry ([#163](https://github.com/ATNexusLab/tightship/issues/163)) |
| Antigravity | CLI | The installer, or `agy plugin install` |
| Antigravity | IDE | The same install as the CLI, if the IDE reads `~/.gemini/config/plugins/` (Rule 3) |
| Antigravity | Desktop app | The same install as the CLI, if the app reads `~/.gemini/config/plugins/` (Rule 3) |
| Antigravity | Zed's terminal | The same install as the CLI |
| Antigravity | Zed's agent panel | The `antigravity-acp` registry entry ([#163](https://github.com/ATNexusLab/tightship/issues/163)) |

#### The matrix page

`docs/guides/install/README.md` opens with, word for word:

> "Where tightship runs: one row per surface of each harness on your machine, with how the plugin gets there
> and which of its pieces load. A piece that does not load says why."

Then one table, its columns in this order: `Harness`, `Surface`, `How it gets there`, `Contract`,
`Skills`, `Commands`, `Hooks`, `Status line`, `Vault`. A piece's cell holds one of three values, word for
word:

| Value | pt-BR | When |
| --- | --- | --- |
| `Yes` | `Sim` | The live session showed the piece working |
| `No¹` | `Não¹` | It did not; the footnote with that number, under the table, gives the reason |
| `Not checked yet` | `Ainda não checado` | No live session on that surface yet; the cell links its issue |

A surface the plugin cannot reach has `No way in¹` (`Sem como instalar¹`) in `How it gets there` and `No¹`
in every piece, all pointing to the same footnote. Under the footnotes, word for word:

> "Web surfaces are not supported: claude.ai, claude.ai/code, Cursor's Cloud Agents, and Antigravity's Remote
> Agents. tightship works on a checkout on your machine."

The page ends with a line linking each harness's install guide.

#### The `Surfaces` section of each install guide

One paragraph per surface of that harness, in the matrix's order: how the plugin gets there, and each piece
that does not load, with its reason. The Zed paragraphs of today's `In Zed` section move here unchanged.
The section's first line, word for word:

> "Which pieces load on each surface is in the [support matrix](../README.md)."

#### The release checklist

A section `Surfaces`, after `Cursor command-line agent`, one row per surface whose matrix row is not `No
way in`. The criterion column reads `NFR-FLEX-09.2`; the check, for each row:

> "Manual: install as the matrix says, open a session on \<surface\>, and check each piece the matrix marks
> `Yes`: the contract answers a question about itself, a skill and a command run, a hook prints at session
> start, the vault's pinned notes show, and the status line renders."

A desktop app or IDE row writes `n/a` in the Linux and Windows columns.

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Adopter | N/A | Each surface runs with the harness's own account and permissions; nothing here adds one |
| Maintainer | N/A | Runs the live checks with their own accounts on each surface |

---

## Business Rules

### 1. One row per surface

Every surface in [The surfaces](#the-surfaces) has one row in the matrix, in that order, and a paragraph in
its harness's install guide. A surface added to that list adds both in the same commit.

### 2. How the plugin gets there

A surface gets the plugin from the installer, from the harness's own command, or from the install of
another surface of the same harness, as its install guide says. No surface adds an install path of its own.

### 3. The checks

- **When:** once per surface by this epic, and at each release by the checklist (Rule 5).
- **What:** after a fresh install, a live session on the surface checks the six pieces one by one, with the
  check in [The release checklist](#the-release-checklist). The result goes in the cell, `Yes` or `No¹`.
- **A surface that does not read the install** of another surface (the Antigravity IDE or desktop app not
  reading `~/.gemini/config/plugins/`) gets `No way in¹`, with the reason.
- **Zed's agent panel:** the cells read `Not checked yet` and link
  [#163](https://github.com/ATNexusLab/tightship/issues/163) until it closes; #163 writes them.

### 4. A piece that does not load

- Its cell reads `No¹`, and the footnote gives the reason in at most 120 characters: what the surface lacks,
  linking the vendor's doc that says so when one does.
- A reason that a change to tightship could remove opens its own issue, linked from the footnote. This
  epic fixes none.

### 5. The release checklist

The `Surfaces` section runs on every release, on the systems Decision 6 sets. A surface whose result
differs from the matrix fails its row until the matrix and the guide say the new result.

### 6. Persistence and audit

- **Files changed:** the docs in [Contract](#contract).
- **Audit:** N/A, the docs are text; git records every change.
- **Events:** N/A.

---

## Errors

| Code | When | Message |
| --- | --- | --- |
| N/A | No code changes: every piece is a doc the adopter or the maintainer reads | N/A |

## Side Effects

- **Persistence:** the docs in Rule 6.
- **Concurrency:** N/A, one session writes them.
- **Transaction:** a surface's matrix row, its guide paragraph, and its checklist row change in the same
  commit, so none of the three disagrees with another.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — A desktop app loads every piece (happy path)

```gherkin
Given tightship installed with the installer on macOS
When a local session starts in Claude Code's desktop app
Then the contract, a skill, a command, a hook, the status line, and the vault are checked one by one
And the matrix row "Claude Code — Desktop app" holds the result of each piece
And the Surfaces section of Claude Code's install guide says the app needs no install beyond the CLI's
```

### Scenario 2 — A piece missing on a surface (alternative)

```gherkin
Given a surface where the status line does not render
When its live check runs
Then its Status line cell reads "No¹"
And the footnote gives the reason in at most 120 characters
And no code changes in this epic
```

### Scenario 3 — A surface that reads another surface's install (alternative)

```gherkin
Given tightship installed in Antigravity CLI with the installer
When a session starts in the Antigravity IDE and in the Antigravity desktop app
Then each piece is checked on each, and the rows "Antigravity — IDE" and "Antigravity — Desktop app" hold the results
And their paragraphs in the Antigravity guide need no step beyond the CLI's install
```

### Scenario 4 — A surface that does not read the install (exception)

```gherkin
Given tightship installed in Antigravity CLI with the installer
And an Antigravity surface that does not load plugins from ~/.gemini/config/plugins/
When its check runs
Then its row reads "No way in¹" and "No¹" in every piece, with the reason in the footnote
And the release checklist has no row for it
```

### Scenario 5 — Zed's agent panel not checked yet (alternative)

```gherkin
Given #163 still open
When the matrix is written
Then every piece of the three Zed agent panel rows reads "Not checked yet" and links #163
And the Zed terminal rows read "Yes" in every piece the harness's CLI row reads "Yes"
```

### Scenario 6 — Web surfaces are named, not supported

```gherkin
Given the matrix page
When the adopter reads it
Then it has no row for claude.ai, claude.ai/code, Cursor's Cloud Agents, or Antigravity's Remote Agents
And it says, word for word, "Web surfaces are not supported: claude.ai, claude.ai/code, Cursor's Cloud Agents, and Antigravity's Remote Agents. tightship works on a checkout on your machine."
```

### Scenario 7 — Every surface has a row and a paragraph

```gherkin
Given the thirteen surfaces of the spec
When the matrix and the three install guides are read
Then each surface has one matrix row and one paragraph in its harness's Surfaces section, in English and pt-BR
```

### Scenario 8 — The release runs on every surface

```gherkin
Given the matrix with every row filled
When the maintainer runs the release checklist
Then the Surfaces section has one row per surface that is not "No way in"
And each desktop app or IDE row reads "n/a" for Linux and Windows
```

---

## Architecture impact

- **Deployment** (`deployment.md`): the `Adopter's machine` row names every harness and its surfaces there,
  the CLI, the desktop app, and the IDE, and links the support matrix. Marked with this epic.

## Data model impact

N/A — no entity, file, or attribute of the data model changes: the matrix is a doc.

## Out of Scope

- **Web surfaces**: claude.ai, claude.ai/code, Cursor's Cloud Agents, and Antigravity's Remote Agents
  (Decision 3).
- **Codex's surfaces**: Codex has no plugin;
  [#142](https://github.com/ATNexusLab/tightship/issues/142), in the backlog, covers them.
- **Fixing a piece that does not load on a surface**: each gets its own issue (Rule 4).
- **Zed's agent panel check**: [#163](https://github.com/ATNexusLab/tightship/issues/163) owns it.
- **Running a desktop app or IDE on Linux and Windows at release**: Decision 6.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- |
| 1 | Write the support matrix from a live check of every surface | Live checks of Claude Code's CLI and desktop app, Cursor's editor and `agent`, Antigravity's CLI, IDE, and desktop app; `docs/guides/install/README.md` and `.pt-BR.md` (new); the `Surfaces` section of the six `install.md`; the line in the six `README.md`; `docs/system/architecture/deployment.md` | Scenarios 1 to 7 | — |
| 2 | Run the release checklist on every surface | `docs/guides/release/checklist.md` | Scenario 8 | 1 |
