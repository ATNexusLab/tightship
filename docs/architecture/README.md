# Architecture

The map of the system, in the sections of arc42, with C4 diagrams. The structure inside the plugin is
not decided yet: the M1 ADR settles it, together with the stack
([OQ-02](../requirements/open-questions.md#oq-02)), the plugin's network and disk reach
([OQ-03](../requirements/open-questions.md#oq-03)), and how canonical content names a harness capability
([OQ-09](../requirements/open-questions.md#oq-09)). Until then, this page holds only what the
requirements already fix.

## Introduction and goals

The workflow is a way of working with coding agents: an always-on contract, on-demand skills, a command
per step of the development loop, a read-only verifier, an optional memory layer, and an optional status
line. The product packages it as a native plugin for four harnesses, built from one canonical source in
this repository ([SRS](../requirements/README.md)).

Quality goals, highest first:

1. One authored copy of each piece, whatever the harness ([NFR-MNT-01](../requirements/non-functional/maintainability.md#nfr-mnt-01),
   [NFR-FLEX-06](../requirements/non-functional/flexibility.md#nfr-flex-06)).
2. A plugin failure never blocks a session ([NFR-REL-01](../requirements/non-functional/reliability.md#nfr-rel-01)).
3. Nothing of the maintainer's identity or vault ships ([NFR-SEC-01](../requirements/non-functional/security.md#nfr-sec-01),
   [NFR-SEC-02](../requirements/non-functional/security.md#nfr-sec-02)).
4. Session start and each prompt stay within their time budgets
   ([NFR-PERF-01](../requirements/non-functional/performance.md#nfr-perf-01),
   [NFR-PERF-02](../requirements/non-functional/performance.md#nfr-perf-02)).

## Constraints

See [the operating environment and constraints](../requirements/overview.md).

## Context and scope

![C4 context](diagrams/context.drawio.svg)

Dashed boxes are not built yet. Every element belongs to the milestone in its label.

| Element | Kind | What flows |
| --- | --- | --- |
| Maintainer | Person | Edits each workflow piece once, in this repository |
| Adopter | Person | Installs the plugin, picks its options, invokes its commands |
| This repository | External system, GitHub | Holds the canonical source and the catalog each harness installs from |
| Claude Code, Cursor, Antigravity CLI, Codex | External systems, one milestone each ([BR-01](../requirements/business-rules.md#br-01)) | Load the plugin in its harness's own format |
| Companion plugins | External systems | Installed with the plugin where the harness allows it ([FR-INS-02](../requirements/functional/install.md#fr-ins-02)) |
| Vault | External system, the adopter's git repository | Notes and handoffs the memory layer reads and writes |

Out of scope: [the SRS's list](../requirements/README.md#scope).

## Solution strategy

Pending the M1 ADR.

## Building blocks, runtime, deployment, and crosscutting concepts

Pending the M1 ADR. Each gets its own file here once the plugin's structure is decided.

## Architecture decisions

None yet. ADRs land in `docs/decisions/` through `/spec`.

## Quality requirements

See [the non-functional requirements](../requirements/non-functional/README.md).

## Risks and technical debt

The known risks are the SRS's [assumptions and dependencies](../requirements/overview.md#assumptions-and-dependencies):
each harness's plugin format and the companion plugins can change without notice.

## Glossary

See [the glossary](../requirements/glossary.md).
