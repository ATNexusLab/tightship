+++
name = "audit"
description = "Full audit of a path or the current diff — security, performance, architecture, organization, maintainability"
argument-hint = "[path, module, or 'diff' — defaults to the working diff] [--only sec|perf|arch|maint]"
+++

Run a full audit on: {{capability:arguments}} (if empty, audit the current working diff via `git diff`).

Dispatch one {{capability:general-agent}} per axis, **in parallel, in one message**, each with the same
target and read-only (report findings, never edit). `--only <axis>` restricts the run to the named axes;
without it, all four run. Each agent loads its lens skill first.

| Axis | Lens |
| --- | --- |
| **sec** | `{{skill:security-audit}}` — map each touched surface to its threat cases (auth, input/output, mutation/state, uploads, network, crypto/secrets, dependencies). |
| **perf** | `{{skill:performance-analysis}}` — all three layers: frontend (critical path, interaction cost, what ships), backend (serial I/O, work inside the request, timeouts and retries), database (query count per row, plan at real volume, pool and lock waits). Evidence over speculation; no premature optimization. |
| **arch** | `{{skill:architecture-reading}}` — adherence to the repo `AGENTS.md` Architecture section and its ADRs: layer/boundary violations, leaked dependencies, misplaced files. |
| **maint** | the contract's **Code I write** — organization and maintainability: duplication, mixed responsibility, god-objects, dead code, magic/hidden behavior, speculative abstraction, type-safety escapes, opaque names. |

`{{command:audit}}` targets an arbitrary path or diff. For the current branch alone,
{{capability:security-review-command}} and {{capability:review-command}} cover sec and maint natively and
run in this session instead.

**Conditional lenses.** Three more skills load when the target calls for them:

- **API contract** (HTTP/RPC endpoints, a GraphQL schema, webhook or event payloads) → **sec** and
  **arch** also load `{{skill:api-design}}` and judge the contract itself: authz per endpoint,
  unknown-field rejection, error shape and leakage, bounded collections, idempotency and concurrency,
  breaking changes inside a version.
- **Data layer** (schema, migrations, or the code that queries them) → **sec**, **perf**, and **arch** also
  load `{{skill:database-design}}`: invariants left to application code, predicates that can yield `NULL`, tenancy
  and privileges, indexes that no access pattern matches, and migrations that break the running version.
- **UI surface** (screens, components, forms, client-side routing) → **sec** and **maint** also load
  `{{skill:frontend-architecture}}`: rules enforced only on the client, secrets past the server/client boundary,
  remote data copied into global state, shareable view state missing from the URL, async surfaces without
  all four states, and keyboard or focus gaps.

A skill is the lens, never the ceiling: a real problem outside its checklist counts, under the same
admission gate as everything else.

## Read the intent first

Before judging anything, each agent reads what governs the target: the root and area `AGENTS.md`,
`docs/requirements/`, `docs/decisions/`, the spec of the feature the target belongs to, and the issue
it is linked to. Behavior one of them states as intended is **not a finding**. If the decision itself
looks risky, it is reported as a **decision to review**, citing its source, never as a bug.

## Admission gate

A finding exists only with:

- a **concrete scenario** in plain words: who does what, with what input, and what goes wrong;
- a `path:line` the agent actually read.

A risk that cannot be written as a scenario is not a finding. A term like IDOR or TOCTOU never stands
in for the scenario. **Zero findings on an axis is a valid result**, reported as such.

Every agent returns each finding as: severity (**Critical / High / Medium / Low**) · `path:line` ·
scenario · why it matters in this project · concrete fix · rough effort · the decision doc it checked.

## Consolidate

Merge the four reports into one ranked list, deduped (the same code often trips more than one axis —
keep it once, tagged with every axis it hits). Rank by severity across axes, not per-axis.

Then each Critical, High and Medium finding goes to its own `{{agent:adversarial-verifier}}`, all in
parallel in one message. A finding it refutes leaves the list. Low is not verified and appears only as a
count.

For each surviving finding, search the backlog via `gh` for an issue that already owns it.

## Report — short, then one at a time

In the chat's language, with no jargon the scenario does not explain:

1. One line: the count per severity, or "no findings".
2. A triage table: `#` · severity · the problem in one plain sentence · `path:line` · owning issue
   (`#N` or `—`).
3. One line: `Decisions to review: N · Low: N · Refuted: N`. Their detail comes only when I ask.
4. **Finding 1 of N** in detail: what happens (the scenario) · why it matters · where (`path:line` with
   the snippet) · the fix and its effort.
5. Stop and wait for my call on it: fix now · issue · drop. Then the next finding, the same way.

## After the report

Nothing is fixed and nothing is filed until I decide — a comment on an existing issue counts as filing.

- What I send to a fix follows the dev loop (tests first where it matters).
- A finding in code this session built is fixed, never sent to an issue: it is not debt.
- What I send to an issue: security findings → labeled `security-debt` (surface · risk · fix ·
  severity); the other axes → labeled `tech-debt`, same shape.
- A deliberate trade-off is not debt; where it is recorded is the contract's **Security** rule.
- Once every finding has my call, summarize: fixed vs deferred (with issue numbers), broken out per axis.

If a finding needs an architectural change, flag it so a `{{command:spec}}` ADR is written before implementing.
