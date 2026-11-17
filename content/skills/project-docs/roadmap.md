# The roadmap folder

The templates of `docs/work/roadmap/`, which holds the plan and its history; status stays on GitHub.

`README.md`:

```markdown
# Roadmap

Live status: the GitHub Project (<url>) and the milestones (<repo url>/milestones).

**Product Goal:** <the future state of the product the milestones move toward>

## M<N> — <name>
<Goal in one sentence.> GitHub Milestone: [M<N>](<milestone url>).

| Epic | Issue | Covers |
| --- | --- | --- |
| <epic name> | #<n> | FR-<AREA>-01, NFR-<CHAR>-02 |

## Requirement coverage
Every approved requirement of a planned milestone maps to exactly one epic.

| Requirement | Epic | Milestone |
| --- | --- | --- |

## Sprints
Velocity and estimate misses: [estimation.md](estimation.md).

- [Sprint <N>](sprints/sprint-NN.md) — <goal>
```

`sprints/sprint-NN.md`:

```markdown
# Sprint <N> — <start> → <end>

**Sprint Goal:** <one sentence a user could observe at the end>
**Definition of Done:** <the repo's gates, as its `AGENTS.md` or `gates` skill lists them>
**Requirements baseline:** v<version>
**Items:** #<n>, #<n>
**Estimate:** <points> points · capacity <person-days> person-days · cap <points> — <within the cap / over
the cap, by <name>'s decision: <reason>>

## Result — <close date>
Done: #… · Carried over: #… → Sprint <N+1>, because <reason> · Back to backlog: #…
Points: <planned> planned · <delivered> delivered · <carried> carried
```

`estimation.md`, read by every sprint plan and appended to at every close. A sprint's points are the sum of
its items' `Estimate`; an epic with sub-issues carries none, so nothing counts twice.

```markdown
# Estimation

Points are relative: 1, 2, 3, 5, 8. A person-day is one person's working day on the sprint.

## Velocity

| Sprint | Person-days | Planned | Delivered | Carried | Points per person-day | Average of last 3 |
| --- | --- | --- | --- | --- | --- | --- |
| <N> | <days> | <points> | <points> | <points> | <delivered ÷ person-days> | <mean of this and the two before> |

## Estimate misses

Each item whose actual size ran more than half over or under its estimate.

| Sprint | Item | Estimate | Actual | Cause |
| --- | --- | --- | --- | --- |
| <N> | #<n> <title> | <points> | <points> | <scope grew / a dependency / underestimated / overestimated>: <one line> |
```
