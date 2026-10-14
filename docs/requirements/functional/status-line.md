# Status line — `STL`

The optional status line and the consent it needs to change the harness's settings.

<a id="fr-stl-01"></a>
## FR-STL-01 — Status line on opt-in

> As an adopter, I want the workflow's status line when I ask for it, so that I see the session's state
> under the prompt.

The system shall show the baseline status line's segments while the status line is enabled.

| Attribute | Value |
| --- | --- |
| Rationale | The status line is part of the workflow the maintainer uses today. |
| Source | Maintainer, elicitation 2026-10-14; baseline |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**
- **FR-STL-01.1** — Given the status line is enabled, when a session runs, then the harness shows each
  segment the baseline status line shows for the same session state.

<a id="fr-stl-02"></a>
## FR-STL-02 — Consent before changing settings

> As an adopter, I want to be asked before my settings change, so that nothing I configured is replaced
> without my knowing.

The system shall ask the adopter before it writes the status line into the harness's settings.

| Attribute | Value |
| --- | --- |
| Rationale | The status line may be a setting the adopter owns, outside the plugin. |
| Source | Analysis 2026-10-14, approved by the maintainer |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**
- **FR-STL-02.1** — Given the adopter declines, when the status line would be enabled, then the harness's
  settings do not change.

<a id="fr-stl-03"></a>
## FR-STL-03 — Turning it off restores the previous setting

> As an adopter, I want my previous status line back when I turn this one off, so that trying it costs me
> nothing.

The system shall restore the status line setting the adopter had before enabling the workflow's.

| Attribute | Value |
| --- | --- |
| Rationale | An opt-in that cannot be undone cleanly discourages trying it. |
| Source | Analysis 2026-10-14, approved by the maintainer |
| Priority | Should |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**
- **FR-STL-03.1** — Given the adopter had a status line before, when they disable the workflow's, then
  the harness's settings hold the previous one exactly.
- **FR-STL-03.2** — Given the adopter had no status line before, when they disable the workflow's, then
  the harness's settings hold none.
