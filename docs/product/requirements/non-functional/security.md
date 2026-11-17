# Security — `SEC`

What the public repository and the shipped plugin never carry.

<a id="nfr-sec-01"></a>

## NFR-SEC-01 — No personal identity in shipped content

The system shall ship no name, email address, user name, or home path of the maintainer.

| Attribute | Value |
| --- | --- |
| Rationale | The repository is public, and an adopter's agent must never take them for the maintainer. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-SEC-01.1** — A scan of every shipped file finds 0 occurrences of the maintainer's name, email
  addresses, user names, and home paths.

<a id="nfr-sec-02"></a>

## NFR-SEC-02 — No secret and no vault content in the repository

The system shall keep every secret and every vault note out of the repository.

| Attribute | Value |
| --- | --- |
| Rationale | The repository is public, and the vault is private data. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **NFR-SEC-02.1** — A secret scan of the repository's whole history finds 0 secrets.
- **NFR-SEC-02.2** — The repository holds 0 vault notes; the vault template holds rules only.
