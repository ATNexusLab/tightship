# Business rules

Policies that hold regardless of feature.

<a id="br-01"></a>

## BR-01 — Harness order and milestones

Harness plugins are released in this order, one milestone each:

| Milestone | Harness |
| --- | --- |
| M1 | Claude Code |
| M2 | Cursor |
| M3 | Antigravity CLI |
| M4 | Codex |

Work on a harness may run alongside work on the harness before it. Its plugin is never released before
that harness's plugin, and the two may ship in the same release.

| Attribute | Value |
| --- | --- |
| Rationale | Claude Code is where the workflow runs today, so it proves the canonical source first. A later harness never reaches an adopter ahead of the one before it. Building two side by side, or shipping them in one release, keeps that order, which is what an adopter sees. |
| Source | Maintainer, elicitation 2026-10-14; parallel work by change request [#56](https://github.com/ATNexusLab/tightship/issues/56); one release for two harnesses by maintainer decision in [#84](https://github.com/ATNexusLab/tightship/pull/84) |
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

The workflow is used with the companion plugins installed. The plugin installs them with itself where the
harness allows it ([FR-INS-02](functional/install.md#fr-ins-02)).

| Attribute | Value |
| --- | --- |
| Rationale | The contract leaves simplicity, response shape, and prose cleanup to them, so without them those rules are missing. |
| Source | Maintainer, elicitation 2026-10-14; retired by [#149](https://github.com/ATNexusLab/tightship/issues/149) |
| Priority | Must |
| Status | retired |
| Milestone | M1 |
| Since | v1.0.0, retired in v4.0.0 |

Retired in version 4.0.0: the plugin carries what it used from the companions, in the contract
([FR-CTR-08](functional/contract.md#fr-ctr-08)), the codebase map
([FR-CTR-09](functional/contract.md#fr-ctr-09)), and the prose skill
([FR-SKL-06](functional/skills.md#fr-skl-06)), and removes what the companions left
([FR-INS-11](functional/install.md#fr-ins-11)). [BR-05](#br-05) keeps it from depending on them again.

<a id="br-05"></a>

## BR-05 — The workflow depends on no third-party plugin

The workflow installs no plugin but its own, and no manifest, catalog, harness definition, or installer of
the workflow names another plugin, except in the list of the plugins the workflow removes.

| Attribute | Value |
| --- | --- |
| Rationale | A third-party plugin changes its rules at any release, at a version the workflow cannot pin in every harness, and a rule the contract leaves to it disappears with it. The audit of epic [#140](https://github.com/ATNexusLab/tightship/issues/140) found what the workflow used from the companions small enough to carry itself. |
| Source | Maintainer, elicitation 2026-10-14; change requests [#30](https://github.com/ATNexusLab/tightship/issues/30) and [#149](https://github.com/ATNexusLab/tightship/issues/149) |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0, changed in v1.5.0 and v4.0.0 |

**Acceptance criteria**

- **BR-05.1** — Given a harness definition that names another plugin outside the list of the plugins the
  workflow removes, when the plugins are built, then the build fails and names that harness.
- **BR-05.2** — Given a machine with no plugin installed, when the adopter installs the workflow in each
  supported harness, then no plugin, catalog, or skill other than the workflow's is installed.
