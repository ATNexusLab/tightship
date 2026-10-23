+++
name = "technical-writing"
description = "Use when writing or updating a README, a page under docs/, or a runbook — whether the doc should exist, where each fact belongs, and how it stays true as the code moves."
+++

# Technical Writing

A doc is a claim about the code, and it decays the moment the code moves. Write the fewest docs that
answer real questions, put each fact in exactly one place, and prefer the form that cannot go stale.

**Scope.** This skill names criteria, never tools. A repo on this workflow takes its docs structure,
diagrams, and site from `{{skill:project-docs}}`; any other repo takes them from its own `AGENTS.md`.
Project docs live in the **repo** — never only in memory.

## Before writing: should this doc exist?

Every doc is another surface that drifts. It earns its place only when it answers a question that
**nothing else already answers** and someone will actually ask.

Already answered — do not write it down:

- What the code says. A reader who needs the exact behavior reads the code, and prose about it will
  disagree with it eventually.
- What a tool prints. Command-line help, a generated schema, a type definition, a config template.
- What the history holds. What changed and when is in the commits, the issues, and the pull requests —
  a file that restates them is a second, worse copy that nobody updates.

Write it when the answer lives only in someone's head: how to get this running, why the pieces fit the
way they do, and what to do when it breaks.

## Where the fact lives

One destination per fact. Most duplication starts as honest uncertainty about which of these it was.

| The fact | Where it lives |
| --- | --- |
| How to run, use, or operate this | `README` / `docs/` — here |
| What the product must be able to do | `docs/requirements/`, the versioned SRS — its `CHANGELOG.md` is the one change log a file keeps |
| What the system is made of, and its data | `docs/architecture/`, `docs/data-model/` |
| What is planned, per milestone and sprint | `docs/roadmap/` — status lives on GitHub |
| How one feature behaves, in detail | that feature's spec |
| Why it was decided this way | the ADR |
| How the agent works in this repo | the repo's own `AGENTS.md` |
| What changed, when, and by whom | git, issues, and pull requests — never a file |

The global docs follow `{{skill:project-docs}}` and the SRS `{{skill:requirements-engineering}}`;
continue in `{{skill:spec-writing}}` for a spec or an ADR, and in `{{command:bootstrap}}` for the repo's
`AGENTS.md`. A doc here **links** to them and never restates them.

## Staying true

Accuracy on the day you write is easy; the failure mode is the Friday after. Prefer the form that
survives:

- **Point instead of restating.** A link to the file or the command stays right when it changes; a
  paraphrase does not.
- **Never copy what a tool generates.** Two copies of the same output means one of them is wrong and
  the reader cannot tell which.
- **Date and scope what is inherently a snapshot.** A benchmark, a screenshot, a version list, a known
  limitation — say when it was true, so a reader can judge it instead of trusting it.
- **A runnable example is the only one that survives.** If it can be executed, run it; if it cannot,
  it will be wrong and nobody will notice.
- **A doc that contradicts the code is a finding.** One of the two is wrong — decide which before
  touching the prose, because rewording a doc to match a bug documents the bug.

## Principles

- **Accurate over complete.** A wrong doc is worse than no doc.
- **Reader-first.** Lead with what they need to *do*. "How do I run, use, or change this?" comes before
  any explanation of how it works.
- **Scannable.** Short sections, headings, lists, runnable blocks. No walls of text.
- **Show, don't assert.** A concrete command beats an adjective.
- **The repo's languages, as its `AGENTS.md` states them** — the doc language for the prose, the code
  language for identifiers, paths, and file names.
- **A reusable doc or skill uses generic placeholders**, never names from the project it was written in,
  and names no OS, machine, or absolute path it does not document.

## The three artifacts

Each answers one question nothing else answers. There is no fourth by default.

- **README** — *"how do I run and use this?"* What it is · quick start, copy-pasteable, from clone to
  running · the commands that matter · where to go next. The one doc that must always be current.
- **`docs/`** — the depth that would bury the README. One page per subject, linked from it.
- **Runbook** — *"it is broken at 3am, what do I do?"* Deploy, roll back, diagnose, escalate. Only
  where something is actually operated.

## Method

1. Name the reader and the one job the doc serves.
2. Write the smallest doc that does that job.
3. Run every command and example in it.

## Never do

- Document an aspiration as if it exists.
- Restate what the code, a tool, or the history already answers.
- Bury the action the reader needs under background.
- Reword a doc to match the code without asking which of the two is wrong.
