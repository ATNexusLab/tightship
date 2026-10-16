# Sprint 1 — 2026-10-12 → 2026-10-18

**Goal:** M1. The maintainer builds the Claude Code plugin from one canonical source. An adopter installs it
with its companions, chooses its options, keeps the contract and the vault in every session, hands a session
off to another machine, turns on the status line, and installs the release on macOS, Linux, and Windows.
**Requirements baseline:** v1.0.1 at the start, v1.4.1 for epic #8
**Items:** [#1](https://github.com/ATNexusLab/tightship/issues/1) to
[#8](https://github.com/ATNexusLab/tightship/issues/8)

M1 was planned as four sprints, two epics each, and ran whole inside this one. The four plans are its stages
below, in the order they ran.

## Stage 1 — the canonical source builds the Claude Code plugin

**Goal:** The maintainer edits one canonical file and the build produces the Claude Code plugin, with the
identity and secret scans passing.
**Items:** [#1](https://github.com/ATNexusLab/tightship/issues/1)

Epic #1 had no spec yet. The stage opened with `/spec` for it, carrying the M1 ADR, then `/epic` to hang its
sub-issues on #1, before any code.

### Stage 1 result — 2026-10-14

Goal reached. Epic #1 and its three sub-issues merged into `develop` with
[PR #12](https://github.com/ATNexusLab/tightship/pull/12); they close when the M1 release reaches `main`.

| Item | Result |
| --- | --- |
| [#1](https://github.com/ATNexusLab/tightship/issues/1) Canonical source and build | Done |
| [#9](https://github.com/ATNexusLab/tightship/issues/9) Build a canonical piece into the Claude Code plugin | Done |
| [#10](https://github.com/ATNexusLab/tightship/issues/10) Scan for identity, secrets, and vault notes | Done |
| [#11](https://github.com/ATNexusLab/tightship/issues/11) Migrate the 22 baseline pieces | Done |

Its spec settled four open questions, recorded in requirements v1.0.2.

## Stage 2 — the adopter installs the plugin and chooses its options

**Goal:** An adopter installs the plugin from this repository's catalog, gets the skills, the commands, the
verifier, and the companions, and chooses which options are on.
**Requirements baseline:** v1.0.2
**Items:** [#2](https://github.com/ATNexusLab/tightship/issues/2),
[#3](https://github.com/ATNexusLab/tightship/issues/3)

Neither epic had a spec yet. The stage opened with `/spec` then `/epic` for #2, then for #3, before any code.
Options (#3) builds on the installed plugin of #2.

### Stage 2 result — 2026-10-15

Goal reached. Epic #2 merged into `develop` with [PR #16](https://github.com/ATNexusLab/tightship/pull/16),
and epic #3 with [PR #17](https://github.com/ATNexusLab/tightship/pull/17). They close when the M1 release
reaches `main`.

| Item | Result |
| --- | --- |
| [#2](https://github.com/ATNexusLab/tightship/issues/2) Install with skills, commands and verifier | Done |
| [#14](https://github.com/ATNexusLab/tightship/issues/14) Install the plugin with its companions from the built catalog | Done |
| [#13](https://github.com/ATNexusLab/tightship/issues/13) Let the verifier run tests in a private copy (FR-VER-02) | Done |
| [#3](https://github.com/ATNexusLab/tightship/issues/3) Options | Done |
| [#15](https://github.com/ATNexusLab/tightship/issues/15) Declare the options and let the harness keep them | Done |

Change request #13 moved the requirements to v1.1.0 during the stage, and epic #3's spec amended ADR 0001: the
options' state belongs to the harness, not to the plugin.

## Stage 3 — the contract in every session, and the memory layer

**Goal:** With the contract on, every session holds the workflow's rules from first to last, through compaction
too, and never the rules of an option that is off. With memory on, a session starts with the vault notes that
matter to the project, and the agent searches, writes, lints, and syncs notes in a vault the adopter created.
**Requirements baseline:** v1.1.0
**Items:** [#4](https://github.com/ATNexusLab/tightship/issues/4),
[#5](https://github.com/ATNexusLab/tightship/issues/5)

Neither epic had a spec yet. The stage opened with `/spec` then `/epic` for #4, then for #5, before any code.
Both read their option by rule 4 of epic #3. The contract's memory rules (FR-CTR-03) came with #4 and point at
the vault that #5 builds.

FR-MEM-11, syncing the vault, entered the stage on 2026-10-15 by the maintainer's decision, through change
request [#23](https://github.com/ATNexusLab/tightship/issues/23), as task 4 of #5. The baseline moved to v1.4.0
for #5.

### Stage 3 result — 2026-10-15

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

Change requests #20, #22, and #23 moved the requirements to v1.4.0 during the stage, and epic #5 added
[ADR 0002](../../decisions/0002-tightship-binary-modules-and-dependencies.md). Its close found the Closeout
step's memory item missing: epic #4's spec left it to epic #5, which never added it. It went to epic #6.
Change request [#29](https://github.com/ATNexusLab/tightship/issues/29) states in the SRS what the stage
approved, as v1.4.1.

## Stage 4 — handoff, status line, and the M1 release

**Goal:** An adopter hands a session off and claims it on another machine, turns on the status line after
consenting, and installs M1 by following the install guide on macOS, Linux, and Windows.
**Requirements baseline:** v1.4.1
**Items:** [#6](https://github.com/ATNexusLab/tightship/issues/6),
[#7](https://github.com/ATNexusLab/tightship/issues/7),
[#8](https://github.com/ATNexusLab/tightship/issues/8)

None of the three epics had a spec yet. The stage opened with `/spec` then `/epic` for #6, then #7, then #8,
before any code. Handoff (#6) builds on the vault of #5 and also restores the Closeout step's memory item.
Status line (#7) adds its command to the `tightship` binary. Release M1 (#8) comes last: it builds the binary
for every platform and releases what the other epics shipped.

### Stage 4 result — 2026-10-15

Goal reached. Epic #6 merged into `develop` with
[PR #40](https://github.com/ATNexusLab/tightship/pull/40), epic #7 with
[PR #42](https://github.com/ATNexusLab/tightship/pull/42), and epic #8 with
[PR #43](https://github.com/ATNexusLab/tightship/pull/43) and
[PR #46](https://github.com/ATNexusLab/tightship/pull/46).
[PR #47](https://github.com/ATNexusLab/tightship/pull/47) released M1 as the Claude Code plugin 1.0.0, which closed
every issue below.

| Item | Result |
| --- | --- |
| [#6](https://github.com/ATNexusLab/tightship/issues/6) Handoff | Done |
| [#32](https://github.com/ATNexusLab/tightship/issues/32) Release and claim the handoff in the binary | Done |
| [#33](https://github.com/ATNexusLab/tightship/issues/33) Ship the handoff command, the option token, and the closeout's memory item | Done |
| [#7](https://github.com/ATNexusLab/tightship/issues/7) Status line | Done |
| [#34](https://github.com/ATNexusLab/tightship/issues/34) Render the baseline's status line | Done |
| [#35](https://github.com/ATNexusLab/tightship/issues/35) Ask, turn on, and put back the status line | Done |
| [#8](https://github.com/ATNexusLab/tightship/issues/8) Release M1 | Done |
| [#36](https://github.com/ATNexusLab/tightship/issues/36) Run the gates on macOS, Linux, and Windows at every pull request | Done |
| [#37](https://github.com/ATNexusLab/tightship/issues/37) Publish a pre-release from a tag | Done |
| [#38](https://github.com/ATNexusLab/tightship/issues/38) Write the Claude Code install guide | Done |
| [#39](https://github.com/ATNexusLab/tightship/issues/39) Run the release checklist on the three systems | Done |
| [#31](https://github.com/ATNexusLab/tightship/issues/31) Let the uninstall turn the status line off first (FR-INS-06.1) | Done |
| [#45](https://github.com/ATNexusLab/tightship/issues/45) Make the repository ready for outside adopters and contributors | Done |

Change requests #30, #31, and #44, and the maintainer's decision on Portuguese translations, moved the
requirements to v1.7.0 during the stage. After the release, the fixes #48 and #49 shipped as 1.0.1 and the
status line's automatic turn-on as 1.0.2.

## Sprint result — 2026-10-15

Goal reached, three days early: M1 is released, and its GitHub Milestone is closed. Nothing carries over.
The sprint closed on 2026-10-15 by the maintainer's decision, and sprint 2 starts the next day.

Two changes came out of the close, both approved for sprint 2 by the maintainer. Cursor and Antigravity CLI
are built side by side, which changes BR-01
([#56](https://github.com/ATNexusLab/tightship/issues/56)). A vault location may name the home folder with `~`,
which adds FR-MEM-09.2 ([#57](https://github.com/ATNexusLab/tightship/issues/57)). The requirements moved to
v1.8.0. The status line bug [#55](https://github.com/ATNexusLab/tightship/issues/55), found after 1.0.2, joins
sprint 2.
