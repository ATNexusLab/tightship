+++
name = "closeout"
description = "Check what the delivery gate does not — surfaces, docs, traceability, security debt, memory — before the commit"
argument-hint = "[optional scope note]"
+++

Run the closeout for the delivery in progress, after its gates are green and it has been run and
observed, and before it is handed over for the commit. Those gates and run-to-verify belong to the
delivery itself and are not repeated here. Each item below gets a recorded decision
(`done at <path>` / `n/a` / `deferred → owner #n or none`). **Silence on any item is a failure.**

Nothing here assumes a stack. Discover the repo's tooling, commands, and conventions from its own
config — never invent them, never assume a particular linter, runner, framework, or database.

## 0. Scope — which surfaces this diff touched

List the changed files (`git diff --name-only`, plus untracked). From them, name the surfaces in
play. **Only the surfaces you name run in §1**, so name them explicitly — an unnamed surface is a
scope decision on the record, not an omission.

## 1. Surface gates

Run only the surfaces named in §0.

### Interface — anything a person sees or operates

- Keyboard reachable, focus visible, controls labeled, contrast holds.
- Loading, empty, and error states exist — not only the happy path.
- Holds at the smallest viewport or terminal width the project supports.
- No layout shift or re-render storm on interaction.
- Nothing needlessly heavy added to what ships to the client.
- User-facing copy in the project's UI language.

### Request handling — anything answering a caller

- Input validated at the boundary; rejection is typed, never a bare string.
- Authorization checked per resource, not merely authentication.
- Errors propagate with actionable messages; nothing swallowed.
- Response shape and status match the documented contract.
- Repeated or retried calls are idempotent where they must be.

### Data & persistence

- Migration is safe against rows that already exist.
- New query paths have the constraints and indexes they need.
- No N+1, no unbounded result set.
- Nothing sensitive newly logged or stored in the clear.

### Async & background work

- Failure is visible — logged or alerted, never silent.
- Retry is bounded and idempotent.
- No unbounded queue or backlog growth.

### Configuration & secrets

- No env fallback at the point a variable is read; a validation file only where something real needs it.
- New variables documented in the committed example/template.
- No secret hardcoded, logged, or committed.

## 2. Docs upkeep (only when warranted)

Use `{{skill:technical-writing}}` as the lens — including its first question, whether the doc should
exist at all.

- Public contract changed (API/CLI/env/schema)? → update `README.md` / `docs/`.
- Architectural decision made? → `{{command:spec}}` an ADR in `docs/decisions/`.
- Setup/run commands changed? → update README Quick Start.
- Architecture or data changed? → `docs/architecture/`, `docs/data-model/`, and their diagrams match
  what was built (`{{skill:project-docs}}`).
- A screen, the brand, or a token changed? → `docs/design/` matches what was built, and the values in
  `docs/stylesheets/docs-theme.css` match the app's tokens.
- A use case changed? → its diagram in `docs/requirements/diagrams/`.

## 3. Traceability — always, never `n/a`

- The work has an issue, and the chain holds: requirement ID → epic and its spec → sub-issue → the
  epic's branch, commits and PR, as `{{skill:delivery}}` names and sizes them.
- Once the PR is merged into the integration branch, the epic and its sub-issues move to Staging
  (`gh project item-edit`). They close, and go to Done, when the release reaches `main`.
- The issue's Status in the Project matches reality, and it sits in the current `Sprint`.

## 4. Deferred work and security debt

What this delivery built is fixed, never deferred; only what it did not create can be — a problem outside
the task, or one held back on purpose or by a limitation. Everything deferred is reported, whether deferred
explicitly or raised and left unanswered: the backlog is searched, and each item is reported with the issue
that already owns it — often the next delivery, named by its spec — or with none. A new issue, or a comment
on one, waits for my word, and every one opened is reported.

Review open `security-debt` issues touching this area (via `gh`). Security may be deferred, never dropped:
a deferred item becomes, at my word, an issue labeled `security-debt` with the surface, the risk, and the
fix. A deliberate trade-off written down with its mitigations is not debt: an architectural one is an ADR,
any other goes to the doc it concerns or the repository's `AGENTS.md`.

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

## Output

```text
Closeout:
- Surfaces:      [which the diff touched]
- Surface gates: [per surface: pass / findings / n/a by scope]
- Docs:          [updated at <path> / n/a]
- Traceability:  [issue #n · branch · PR → integration branch · Status/Sprint ok]
- Security-debt: [none / issues #… / filed #…]
- Memory:        [promoted <note> / nothing durable / n/a — no vault]
```

Do not hand the delivery over for the commit until this block is produced with no failing item.
