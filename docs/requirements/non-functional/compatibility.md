# Compatibility — `COMP`

How the product coexists with itself across harnesses.

<a id="nfr-comp-01"></a>
## NFR-COMP-01 — One vault across harnesses

The system shall use one vault, in one format, from every supported harness on the same machine.

| Attribute | Value |
| --- | --- |
| Rationale | A handoff exists to switch harness mid-task, so a note written in one has to be read in another. |
| Source | Baseline: the handoff command and the vault's rules; confirmed by the maintainer in validation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M2 |
| Since | v1.0.0 |

**Acceptance criteria**
- **NFR-COMP-01.1** — A note or handoff written from one supported harness is found by search and shown
  at session start from each of the others, with 0 conversion steps.
