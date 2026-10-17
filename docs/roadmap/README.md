# Roadmap

Live status: the [GitHub Project](https://github.com/orgs/ATNexusLab/projects/16) and the
[milestones](https://github.com/ATNexusLab/tightship/milestones).

## M1 — Claude Code

The workflow installs and runs as a native Claude Code plugin, built from the canonical source.
GitHub Milestone: [M1 — Claude Code](https://github.com/ATNexusLab/tightship/milestone/1).

| Epic | Issue | Covers |
| --- | --- | --- |
| Canonical source and build | [#1](https://github.com/ATNexusLab/tightship/issues/1) | NFR-MNT-01, NFR-FLEX-06, NFR-SEC-01, NFR-SEC-02, BR-02, BR-03, FR-SKL-03 |
| Install with skills, commands and verifier | [#2](https://github.com/ATNexusLab/tightship/issues/2) | FR-INS-01, FR-INS-02, FR-SKL-01, FR-SKL-02, FR-VER-01, FR-VER-02, BR-04 |
| Options | [#3](https://github.com/ATNexusLab/tightship/issues/3) | FR-INS-03, FR-INS-04, FR-INS-05, FR-INS-06 |
| Contract | [#4](https://github.com/ATNexusLab/tightship/issues/4) | FR-CTR-01, FR-CTR-02, FR-CTR-03, FR-CTR-04, BR-05 |
| Memory | [#5](https://github.com/ATNexusLab/tightship/issues/5) | FR-MEM-01 to FR-MEM-05, FR-MEM-08 to FR-MEM-11, NFR-REL-01, NFR-PERF-01, NFR-PERF-02 |
| Handoff | [#6](https://github.com/ATNexusLab/tightship/issues/6) | FR-MEM-06, FR-MEM-07 |
| Status line | [#7](https://github.com/ATNexusLab/tightship/issues/7) | FR-STL-01, FR-STL-02, FR-STL-03 |
| Release M1 | [#8](https://github.com/ATNexusLab/tightship/issues/8) | FR-INS-07, FR-INS-08, FR-CTR-05, NFR-FLEX-01, NFR-FLEX-05, BR-01 |

Epic #1's [spec](../specs/canonical-source-and-build/build-the-plugin.md) carries the M1 ADR,
[ADR 0001](../decisions/0001-rust-build-and-tokenized-canonical-content.md), which settles the stack and the
architecture. Options (#3) comes before Contract (#4) because turning the contract off needs the option
mechanism.

## M2 — Cursor

The workflow installs and runs as a native Cursor plugin, in the editor and in the command-line agent.
GitHub Milestone: [M2 — Cursor](https://github.com/ATNexusLab/tightship/milestone/2).
Released in [v1.1.0](https://github.com/ATNexusLab/tightship/releases/tag/v1.1.0) on 2026-10-16.

| Epic | Issue | Covers |
| --- | --- | --- |
| Cursor plugin | [#58](https://github.com/ATNexusLab/tightship/issues/58) | NFR-FLEX-02, NFR-FLEX-07, NFR-COMP-01 |

One epic carries the whole harness, its release included: the canonical content, the build, and the binary
already exist, and the spec settles what Cursor can and cannot do (OQ-04 to OQ-08). The vault location with `~`
(FR-MEM-09.2) is delivered in this milestone, outside any epic, by
[#60](https://github.com/ATNexusLab/tightship/issues/60). It reaches every harness through the `tightship` binary.

## M3 — Antigravity CLI

The workflow installs and runs as a native Antigravity CLI plugin.
GitHub Milestone: [M3 — Antigravity CLI](https://github.com/ATNexusLab/tightship/milestone/3).
Released in [v1.1.0](https://github.com/ATNexusLab/tightship/releases/tag/v1.1.0) on 2026-10-16.

| Epic | Issue | Covers |
| --- | --- | --- |
| Antigravity CLI plugin | [#59](https://github.com/ATNexusLab/tightship/issues/59) | NFR-FLEX-03 |

By [BR-01](../requirements/business-rules.md#br-01), it is built alongside M2 and released after it.

## Requirement coverage

Every approved requirement of M1 to M3 maps to exactly one epic. M4 is planned when its turn comes
([BR-01](../requirements/business-rules.md#br-01)).

| Requirement | Epic | Milestone |
| --- | --- | --- |
| FR-CTR-01 | #4 | M1 |
| FR-CTR-02 | #4 | M1 |
| FR-CTR-03 | #4 | M1 |
| FR-CTR-04 | #4 | M1 |
| FR-CTR-05 | #8 | M1 |
| FR-SKL-01 | #2 | M1 |
| FR-SKL-02 | #2 | M1 |
| FR-SKL-03 | #1 | M1 |
| FR-VER-01 | #2 | M1 |
| FR-VER-02 | #2 | M1 |
| FR-MEM-01 | #5 | M1 |
| FR-MEM-02 | #5 | M1 |
| FR-MEM-03 | #5 | M1 |
| FR-MEM-04 | #5 | M1 |
| FR-MEM-05 | #5 | M1 |
| FR-MEM-06 | #6 | M1 |
| FR-MEM-07 | #6 | M1 |
| FR-MEM-08 | #5 | M1 |
| FR-MEM-09 | #5, and #60 for FR-MEM-09.2 | M1, M2 |
| FR-MEM-10 | #5 | M1 |
| FR-MEM-11 | #5 | M1 |
| FR-STL-01 | #7 | M1 |
| FR-STL-02 | #7 | M1 |
| FR-STL-03 | #7 | M1 |
| FR-INS-01 | #2 | M1 |
| FR-INS-02 | #2 | M1 |
| FR-INS-03 | #3 | M1 |
| FR-INS-04 | #3 | M1 |
| FR-INS-05 | #3 | M1 |
| FR-INS-06 | #3 | M1 |
| FR-INS-07 | #8 | M1 |
| FR-INS-08 | #8 | M1 |
| NFR-FLEX-01 | #8 | M1 |
| NFR-FLEX-02 | #58 | M2 |
| NFR-FLEX-03 | #59 | M3 |
| NFR-FLEX-05 | #8 | M1 |
| NFR-FLEX-06 | #1 | M1 |
| NFR-FLEX-07 | #58 | M2 |
| NFR-COMP-01 | #58 | M2 |
| NFR-SEC-01 | #1 | M1 |
| NFR-SEC-02 | #1 | M1 |
| NFR-MNT-01 | #1 | M1 |
| NFR-REL-01 | #5 | M1 |
| NFR-PERF-01 | #5 | M1 |
| NFR-PERF-02 | #5 | M1 |
| BR-01 | #8 | M1 |
| BR-02 | #1 | M1 |
| BR-03 | #1 | M1 |
| BR-04 | #2 | M1 |
| BR-05 | #4 | M1 |

## Sprints

- [Sprint 1](sprints/sprint-01.md) — M1, from the canonical source to the release on the three systems
- [Sprint 2](sprints/sprint-02.md) — Cursor and Antigravity CLI, the vault location with `~`, a status fix
