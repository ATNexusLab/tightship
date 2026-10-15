# Business rules

Policies that hold regardless of feature.

<a id="br-01"></a>

## BR-01 — Harness order and milestones

Harness plugins are released one at a time, in this order, one milestone each:

| Milestone | Harness |
| --- | --- |
| M1 | Claude Code |
| M2 | Cursor |
| M3 | Antigravity CLI |
| M4 | Codex |

Work on a harness starts only after the previous harness's plugin is released.

| Attribute | Value |
| --- | --- |
| Rationale | Claude Code is where the workflow runs today, so it proves the canonical source first. Each later harness is resolved in its own turn. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

<a id="br-02"></a>

## BR-02 — One place to edit

The workflow is edited only in this repository. Once the maintainer has migrated, a harness's user
directory holds only personal settings and what a plugin cannot carry.

| Attribute | Value |
| --- | --- |
| Rationale | Two editable copies diverge, and only one of them can be installed by others. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

<a id="br-03"></a>

## BR-03 — English content, with Portuguese translations

Everything a harness plugin installs and every document and identifier in this repository is written in
English, and the English version prevails. The root `README.md`, `CONTRIBUTING.md`, `SECURITY.md`,
`CODE_OF_CONDUCT.md`, and the install guides also have a Brazilian Portuguese translation, named
`<file>.pt-BR.md` next to the English and updated in the same change.

| Attribute | Value |
| --- | --- |
| Rationale | The repository is public and open to any adopter, and most of its first readers read Brazilian Portuguese. |
| Source | Maintainer, elicitation 2026-10-14; translations decided 2026-10-15 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0, changed in v1.7.0 |

<a id="br-04"></a>

## BR-04 — The companion plugins are required

The workflow is used with ponytail, i-have-adhd, and humanizer installed. The plugin installs them with
itself where the harness allows it ([FR-INS-02](functional/install.md#fr-ins-02)).

| Attribute | Value |
| --- | --- |
| Rationale | The contract leaves simplicity, response shape, and prose cleanup to them, so without them those rules are missing. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

<a id="br-05"></a>

## BR-05 — Companion plugins own their rules

The workflow states no rule and ships no function that ponytail, i-have-adhd, or humanizer provides at the
version the plugin installs.

| Attribute | Value |
| --- | --- |
| Rationale | A rule kept in two products diverges. A companion's rules change between versions, so the comparison holds only against the version the plugin installs. |
| Source | Maintainer, elicitation 2026-10-14; change request [#30](https://github.com/ATNexusLab/tightship/issues/30) |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0, changed in v1.5.0 |

**Acceptance criteria**

- **BR-05.1** — Given the shipped contract and pieces, when each rule is compared with the companions at the
  versions the plugin's manifest declares, then no rule restates one a companion states.
