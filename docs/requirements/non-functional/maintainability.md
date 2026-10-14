# Maintainability — `MNT`

How the product is edited.

<a id="nfr-mnt-01"></a>
## NFR-MNT-01 — One authored copy per piece

The system shall have exactly one authored copy of each workflow piece; every harness's version of it is
generated from that copy.

| Attribute | Value |
| --- | --- |
| Rationale | A piece edited in several copies diverges ([BR-02](../business-rules.md#br-02)). |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**
- **NFR-MNT-01.1** — Changing one piece's text means editing 1 file, and every harness's plugin built
  afterward carries the change.
