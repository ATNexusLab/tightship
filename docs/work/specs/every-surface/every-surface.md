# Support every surface of each harness

> **Status:** published
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
4. **The matrix records what the unit and integration tests prove; the rest is confirmed in use.** Revised
   in the grilling of [#168](https://github.com/ATNexusLab/tightship/issues/168): no live session runs in
   this epic. A desktop app or IDE, which no test runs, waits for
   [#170](https://github.com/ATNexusLab/tightship/issues/170). The Zed agent panel rows come from
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
| Each piece's result | one of three values | Yes | A test of the suite, or a session in use on that surface | Rule 3 |
| The reason a piece does not load | text, at most 120 characters | When the result is `No` | The test or the session, or the vendor's docs, linked | Rule 4 |

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
| Antigravity | IDE | The same install as the CLI, if the IDE reads `~/.gemini/config/plugins/` ([#170](https://github.com/ATNexusLab/tightship/issues/170)) |
| Antigravity | Desktop app | The same install as the CLI, if the app reads `~/.gemini/config/plugins/` ([#170](https://github.com/ATNexusLab/tightship/issues/170)) |
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
| `Yes` | `Sim` | A test of the suite covers the piece on the binary this surface runs, or a session in use showed it working |
| `No¹` | `Não¹` | A test or a session showed it not loading; the footnote with that number, under the table, gives the reason |
| `Not checked yet` | `Ainda não checado` | No test covers it and no session confirmed it yet; the cell links its issue |

A footnote under a row of `Yes` cells names the tests behind them and what those tests leave uncovered.
Under the footnotes, word for word:

> "Web surfaces are not supported: claude.ai, claude.ai/code, Cursor's Cloud Agents, and Antigravity's Remote
> Agents. tightship works on a checkout on your machine."

The page ends with a line linking each harness's install guide.

#### The `Surfaces` section of each install guide

One paragraph per surface of that harness, in the matrix's order: how the plugin gets there, and each piece
that does not load, with its reason. The Zed paragraphs of today's `In Zed` section move here unchanged.
The section's first line, word for word:

> "Which pieces load on each surface is in the [support matrix](../README.md)."

#### The release checklist

A section `Surfaces`, after `Cursor command-line agent`, one row per surface of the matrix. The criterion
column reads `NFR-FLEX-09.2`; the check, for each row:

> "Manual: install as the matrix says, open a session on \<surface\>, and check each piece the matrix marks
> `Yes`: the contract answers a question about itself, a skill and a command run, a hook prints at session
> start, the vault's pinned notes show, and the status line renders."

A desktop app or IDE row writes `n/a` in the Linux and Windows columns.

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Adopter | N/A | Each surface runs with the harness's own account and permissions; nothing here adds one |
| Maintainer | N/A | Confirms the desktop apps and IDEs in use with their own accounts ([#170](https://github.com/ATNexusLab/tightship/issues/170)) |

---

## Business Rules

### 1. One row per surface

Every surface in [The surfaces](#the-surfaces) has one row in the matrix, in that order, and a paragraph in
its harness's install guide. A surface added to that list adds both in the same commit.

### 2. How the plugin gets there

A surface gets the plugin from the installer, from the harness's own command, or from the install of
another surface of the same harness, as its install guide says. No surface adds an install path of its own.

### 3. The checks

- **When:** once by this epic, from the suite; in use, for a surface no test runs; and at each release by
  the checklist (Rule 5).
- **What:** a piece reads `Yes` when a unit or integration test covers it on the binary the surface runs: a
  CLI, and Zed's terminal running that CLI. The footnote names the tests and what they leave uncovered.
- **A desktop app or IDE**, which no test runs, reads `Not checked yet` in every piece and links
  [#170](https://github.com/ATNexusLab/tightship/issues/170). A session in use on it checks the six pieces
  with the check in [The release checklist](#the-release-checklist), and its result replaces the cell.
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

### Scenario 1 — A CLI covered by the suite (happy path)

```gherkin
Given the unit and integration tests of Claude Code's plugin
When the matrix is written
Then every piece of the row "Claude Code — CLI" reads "Yes"
And its footnote names the tests behind those cells and what they leave uncovered
```

### Scenario 2 — A piece missing on a surface (alternative)

```gherkin
Given a surface where the status line does not render
When a test or a session in use shows it
Then its Status line cell reads "No¹"
And the footnote gives the reason in at most 120 characters
And no code changes in this epic
```

### Scenario 3 — A surface that reads another surface's install (alternative)

```gherkin
Given tightship installed in Antigravity CLI with the installer
When the matrix is written
Then every piece of the rows "Antigravity — IDE" and "Antigravity — Desktop app" reads "Not checked yet" and links #170
And their paragraphs in the Antigravity guide need no step beyond the CLI's install
```

### Scenario 4 — A desktop app confirmed in use (alternative)

```gherkin
Given the row "Claude Code — Desktop app" reading "Not checked yet"
When a session in use on the app checks the six pieces
Then each cell reads "Yes" or "No¹", with the reason in the footnote
And the app's paragraph in Claude Code's install guide changes in the same commit
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
Then the Surfaces section has one row per surface of the matrix
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
- **A live session on a desktop app or IDE**: [#170](https://github.com/ATNexusLab/tightship/issues/170)
  confirms them in use (Decision 4).
- **Running a desktop app or IDE on Linux and Windows at release**: Decision 6.

## Task Breakdown

| # | Issue | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- | --- |
| 1 | [#168](https://github.com/ATNexusLab/tightship/issues/168) | Write the support matrix of every surface | The suite's tests behind each CLI row; `docs/guides/install/README.md` and `.pt-BR.md` (new); the `Surfaces` section of the six `install.md`; the line in the six `README.md`; `docs/system/architecture/deployment.md` | Scenarios 1 to 3, 5 to 7 | — |
| 2 | [#169](https://github.com/ATNexusLab/tightship/issues/169) | Run the release checklist on every surface | `docs/guides/release/checklist.md` | Scenario 8 | 1 |
| 3 | [#170](https://github.com/ATNexusLab/tightship/issues/170) | Confirm the GUI surfaces in use | The desktop app and IDE rows of the matrix and their paragraphs in the install guides | Scenario 4 | 1 |
