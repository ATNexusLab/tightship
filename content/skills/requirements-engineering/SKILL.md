+++
name = "requirements-engineering"
description = "Use when eliciting, analysing, writing, validating, or changing a project's requirements — the docs/requirements/ SRS folder, its requirement IDs and attributes, its versioning, and change control. Based on ISO/IEC/IEEE 29148 and the ISO/IEC 25010 quality model, run inside Scrum sprints."
+++

# Requirements Engineering

Requirements are not static. They are elicited, analysed, specified, validated, and then **managed**:
every change is proposed, analysed for impact, approved, and versioned. A sprint plans against a known
version of them.

## The process

| Phase | What happens | Output |
| --- | --- | --- |
| **Elicitation** | Interview the stakeholders; read the repo, the ADRs, the existing product. Ask only what cannot be inferred. | Raw needs, each with its source |
| **Analysis** | Classify each need (functional, non-functional, business rule, constraint), find conflicts, duplicates, and dependencies, check feasibility, prioritise with MoSCoW. | Needs settled with the stakeholder |
| **Specification** | Write each need as a requirement with its ID and attributes, into the SRS. | `docs/requirements/` |
| **Validation** | Run the quality checklist on every requirement touched and on the whole set. A stakeholder confirms it. | Status `approved` |
| **Management** | Change control, versioning, and trace to epics. | Version history, baselines |

## The SRS — `docs/requirements/`

A folder, not one file: people read one area at a time. Doc language follows the repo; IDs keep their
`FR`/`NFR`/`BR` form, and file names and section anchors follow the repo's code language. The folder
follows the rules of `{{skill:project-docs}}`.

```text
docs/requirements/
  README.md                  the SRS front page and the index of every requirement
  overview.md                stakeholders, operating environment, constraints, assumptions and dependencies
  glossary.md                every domain term the requirements use, one line each
  functional/<area>.md       FR-<AREA>-* of one area
  non-functional/<char>.md   NFR-<CHAR>-* of one ISO/IEC 25010 characteristic
  business-rules.md          BR-*: policies that hold regardless of feature
  open-questions.md          undecided items; each is deleted once settled
  CHANGELOG.md               version history
```

`README.md`:

```markdown
# <Product> — Software Requirements Specification

**Version:** <MAJOR.MINOR.PATCH> · **Date:** <YYYY-MM-DD> · [Changelog](CHANGELOG.md)

## Purpose
The problem the product solves and why it exists.

## Scope
In scope. Out of scope, each with its reason.

## Contents
[Overview](overview.md) · [Glossary](glossary.md) · [Business rules](business-rules.md) ·
[Open questions](open-questions.md)

## Requirements
| ID | Name | Priority | Status | Milestone |
| --- | --- | --- | --- | --- |
| [FR-<AREA>-01](functional/<area>.md#fr-<area>-01) | <short name> | Must | approved | M1 |
```

The index table is how the whole set is read at a glance; every row links to the requirement. A file per
area opens with the area's code and one line of what it covers.

### A requirement

```markdown
## FR-<AREA>-NN — <short name>

> As a <role>, I want <capability>, so that <benefit>.

The system shall <one capability, stated so it can be tested>.

| Attribute | Value |
| --- | --- |
| Rationale | <why it exists> |
| Source | <stakeholder, ADR, or regulation> |
| Priority | Must / Should / Could / Won't (this release) |
| Status | proposed / approved / retired |
| Milestone | M<N> |
| Since | v<version that introduced it> |

**Acceptance criteria**
- **FR-<AREA>-NN.1** — Given <state>, when <action>, then <observable result>.
```

- **Functional:** `FR-<AREA>-NN`, with the user story. **Non-functional:** `NFR-<CHARACTERISTIC>-NN`,
  without a story, with a measurable criterion (a number and its unit). **Business rule:** `BR-NN`.
- `<AREA>` is a short uppercase code declared at the top of its file (`AGT`, `PRV`, …).
  `<CHARACTERISTIC>` is one of the nine ISO/IEC 25010:2023 characteristics: `FUNC` functional
  suitability · `PERF` performance efficiency · `COMP` compatibility · `INTR` interaction capability ·
  `REL` reliability · `SEC` security · `MNT` maintainability · `FLEX` flexibility · `SAFE` safety.
- **An ID is never reused or renumbered.** A retired requirement stays in place with Status `retired`,
  the version that retired it, and why.
- Acceptance criteria stay at capability level. Field types, literal messages, and numeric limits of a
  feature belong to its `{{command:spec}}`, which cites the requirement IDs it details.

## Quality checklist (ISO/IEC/IEEE 29148)

Each requirement is:

- **Necessary** — removing it leaves a stakeholder need unmet.
- **Appropriate** — at the level of the product, not the design of one feature.
- **Unambiguous** — one reading only; no "fast", "easy", "etc.", "and/or", "as appropriate".
- **Complete** — needs nothing outside itself to be understood.
- **Singular** — one capability; two "shall"s are two requirements.
- **Feasible** — achievable within the constraints.
- **Verifiable** — each acceptance criterion has an observable result or a number.
- **Correct** — it states the need its source expressed.
- **Conforming** — it follows this template.

The set is **complete**, **consistent** (no two requirements contradict), **feasible** as a whole,
**comprehensible**, and **able to be validated** by a stakeholder reading it.

## Versioning

SemVer on the document, bumped in the same commit as the change:

| Bump | When |
| --- | --- |
| MAJOR | Scope changes, a milestone's content moves, or a requirement is retired |
| MINOR | A requirement or acceptance criterion is added |
| PATCH | Wording or attributes change without changing meaning |

Every version gets one entry in `CHANGELOG.md`, and the version in `README.md` moves with it: the date,
the version, the IDs added, changed, or retired, and why.

**Baseline:** `{{command:sprint}} plan` records the version a sprint was planned against. A change during a sprint
bumps the version; it enters the running sprint only by the user's decision.

## Change control

1. **Request** — a GitHub issue labelled `requirement-change`: what should change, and why.
2. **Impact analysis** — the requirements affected, and through `docs/roadmap/` the epics, specs, and
   shipped code that depend on them.
3. **Decision** — the stakeholder approves or rejects it; the decision is written on the issue.
4. **Apply** — `{{command:requirements}}` edits the SRS, bumps the version, writes `CHANGELOG.md`, and
   updates the roadmap's coverage. The PR closes the request issue.

Backlog refinement in `{{command:sprint}}` is where pending change requests get decided.
