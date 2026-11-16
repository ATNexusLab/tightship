# Sprint 4 — 2026-11-16 → 2026-11-22

**Goal:** v1.3.0 ships what sprint 3 built, and an adopter installs the workflow as a native Codex plugin,
reads which pieces run on each surface of every harness, Zed's agent panel included, and gets docs that
follow one order of standards, stay in step with the code, serve a client as well as the team, and export
to PDF, with sprints estimated from their own history.
**Requirements baseline:** v4.1.0 · change requests decided: none open; #140, #141, and #149 were decided
in sprint 3 and close with the release
**Milestones:** [M4 — Codex](https://github.com/ATNexusLab/tightship/milestone/4); the other items carry
none until their requirement changes place them

The Project's `Sprint 6` iteration runs from 2026-11-16 to 2026-11-22. The sprint takes the whole backlog:
sprint 3's plan held three days of work, finished on its first day, and the rest of the week was filled
with items pulled in along the way.

## Items, in dependency order

| Item | Why here |
| --- | --- |
| Release v1.3.0 | Homologates `develop` and ships sprint 3, which moves its items from Staging to Done. No issue: the `release` skill carries it. |
| [#163](https://github.com/ATNexusLab/tightship/issues/163) Check every harness in Zed's agent panel | Carried from sprint 3 through the Backlog. Under the Zed spec and NFR-FLEX-08 already, so it needs no spec. |
| [#137](https://github.com/ATNexusLab/tightship/issues/137) Set an order of precedence for documentation standards | An ADR, and every doc type names its standard. First of the docs items, since #138 and #162 depend on it. |
| [#138](https://github.com/ATNexusLab/tightship/issues/138) Let bootstrap write project docs, client presentation docs, or both | After #137, which picks the standard the client docs follow. |
| [#162](https://github.com/ATNexusLab/tightship/issues/162) Export the docs, whole or by page | After #137. Builds on the HTML Zensical already generates. |
| [#160](https://github.com/ATNexusLab/tightship/issues/160) Keep the docs in step with the code | A `tightship docs check` in the binary, run by `closeout`, `sprint close`, and CI. Last of the docs items, so it checks what they wrote. |
| [#161](https://github.com/ATNexusLab/tightship/issues/161) Estimate sprints from their own history | Changes `sprint` itself and adds an `Estimate` field to the Project. Independent of the rest. |
| [#142](https://github.com/ATNexusLab/tightship/issues/142) Codex plugin | M4. The spec settles OQ-04, OQ-06, and OQ-08 for Codex. |
| [#134](https://github.com/ATNexusLab/tightship/issues/134) Support every surface of each harness | After #142, since its matrix has rows for Codex's surfaces, and after #163, which fills the Zed rows. |

## Specs before code

- [#142](https://github.com/ATNexusLab/tightship/issues/142): `/spec`, then `/epic` on the existing issue.
- [#134](https://github.com/ATNexusLab/tightship/issues/134): `/requirements` to bring the support matrix
  into the SRS, then `/spec` and `/epic` on the existing issue.
- #138, #160, #161, and #162: each adds behavior the SRS does not hold, so each goes through `/requirements`
  first, which also maps it to a milestone and an epic in the roadmap.

Issues #163 and #137 are one delivery each on their own branch, with no spec.
