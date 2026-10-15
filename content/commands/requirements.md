+++
name = "requirements"
description = "Elicit, change, or validate the project's requirements (docs/requirements/, a versioned SRS)"
argument-hint = "[a change to make, a change-request issue number, or nothing for kickoff]"
+++

Run requirements engineering on `docs/requirements/` for: {{capability:arguments}}

The method — phases, SRS template, requirement IDs and attributes, the quality checklist, versioning, and
change control — is the `{{skill:requirements-engineering}}` skill. Follow it; this command only picks the path.

## Kickoff — `docs/requirements/` does not exist

1. **Elicitation.** Read the repo and any existing docs first, then interview me for what they cannot
   answer — problem, stakeholders, scope, needs, constraints. Never invent an answer; an undecided item
   goes to Open questions.
2. **Analysis.** Classify and prioritise what came out, and show me the conflicts, duplicates, and
   dependencies you found. We settle them before anything is written.
3. **Specification.** Write the SRS from the skill's template at version `1.0.0`, every requirement with
   Status `proposed`.
4. **Validation.** Run the quality checklist on every requirement and on the set, fix what fails, then
   walk me through it. What I confirm becomes `approved`.

## Change — the SRS exists

1. **Request.** The change comes from `{{capability:arguments}}` or a `requirement-change` issue. Without an issue,
   open one first — every change is traceable.
2. **Impact analysis.** Name the requirements it adds, changes, or retires, and — through
   `docs/roadmap/` — the epics, specs, and shipped code that depend on them. Show me before editing.
3. **Apply.** With my approval: edit the requirements (a changed one keeps its ID; a new one takes the
   next free ID in its group; a removed one is `retired`, never deleted), bump the version, write the
   `CHANGELOG.md` entry, and update the roadmap's coverage table.
4. **Validation.** The quality checklist on every requirement touched, and the set checks.

## Relationship to the other docs

- A `{{command:spec}}` details one or more requirements and cites their IDs. When it settles an Open question,
  the question is deleted; question numbers are never reused.
- A feature specced elsewhere is referenced here, never restated. One source per fact.

## Rules

- This is product documentation — it lives in the repo, not in memory.
- Do not write code from this command.

## Output

```text
Requirements:
- Version:   [old → new, and why that bump]
- Changed:   [IDs added / changed / retired]
- Impact:    [epics, specs, code affected / none]
- Checklist: [passed / what failed and was fixed]
- Open:      [questions still open / none]
- Request:   [#issue / kickoff]
```
