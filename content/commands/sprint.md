+++
name = "sprint"
description = "Plan a sprint (building the roadmap on first run) or close the current one"
argument-hint = "plan | close"
+++

Run the sprint step for this repo: {{capability:arguments}}

The backlog lives in the repo's GitHub Project (created by `{{command:bootstrap}}`); the plan and its
history live in `docs/work/roadmap/`, written with the `{{skill:project-docs}}` template. Status is never
mirrored into the docs — the Project and the GitHub Milestones own it.

Status runs Backlog · Todo · In Progress · In Review · Staging · Done, and the `Sprint` iteration field says
when. Staging is merged into the integration branch; the release to `main` closes the issue, which moves it
to Done and archives it. Only the current sprint is planned in detail; the next one is planned when it
closes.

Check what this needs before starting: a Project with the six-option Status, a `Sprint` iteration
field, and an `Estimate` number field (`{{command:bootstrap}}`), and a `docs/product/requirements/` SRS
with requirement IDs (`{{command:requirements}}`). One missing → say so and ask whether to go on, before
spending work on it.

## `plan`

### 1. Roadmap — only when `docs/work/roadmap/` is missing, or a milestone has no epics yet

- Split the next unplanned milestone of `docs/product/requirements/` into epics. An epic is one
  capability a user can observe, sized to be specced as one `{{command:spec}}`.
- Every approved requirement of the milestone maps to exactly one epic. One without an epic, or with
  two, is fixed before anything is written.
- Run a `{{skill:grilling}}` session over the split: the epics, their order, and what each covers. Nothing is
  created before it closes.
- Create the milestone as a GitHub Milestone `M<N> — <name>` if it does not exist
  (`gh api repos/<owner>/<repo>/milestones -f title=... -f description=...`).
- Create each epic as an issue: label `epic` and its `area:*`, the milestone's GitHub Milestone, added to
  the Project in Backlog. Body:

  ```markdown
  <One sentence: what the user can do when this epic is done.>

  **Covers:** FR-<AREA>-NN, NFR-<CHAR>-NN
  **Spec:** pending — written when the epic enters a sprint.
  ```

- Write `docs/work/roadmap/README.md`: the milestone with its GitHub Milestone link, its epics with issue
  numbers, and the requirement coverage table.

### 2. Refinement

Every open `requirement-change` issue gets decided before the sprint is picked: approved ones go through
`{{command:requirements}}`, rejected ones are closed with the reason. The sprint then plans against the resulting
version of `docs/product/requirements/` — its **baseline**.

### 3. The sprint

- Pick the Sprint Goal: one sentence a user could observe when it ends. Then the items that reach it,
  in dependency order: an epic whole, or the part of one that the Sprint Goal needs.
- Estimate each item from `docs/work/roadmap/estimation.md` and the items that already closed: propose 1,
  2, 3, 5, or 8 points, citing the closed item it resembles and its `Estimate`. An epic with sub-issues
  gets no estimate; its sub-issues carry it, so nothing counts twice.
- Ask each person assigned to the sprint how many days they have for it. The cap is the average points per
  person-day of the last three sprints in `estimation.md`, or of those there are, times the person-days.
  With no sprint there yet, say the plan has no cap. Items past the cap are cut, or kept by my explicit
  decision, which the sprint's plan records with its reason.
- An item whose epic has no closed spec gets one before any code — list those specs as the sprint's
  first work, each a `{{command:spec}}` then an `{{command:epic}}` that hangs the tasks on the existing epic issue.
- Agree the Sprint Goal, the items, and their estimates with me. Then set each item's `Sprint` to the
  iteration, its `Estimate` (`gh project item-edit --number`), and its Status to Todo, write
  `docs/work/roadmap/sprints/sprint-NN.md` with its requirements baseline and its estimate, and link it
  from the roadmap's `README.md`.

Only this sprint is planned in detail. The next one is planned when this one closes.

## `close`

1. List the sprint's items from the Project: done — Staging or Done — and not done. An item in Staging
   is done for the sprint; the release closes it.
2. For each item not done: why, and whether it carries over to the next sprint or returns to Backlog —
   I decide.
3. Write the dated result into `docs/work/roadmap/sprints/sprint-NN.md`.
   Every issue of a milestone closed → close its GitHub Milestone.
4. Measure the estimates. Sum the points planned, delivered, and carried. For each item, propose its
   actual size in points from its diff and its days in progress, and I confirm it. Each item whose actual
   size is more than half over or under its estimate gets one line with its cause: scope grew, a
   dependency, underestimated, or overestimated. Append the sprint's row and those lines to
   `docs/work/roadmap/estimation.md`, with the average of the last three sprints recomputed.
5. Run `tightship docs check`, and read each doc under `docs/system/` against the code its `covers` lists:
   does it still hold? Each finding, and each doc that no longer holds, is proposed as an item of the next
   sprint; it becomes one at my word.
6. A requirement or architecture change the sprint revealed → `{{command:requirements}}` or an ADR, named here.
7. Then run `plan` for the next sprint.

## Output

```text
Sprint <N>:
- Sprint Goal: [one sentence]
- Baseline:    [docs/product/requirements/ v<version> · change requests decided: #… / none]
- Items:       [#n title — Status · points], in dependency order
- Estimate:    [points planned · capacity <person-days> · cap <points> / no cap yet · within / over, by decision]
- Velocity:    [last sprint: planned · delivered · carried · average of last 3 · misses recorded: #… / none]
- Specs:       [needed before code / all closed]
- Roadmap:     [docs/work/roadmap/ updated — what changed]
- Carried:     [from the previous sprint / none]
- Docs drift:  [findings proposed as items: #… / none]
```

Every doc change goes through the normal commit approval.
