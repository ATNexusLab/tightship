# Verifier — `VER`

The read-only subagent that tries to refute a claim before it is trusted.

<a id="fr-ver-01"></a>
## FR-VER-01 — Verifier can be dispatched

> As an adopter, I want the agent to dispatch the verifier, so that a finding or a claim of done is
> challenged by an independent skeptic.

The system shall make the verifier available as a subagent the agent can dispatch.

| Attribute | Value |
| --- | --- |
| Rationale | Skills and commands of the baseline dispatch it. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**
- **FR-VER-01.1** — Given the plugin is installed, when the agent dispatches the verifier with a claim,
  then the verifier returns a verdict on that claim.

<a id="fr-ver-02"></a>
## FR-VER-02 — Verifier is read-only

> As an adopter, I want the verifier unable to change anything, so that a check never alters what it
> checks.

The system shall give the verifier no tool that writes a file or changes the repository.

| Attribute | Value |
| --- | --- |
| Rationale | A verifier that edits can make the claim true instead of testing it. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**
- **FR-VER-02.1** — Given the verifier is running, when it tries to write or edit a file, then the
  harness refuses the action.
