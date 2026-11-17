# Reliability — `REL`

What a failure of the plugin may and may not do to the session.

<a id="nfr-rel-01"></a>

## NFR-REL-01 — A plugin failure never blocks the session

The system shall let the session continue when any of its hooks or scripts fails.

| Attribute | Value |
| --- | --- |
| Rationale | The workflow assists the session; a broken vault or a missing tool must not stop the adopter's work. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-REL-01.1** — With the vault missing, unreadable, or malformed, 100% of sessions start and accept
  prompts.
