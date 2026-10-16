# Flexibility — `FLEX`

The harnesses and operating systems the product runs on, and how the canonical content stays apart from
any one harness.

<a id="nfr-flex-01"></a>

## NFR-FLEX-01 — Claude Code is supported

The system shall pass every M1 acceptance criterion in Claude Code, from version 2.1.295.

| Attribute | Value |
| --- | --- |
| Rationale | Claude Code is the first harness ([BR-01](../business-rules.md#br-01)). 2.1.295 is the first version whose `SessionStart` hooks' exported variables reach the Bash tool after an in-app `/resume` or `/branch` ([epic #8 spec](../../specs/release-m1/release-the-plugin.md)). |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-FLEX-01.1** — 100% of the functional acceptance criteria pass in Claude Code 2.1.295.

<a id="nfr-flex-02"></a>

## NFR-FLEX-02 — Cursor is supported, editor and command line

The system shall pass every functional acceptance criterion in Cursor's editor and in its command-line
agent.

| Attribute | Value |
| --- | --- |
| Rationale | The maintainer uses both. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M2 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-FLEX-02.1** — 100% of the functional acceptance criteria pass in Cursor's editor, and 100% in its
  command-line agent, each at its minimum supported version. A criterion one of them cannot meet is
  settled by [OQ-07](../open-questions.md#oq-07) or [OQ-08](../open-questions.md#oq-08) before M2 starts.

<a id="nfr-flex-03"></a>

## NFR-FLEX-03 — Antigravity CLI is supported

The system shall pass every functional acceptance criterion in Antigravity CLI, from version 1.4.0.

| Attribute | Value |
| --- | --- |
| Rationale | Third harness in [BR-01](../business-rules.md#br-01). 1.4.0 is the version verified for `plugin.json` / `hooks.json` validation, `injectSteps` hook output, and `statusLine` support ([epic #59 spec](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md)). |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M3 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-FLEX-03.1** — 100% of the functional acceptance criteria pass in Antigravity CLI 1.4.0, except
  [FR-MEM-02.2](../functional/memory.md#fr-mem-02), which [epic #59 spec](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md)
  records as impossible in Antigravity CLI 1.4.0 because `SessionStart` fires only once at conversation
  start and no post-compaction hook event exists.

<a id="nfr-flex-04"></a>

## NFR-FLEX-04 — Codex is supported

The system shall pass every functional acceptance criterion in Codex.

| Attribute | Value |
| --- | --- |
| Rationale | Fourth harness in [BR-01](../business-rules.md#br-01). |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M4 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-FLEX-04.1** — 100% of the functional acceptance criteria pass in Codex at its minimum supported
  version, except those its spec records as impossible in that harness, each with its reason.

<a id="nfr-flex-05"></a>

## NFR-FLEX-05 — Three operating systems

The system shall run with the same behavior on macOS, Linux, and Windows without WSL.

| Attribute | Value |
| --- | --- |
| Rationale | The maintainer works on the three, and an adopter may use any of them. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-FLEX-05.1** — 100% of the functional acceptance criteria of a milestone pass on each of the three
  operating systems before its release.

<a id="nfr-flex-06"></a>

## NFR-FLEX-06 — Harness-neutral canonical content

The system shall keep every name of a harness, its paths, and its tools out of the canonical content,
except in the per-harness places [ADR 0001](../../decisions/0001-rust-build-and-tokenized-canonical-content.md)
sets apart.

| Attribute | Value |
| --- | --- |
| Rationale | Canonical content tied to one harness has to be rewritten for each of the others. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-FLEX-06.1** — A scan of the canonical content outside the per-harness places finds 0 harness
  names, harness paths, and harness tool names.

<a id="nfr-flex-07"></a>

## NFR-FLEX-07 — A new harness leaves the canonical content untouched

The system shall add a harness without changing a canonical piece.

| Attribute | Value |
| --- | --- |
| Rationale | If adding a harness edits shared content, it can break the harnesses already released. |
| Source | Analysis 2026-10-14, approved by the maintainer |
| Priority | Should |
| Status | approved |
| Milestone | M2 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-FLEX-07.1** — The diff that adds a harness changes 0 lines of canonical content.
