# Changelog

## 1.8.0 — 2026-10-15

Two changes the maintainer approved in the planning of sprint 2. Cursor and Antigravity CLI are built in the
same sprint, by change request [#56](https://github.com/ATNexusLab/tightship/issues/56). A vault location can
name the home folder with `~`, by change request [#57](https://github.com/ATNexusLab/tightship/issues/57).

**Added**

- FR-MEM-09.2: a vault location that is `~` alone or starts with `~` and a path separator resolves under the
  user's home folder, on macOS, Linux, and Windows.

**Changed**

- BR-01: work on a harness may run alongside work on the one before it. The releases keep their order.

## 1.7.0 — 2026-10-15

The documents an adopter or a contributor reads first get a Brazilian Portuguese translation, decided by the
maintainer on 2026-10-15. English stays the version that prevails.

**Changed**

- BR-03: the root `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, and the install
  guides also have a `<file>.pt-BR.md` translation, updated in the same change as the English.

## 1.6.1 — 2026-10-15

Editorial. No requirement changes meaning. The repository takes pull requests from outside contributors.
Change request [#44](https://github.com/ATNexusLab/tightship/issues/44).

**Changed**

- Overview: the maintainer reviews and merges every change, and a Contributor row joins the stakeholders.

## 1.6.0 — 2026-10-15

FR-INS-06.1 could not hold once the status line is on: Claude Code lets no plugin set `statusLine`, runs no
plugin code on uninstall, and deletes the plugin's data folder, which holds the adopter's previous status
line. Change request [#31](https://github.com/ATNexusLab/tightship/issues/31).

**Changed**

- FR-INS-06 and FR-INS-06.1 hold for an uninstall that follows the install guide's steps, which turn
  `status_line` off and open one session first.

## 1.5.1 — 2026-10-15

Editorial. The [spec of epic #8](../specs/release-m1/release-the-plugin.md) settles OQ-04 for Claude Code.

**Changed**

- NFR-FLEX-01 and NFR-FLEX-01.1 name Claude Code 2.1.295 as the minimum supported version, with the reason
  in the Rationale.
- OQ-04 asks only for the minimum versions of Cursor, Antigravity CLI, and Codex, and affects NFR-FLEX-02 to
  NFR-FLEX-04.
- Overview: the harness versions table states Claude Code's minimum version.

## 1.5.0 — 2026-10-15

BR-05 could not be checked: it named no version of the companions, whose rules change between versions, and
it had no acceptance criterion. Change request [#30](https://github.com/ATNexusLab/tightship/issues/30).

**Added**

- BR-05.1 compares each rule of the shipped contract and pieces with the companions at the versions the
  plugin's manifest declares.

**Changed**

- BR-05 holds against the version of each companion the plugin installs.

## 1.4.1 — 2026-10-15

Editorial. No requirement changes meaning: the SRS now states what sprint 3 approved and shipped. Change
request [#29](https://github.com/ATNexusLab/tightship/issues/29).

**Changed**

- FR-CTR-04 and FR-MEM-02 record the versions that changed them, and FR-CTR-04.1 and FR-MEM-02.1 cite the
  epic #4 and epic #5 specs instead of "the spec of M1".
- FR-MEM-11's statement gives a new id to a local note whose id the remote already took, as FR-MEM-11.3
  checks. Its Source cites #23.
- Glossary: Note and Vault folder.

## 1.4.0 — 2026-10-15

Syncing the vault across machines took a request in free text, and two machines that wrote before syncing gave
two notes the same id. Change request [#23](https://github.com/ATNexusLab/tightship/issues/23), approved
during the implementation of epic #5, which takes it into sprint 3.

**Added**

- FR-MEM-11 syncs the vault with its git remote at the adopter's request. FR-MEM-11.1 checks that notes from
  both sides end on both, FR-MEM-11.2 that a conflict leaves the remote unchanged and is handed to the agent,
  and FR-MEM-11.3 that a local note whose id the remote already took gets a new one.

## 1.3.0 — 2026-10-15

FR-MEM-02 asks for a checkpoint before a compaction. Claude Code can't add text to the context before a
compaction, and the baseline never did: its checkpoint came right after. Change request
[#22](https://github.com/ATNexusLab/tightship/issues/22), approved during the
[spec of epic #5](../specs/memory/keep-the-vault.md).

**Changed**

- FR-MEM-02 places the compaction's checkpoint right after the compaction. FR-MEM-02.2 checks that the context
  following the compaction holds a checkpoint.
- Overview: the plugin also writes to the session's environment file, from the amendment of ADR 0001 in the
  same spec.

## 1.2.0 — 2026-10-15

The baseline contract quotes ponytail 4.9.0, and the plugin installs ponytail 5.2.0, whose comment marker,
self-check rule, and comment default differ. One clause of the baseline also repeats ponytail. Change request
[#20](https://github.com/ATNexusLab/tightship/issues/20), approved after the comparison of scenario 18 of the
[spec of epic #4](../specs/contract/load-the-contract.md).

**Changed**

- FR-CTR-04 also lets a baseline rule change where BR-05 requires it: a companion already states the rule, or
  no longer has the default the rule cites. FR-CTR-04.1 accepts a rule listed in the spec of M1 as changed for
  BR-05, with the companion's text.

## 1.1.1 — 2026-10-15

Editorial. No requirement changes meaning. The spec of epic #4 settles OQ-06 for Claude Code.

**Changed**

- OQ-06 now asks about Cursor, Antigravity CLI, and Codex only. For Claude Code, the
  [spec of epic #4](../specs/contract/load-the-contract.md) settles it: the contract reaches the session
  through `SessionStart` hooks, one per part of at most 10,000 characters, fired again after each compaction.

## 1.1.0 — 2026-10-14

The verifier's method mutates a private copy of the repository and runs its tests there, which FR-VER-02
forbade. Change request [#13](https://github.com/ATNexusLab/tightship/issues/13), approved during the spec of
epic #2.

**Added**

- FR-VER-02.2: the repository's working tree is unchanged when the verifier returns.

**Changed**

- FR-VER-02 is renamed "Verifier changes only a private copy": the verifier gets no file-editing tool of the
  harness and changes files only through its shell, in a private copy. FR-VER-02.1 says the harness refuses
  its file-editing tool.
- OQ-05 now asks about Cursor, Antigravity CLI, and Codex only. For Claude Code, the
  [spec of epic #2](../specs/install-with-skills-commands-and-verifier/install-the-plugin.md) settles it: the
  companions are dependencies of the plugin, each installed from its own catalog.

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
