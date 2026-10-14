# Performance efficiency — `PERF`

How much time the plugin adds to a session.

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
