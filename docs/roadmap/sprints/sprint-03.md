# Sprint 3 — 2026-10-26 → 2026-11-01

**Goal:** With the contract on, every session holds the workflow's rules from first to last, through compaction
too, and never the rules of an option that is off. With memory on, a session starts with the vault notes that
matter to the project, and the agent searches, writes, lints, and syncs notes in a vault the adopter created.
**Requirements baseline:** v1.1.0
**Items:** [#4](https://github.com/ATNexusLab/tightship/issues/4),
[#5](https://github.com/ATNexusLab/tightship/issues/5)

Neither epic has a spec yet. The sprint opens with `/spec` then `/epic` for #4, then for #5, before any code.
Both read their option by rule 4 of epic #3. The contract's memory rules (FR-CTR-03) come with #4 and point at
the vault that #5 builds. Handoff (#6) builds on #5 and Status line (#7) waits for sprint 4.

FR-MEM-11, syncing the vault, entered the sprint on 2026-10-15 by the maintainer's decision, through change
request [#23](https://github.com/ATNexusLab/tightship/issues/23), as task 4 of #5. The baseline moves to v1.4.0
for #5.

## Result — 2026-10-15

Goal reached. Epic #4 merged into `develop` with [PR #21](https://github.com/ATNexusLab/tightship/pull/21),
and epic #5 with [PR #28](https://github.com/ATNexusLab/tightship/pull/28). They close when the M1 release
reaches `main`.

| Item | Result |
| --- | --- |
| [#4](https://github.com/ATNexusLab/tightship/issues/4) Contract | Done |
| [#18](https://github.com/ATNexusLab/tightship/issues/18) Render the contract into session-start hooks | Done |
| [#19](https://github.com/ATNexusLab/tightship/issues/19) Migrate the baseline's contract | Done |
| [#20](https://github.com/ATNexusLab/tightship/issues/20) Let the contract follow the companions it defers to (FR-CTR-04) | Done |
| [#5](https://github.com/ATNexusLab/tightship/issues/5) Memory | Done |
| [#24](https://github.com/ATNexusLab/tightship/issues/24) Ship the tightship binary with the vault folder and `vault init` | Done |
| [#25](https://github.com/ATNexusLab/tightship/issues/25) Place the vault's notes and checkpoints in the session | Done |
| [#26](https://github.com/ATNexusLab/tightship/issues/26) Write, search, and lint notes | Done |
| [#27](https://github.com/ATNexusLab/tightship/issues/27) Sync the vault | Done |
| [#22](https://github.com/ATNexusLab/tightship/issues/22) Place the compaction checkpoint right after the compaction (FR-MEM-02) | Done |
| [#23](https://github.com/ATNexusLab/tightship/issues/23) Sync the vault with its git remote (FR-MEM-11) | Done |

The sprint ran before its planned start; its dates stay as planned. Change requests #20, #22, and #23 moved
the requirements to v1.4.0 during the sprint, and epic #5 added
[ADR 0002](../../decisions/0002-tightship-binary-modules-and-dependencies.md). The close found the
Closeout step's memory item missing: epic #4's spec left it to epic #5, which never added it. It goes to
epic #6. Change request [#29](https://github.com/ATNexusLab/tightship/issues/29) states in the SRS what the
sprint approved, as v1.4.1. Nothing carries over.
