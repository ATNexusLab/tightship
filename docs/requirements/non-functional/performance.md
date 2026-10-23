# Performance efficiency — `PERF`

How much time and context the plugin adds to a session.

<a id="nfr-perf-01"></a>

## NFR-PERF-01 — Session-start overhead

The system shall add at most 500 ms to the start of a session, at the 95th percentile, on a vault of
1,000 notes.

| Attribute | Value |
| --- | --- |
| Rationale | Session start runs before every piece of work. |
| Source | Maintainer, validation 2026-10-14 |
| Priority | Should |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-PERF-01.1** — Across 20 session starts with every option on and a vault of 1,000 notes, the 95th
  percentile of the time the plugin's hooks take is 500 ms or less.

<a id="nfr-perf-02"></a>

## NFR-PERF-02 — Per-prompt overhead

The system shall add at most 100 ms to each prompt, at the 95th percentile.

| Attribute | Value |
| --- | --- |
| Rationale | It is paid on every prompt of every session. |
| Source | Maintainer, validation 2026-10-14 |
| Priority | Should |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-PERF-02.1** — Across 100 prompts with every option on, the 95th percentile of the time the
  plugin's hooks take per prompt is 100 ms or less.

<a id="nfr-perf-03"></a>

## NFR-PERF-03 — Contract size

The system shall ship a contract of at most 8,000 characters with every option on.

| Attribute | Value |
| --- | --- |
| Rationale | The contract takes context in every session and again after every compaction. The contract of v1.1.1 is 25,418 characters. |
| Source | Maintainer, change request [#118](https://github.com/ATNexusLab/tightship/issues/118) |
| Priority | Must |
| Status | approved |
| Milestone | M4 |
| Since | v2.0.0 |

**Acceptance criteria**

- **NFR-PERF-03.1** — Given every option on, when the built contract is measured, then it is 8,000
  characters or fewer.
