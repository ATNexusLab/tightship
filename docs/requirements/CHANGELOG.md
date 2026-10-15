# Changelog

## 1.0.2 — 2026-10-14

Editorial. No requirement changes meaning. The spec of epic #1 and [ADR 0001](../decisions/0001-rust-build-and-tokenized-canonical-content.md)
settle four open questions.

**Changed**

- Overview: the adopter prerequisites and the plugin's network and disk reach are stated, from ADR 0001.
- NFR-FLEX-06 names the per-harness places ADR 0001 sets apart.
- Glossary: adds `tightship` and per-harness place.

**Removed**

- OQ-01, the plugin's name: `tightship`.
- OQ-02, the stack: Rust, with tokenized Markdown as the canonical content.
- OQ-03, the plugin's network and disk reach.
- OQ-09, how canonical content names a harness capability: a capability token and the per-harness places.

## 1.0.1 — 2026-10-14

Editorial. No requirement changes meaning.

**Changed**

- Every functional area file embeds its use case diagram from `diagrams/`.
- A blank line now precedes each heading and list, so lists render as lists on the local docs site.

## 1.0.0 — 2026-10-14

Kickoff. First version of the SRS, from the elicitation and analysis held with the maintainer on
2026-10-14 against the baseline `1f41699`. The maintainer approved every requirement in the validation of
the same day, so each one enters with Status `approved`.

**Added**

- Contract: FR-CTR-01 to FR-CTR-05.
- Skills and commands: FR-SKL-01 to FR-SKL-03.
- Verifier: FR-VER-01, FR-VER-02.
- Memory: FR-MEM-01 to FR-MEM-10.
- Status line: FR-STL-01 to FR-STL-03.
- Installation: FR-INS-01 to FR-INS-08.
- Flexibility: NFR-FLEX-01 to NFR-FLEX-07.
- Compatibility: NFR-COMP-01.
- Security: NFR-SEC-01, NFR-SEC-02.
- Maintainability: NFR-MNT-01.
- Reliability: NFR-REL-01.
- Performance efficiency: NFR-PERF-01, NFR-PERF-02.
- Business rules: BR-01 to BR-05.
- Open questions: OQ-01 to OQ-09.
