# Estimation

Points are relative: 1, 2, 3, 5, 8. A person-day is one person's working day on the sprint. Each sprint
plan reads this file before it estimates, and each close appends its row and its misses.

Sprints 1 to 3 had no estimates when they ran. Their points were estimated on 2026-11-17 from each item's
diff, one person worked each, and their person-days are the working days of the period each sprint's page
records. The Project holds an item's `Estimate` from sprint 4 on: GitHub refuses to edit an archived item,
and the items of sprints 1 to 3 were already archived, except #141 and #163.

## Velocity

| Sprint | Person-days | Planned | Delivered | Carried | Points per person-day | Average of last 3 |
| --- | --- | --- | --- | --- | --- | --- |
| [1](sprints/sprint-01.md) | 5 | 72 | 72 | 0 | 14.4 | 14.4 |
| [2](sprints/sprint-02.md) | 6 | 43 | 43 | 0 | 7.2 | 10.8 |
| [3](sprints/sprint-03.md) | 5 | 36 | 34 | 2 | 6.8 | 9.5 |

Sprint 1 built all of M1 in one week, and sprint 3 finished its plan on its first day and filled the rest
with items pulled in along the way, so the average runs high until sprints with estimates made at planning
replace them.

## Estimates per item, sprints 1 to 3

| Sprint | Items, with points |
| --- | --- |
| 1 | #9 (8), #10 (3), #11 (5), #14 (2), #15 (2), #18 (5), #19 (3), #20 (1), #24 (5), #25 (5), #26 (5), #27 (3), #31 (1), #32 (3), #33 (2), #34 (3), #35 (3), #36 (2), #37 (3), #38 (3), #39 (2), #45 (3) |
| 2 | #55 (3), #56 (1), #57 (1), #59 (5), #60 (2), #65 (5), #66 (2), #67 (3), #68 (3), #69 (3), #70 (3), #71 (2), #72 (3), #73 (2), #74 (2), #75 (3) |
| 3 | #132 (3), #133 (3), #135 (5), #136 (1), #139 (1), #141 (1), #149 (2), #150 (2), #151 (5), #152 (5), #153 (2), #156 (2), #157 (1), #159 (1); carried: #163 (2) |

Epics with sub-issues carry no points: #1 to #8, #58, #140, and #158.

## Estimate misses

Each item whose actual size ran more than half over or under its estimate. Sprints 1 to 3 have none: their
estimates were made from their actual size.

| Sprint | Item | Estimate | Actual | Cause |
| --- | --- | --- | --- | --- |
