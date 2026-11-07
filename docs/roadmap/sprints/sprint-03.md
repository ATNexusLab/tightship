# Sprint 3 — 2026-11-09 → 2026-11-15

**Goal:** An adopter installs from any shell on Windows, Linux, or macOS, reads everything the installer
prints, and ends with memory and the status line working with no manual step; the status line keeps
updating while the session is idle.
**Requirements baseline:** v3.0.0 · change requests decided:
[#141](https://github.com/ATNexusLab/tightship/issues/141), approved;
[#140](https://github.com/ATNexusLab/tightship/issues/140) waits for its audit, an item of this sprint
**Milestones:** none; the items fix and extend what M1 to M3 shipped

The Project's `Sprint 5` iteration runs from 2026-11-09 to 2026-11-15. Sprints 3 and 4 of the Project ran
without a plan, so the iteration's number is ahead of the sprint's.

## Items, in dependency order

| Item | Why here |
| --- | --- |
| [#132](https://github.com/ATNexusLab/tightship/issues/132) Default the vault to `~/ai-memory` and apply Claude Code's status line at install | Both gaps leave the adopter a step to do after the installer, against FR-INS-09.1. |
| [#135](https://github.com/ATNexusLab/tightship/issues/135) Print text that reads correctly in any shell | The largest item: both installers, their tests per shell, the release checklist, and `docs/install/` with its pt-BR pages. It settles ASCII or UTF-8 with a fallback for the binary and the status line. After #132, which changes the same prompts. |
| [#133](https://github.com/ATNexusLab/tightship/issues/133) Keep the 7d gauge and refresh the status line while idle | The binary's status line; after #135, which may change the characters it prints. |
| [#139](https://github.com/ATNexusLab/tightship/issues/139) Configure the editor extensions bootstrap recommends | `content/` only. |
| [#136](https://github.com/ATNexusLab/tightship/issues/136) Keep only the physical data model diagram | `content/` only. |
| [#140](https://github.com/ATNexusLab/tightship/issues/140) Audit the companion plugins | The audit decides the change request: approved, it moves BR-04 and BR-05 through `/requirements`; rejected, it is closed with the reason. It also answers OQ-05, which the Codex spec needs. |

## Specs before code

None. No item is an epic: each is one delivery on its own branch. The Codex epic of M4 waits for #140.
