+++
name = "epic"
description = "Turn a closed spec into a GitHub epic issue with native sub-issues"
argument-hint = "[path to the closed spec]"
+++

Publish the spec at {{capability:arguments}} as a GitHub epic plus one sub-issue per task.

This command does **not** design. If the spec is not closed, it stops — writing the missing detail is
`{{command:spec}}`'s job, not something to improvise at publish time.

## 1. Gate — refuse an open spec

Read the spec and check, in this order. The spec is in the repo's doc language, so each section and
status is recognized by its role in `{{skill:spec-writing}}`'s `templates.md`, never by a literal string. Any
failure stops the command with the exact reason:

- The header's status is `closed`.
- No placeholder or hedge in the spec's language — `TBD`, `???`, "to be defined", "maybe" and their
  translations — and no question mark in a declarative sentence.
- The Task Breakdown exists, is non-empty, and every row has a title, a scope, and an acceptance
  criterion.
- Every Depends on value names a task number that exists in the table.
- Every Gherkin scenario in the spec is claimed by at least one task.
- Every task whose scope deletes or moves something states the count of its consumers.

Report which check failed and what to fix. Do not partially publish.

## 2. Discover the repo's conventions

- `gh repo view` — confirm the target repo, and confirm I meant this one.
- `gh label list` — reuse the labels that exist. Apply `epic` to the parent if the repo has it; map each
  task to the repo's `area:*` / type labels. Never create a label without asking.
- The epic issue already exists — `{{command:sprint}} plan` created it from the roadmap. Find it on
  `docs/roadmap/` and reuse it: its body gains the spec path and the scope below. Create one only when
  the repo has no roadmap, and say so.
- The repo's GitHub Project, its `Sprint` iteration field, and the current iteration.

## 3. Build the bodies

Both bodies are written in the repo's doc language, headings included; the templates below name their
parts in English. When the repo already has epics, their wording of each heading is the one to reuse.

A title is only the name of the work. Hierarchy is the native parent and grouping is the label, so no
title carries a prefix like `[EPIC]` that repeats them — even where older issues in the repo do.

**Epic (parent):**

```markdown
<One sentence: what the feature delivers and to whom.>

**Spec:** `<path of the file in the repo>`

## Scope
<The 3–6 behaviors the spec closes.>

## Out of scope
<Copied from the spec's section.>

## Tasks
<Left empty — GitHub's native sub-issues fill this list on their own.>
```

**Each sub-issue** is one delivery — reviewed in one sitting, never a micro-task.
Its sections mirror what the epic's PR will ask for, so the PR body is already written by the time the
code is:

```markdown
## Context
<The spec's business rule(s) this task implements, with their number.> — spec: `<path>`

## Scope
<What goes in. Files/layer the task touches.>

## Acceptance criterion
<The spec's Gherkin scenario(s), copied in full.>

## Possible impacts
<What else in the system this change touches.>

## How to test
1. <step>
2. <step>
```

## 4. Show me everything, then wait

Print the epic body and every sub-issue body in full, plus the labels each will get. Wait for my approval.
Nothing is created before I say so — approving the plan is not approving the publish.

## 5. Publish

1. Update the existing epic's body: `gh issue edit <epic-number> --body-file ...`.
2. For each task, **in dependency order**:
   `gh issue create --title ... --body-file ... --parent <epic-number> --label ... --project "<project title>"`
   `--parent` creates a real GitHub sub-issue — do not fake the link with a checklist.
3. Set each task's `Sprint` to the current iteration and its Status to Todo:
   `gh project item-edit --id <item-id> --project-id <project-id> --field-id <field-id> --iteration-id <iteration-id>`,
   and `--single-select-option-id` for Status.
4. If a `gh` call fails, stop immediately and report which issues already exist. Never retry blindly and
   never leave duplicates behind.

## 6. Close the loop

Update the spec in place: the status to `published`, `Epic: #N`, and one column in the Task Breakdown carrying
each task's issue number. Tell me what changed and get approval before committing it — the same rule as
any other commit.

Then print the epic URL and the task numbers in dependency order. Branch, commits and PR follow the
contract's **Git** section.
