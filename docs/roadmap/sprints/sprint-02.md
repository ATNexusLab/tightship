# Sprint 2 — 2026-10-16 → 2026-10-25

**Goal:** An adopter installs the workflow as a native plugin in Cursor, in the editor and the command-line
agent, and in Antigravity CLI. They point every harness at the same vault with a location like
`~/ai-memory` on macOS, Linux, and Windows, and the `status_line` option shows and hides the status line in
Claude Code when it is toggled.
**Requirements baseline:** v1.8.0 · change requests decided: [#56](https://github.com/ATNexusLab/tightship/issues/56),
[#57](https://github.com/ATNexusLab/tightship/issues/57)
**Milestones:** [M2 — Cursor](https://github.com/ATNexusLab/tightship/milestone/2), [M3 — Antigravity CLI](https://github.com/ATNexusLab/tightship/milestone/3)

The Project's `Sprint 2` iteration runs from 2026-10-19 to 2026-10-25. The sprint starts three days earlier
because sprint 1 closed early. The iterations stay as they are, since redefining them would clear the
`Sprint` value of the archived items.

## Items, in dependency order

| Item | Why here |
| --- | --- |
| [#56](https://github.com/ATNexusLab/tightship/issues/56), [#57](https://github.com/ATNexusLab/tightship/issues/57) Change requests | Applied in v1.8.0. The PR that plans this sprint closes them. |
| [#60](https://github.com/ATNexusLab/tightship/issues/60) Resolve a vault location that starts with `~` | In the `tightship` binary, so every harness gets it. It comes first because the other two items reuse the vault. |
| [#55](https://github.com/ATNexusLab/tightship/issues/55) The `status_line` option does not show or hide the status line when toggled | Claude Code only. It can run between the specs. |
| [#58](https://github.com/ATNexusLab/tightship/issues/58) Cursor plugin | M2. The epic needs `/spec` then `/epic` before any code. |
| [#59](https://github.com/ATNexusLab/tightship/issues/59) Antigravity CLI plugin | M3. The epic needs `/spec` then `/epic` before any code. It runs alongside #58 by BR-01, and is released after it. |

## Specs before code

- [#58](https://github.com/ATNexusLab/tightship/issues/58): `/spec`, which settles OQ-04 to OQ-08 for Cursor,
  then `/epic`.
- [#59](https://github.com/ATNexusLab/tightship/issues/59): `/spec`, which settles OQ-04, OQ-05, OQ-06, and
  OQ-08 for Antigravity CLI, then `/epic`.

Issues #60 and #55 have no epic and need no spec: each is one delivery on its own branch.
