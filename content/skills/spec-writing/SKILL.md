+++
name = "spec-writing"
description = "Use when writing a design document before implementation — a feature spec, an ADR, or acceptance criteria. Carries the templates and the gate that no spec ships with unresolved ambiguity."
+++

# Spec Writing

## When to use

| Artifact | For |
| --- | --- |
| **Feature spec** | A feature that will be built and tracked. Maps the whole behavior, then breaks into tasks. → `templates.md` |
| **ADR** | An architectural decision. → template below |

A feature spec is written by `{{command:spec}}` and published by `{{command:epic}}`. The two are separate
on purpose: the spec closes before a single issue exists.

## Core rule — a spec is closed or it is not a spec

A **closed** spec has zero unresolved blocking ambiguity. Not "documented as a risk" — resolved. A `Risks`
section carried into implementation means the design is unfinished.

The gate, checked before the status moves to `closed`:

- No placeholder or hedge in the spec's language — `TBD`, `???`, "to be defined", "maybe" and their
  translations — and no question mark in a declarative sentence.
- Every user-facing message is the **literal string**, in quotes — not "an error message".
- Every limit is a **number** — not "a reasonable limit".
- Every field states type, obligatoriness, and where its data comes from.
- Every business rule has at least one Gherkin scenario.
- Out of Scope is filled in. An empty out-of-scope section means the boundary was never drawn.

When something genuinely cannot be resolved now, it goes to Out of Scope with the reason — it leaves
the spec entirely. It never stays as a soft spot inside it.

## Feature spec

Two profiles — **UI** and **API** — in `templates.md`. Read that file before writing one.

Location: `docs/specs/<epic>/<feature>.md`.

Every feature spec ends with its Task Breakdown — the table `{{command:epic}}` publishes verbatim. Rules for
slicing it are in `templates.md`.

## ADR template → `docs/decisions/NNNN-<slug>.md`

```markdown
# NNNN. <Title>
- Status: proposed | accepted | superseded by NNNN
- Date: <YYYY-MM-DD>

## Context
The forces at play: problem, constraints, what must hold true.

## Decision
The choice, stated plainly.

## Consequences
What becomes easier, what becomes harder, what we accept.

## Alternatives considered
Option · why rejected.
```

An ADR must conform to — or explicitly amend — the repo's `Architecture` section.

## Acceptance criteria format

Gherkin with the keywords of the repo's doc language: `Given <state>, When <action>, Then <observable
result>`, or that language's equivalents. Each must
be testable — if it has no metric or observable result, sharpen it. Coverage floor: one happy path, one
alternative per dynamic behavior, one exception per rule that can fail.

## Validation checklist

- [ ] The closed-spec gate above passes, line by line.
- [ ] Every business rule maps to at least one scenario.
- [ ] Every scenario maps to at least one task in the Task Breakdown.
- [ ] Depends on carries task numbers, not prose.
- [ ] The doc respects (or knowingly amends) the decided architecture.
- [ ] Requirements lists the requirement IDs of its epic, and every acceptance criterion of each one is
      covered by a scenario.
- [ ] Architecture impact and Data model impact name elements as `docs/architecture/`
      and `docs/data-model/` name them, and those docs carry the change.
- [ ] An ADR states context, decision, consequences, and alternatives.

## Never do

- Write code from a spec command — design only.
- Invent a requirement, a permission name, a message, or a limit the user has not confirmed.
- Open an issue from a spec that is still `draft`.
