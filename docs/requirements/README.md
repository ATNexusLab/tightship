# Workflow — Software Requirements Specification

**Version:** 1.6.0 · **Date:** 2026-10-15 · [Changelog](CHANGELOG.md)

## Purpose

The workflow is a way of working with coding agents. It has an always-on working contract, skills the
agent loads on demand, a command for each step of the development loop, a read-only verifier subagent,
a curated memory layer, and a status line.

Today it lives in one person's private Claude Code user directory and is installed by a guided script
that merges files into that directory. In that form it reaches one harness, cannot be installed piece by
piece, and is edited in a place nobody else can install from.

This product packages the workflow as a native plugin for four agent harnesses, built from one canonical
source in this repository. Anyone installs it with the harness's own plugin mechanism and chooses which
optional parts to turn on.

## Scope

In scope:

- The canonical source of every workflow piece.
- One plugin per harness: Claude Code, Cursor, Antigravity CLI, and Codex, released in that order
  ([BR-01](business-rules.md#br-01)).
- The contract, the skills, the commands, and the verifier.
- The memory layer and the vault template, as an option.
- The status line, as an option.
- Installation with the companion plugins, the choice of options, update, uninstall, and the guided
  migration from the pre-plugin install.

Out of scope:

| Item | Reason |
| --- | --- |
| Listing in a vendor's official or curated marketplace | It depends on each vendor's review. The plugin installs from this repository's own catalog without it. |
| Personal harness settings: theme, effort, language, permissions, notifications, enabled plugins | They are one machine's preferences, not workflow, and a plugin cannot carry them. |
| Skills that serve only the maintainer's machine, such as the desktop and crash-diagnosis skills | They are personal configuration, not workflow. |
| MCP servers | The workflow provisions none. |
| The content of any person's vault | It is private data and never ships ([NFR-SEC-02](non-functional/security.md#nfr-sec-02)). |
| Harnesses other than the four named | None was requested. |

## Contents

[Overview](overview.md) · [Glossary](glossary.md) · [Business rules](business-rules.md) ·
[Open questions](open-questions.md) · [Functional](functional/README.md) ·
[Non-functional](non-functional/README.md)

Milestones M1 to M4 are defined by [BR-01](business-rules.md#br-01).

## Requirements

| ID | Name | Priority | Status | Milestone |
| --- | --- | --- | --- | --- |
| [FR-CTR-01](functional/contract.md#fr-ctr-01) | Contract in every session | Must | approved | M1 |
| [FR-CTR-02](functional/contract.md#fr-ctr-02) | Contract can be turned off | Must | approved | M1 |
| [FR-CTR-03](functional/contract.md#fr-ctr-03) | Contract matches the enabled options | Must | approved | M1 |
| [FR-CTR-04](functional/contract.md#fr-ctr-04) | Every baseline rule is carried over | Must | approved | M1 |
| [FR-CTR-05](functional/contract.md#fr-ctr-05) | Behavior without the contract is documented | Must | approved | M1 |
| [FR-SKL-01](functional/skills.md#fr-skl-01) | Skills available | Must | approved | M1 |
| [FR-SKL-02](functional/skills.md#fr-skl-02) | Commands invocable | Must | approved | M1 |
| [FR-SKL-03](functional/skills.md#fr-skl-03) | References resolve | Must | approved | M1 |
| [FR-VER-01](functional/verifier.md#fr-ver-01) | Verifier can be dispatched | Must | approved | M1 |
| [FR-VER-02](functional/verifier.md#fr-ver-02) | Verifier changes only a private copy | Must | approved | M1 |
| [FR-MEM-01](functional/memory.md#fr-mem-01) | Memory context at session start | Must | approved | M1 |
| [FR-MEM-02](functional/memory.md#fr-mem-02) | Checkpoint reminder | Must | approved | M1 |
| [FR-MEM-03](functional/memory.md#fr-mem-03) | Search the notes | Must | approved | M1 |
| [FR-MEM-04](functional/memory.md#fr-mem-04) | Write a note in one step | Must | approved | M1 |
| [FR-MEM-05](functional/memory.md#fr-mem-05) | Lint the vault | Must | approved | M1 |
| [FR-MEM-06](functional/memory.md#fr-mem-06) | Release a handoff | Must | approved | M1 |
| [FR-MEM-07](functional/memory.md#fr-mem-07) | Claim a handoff | Must | approved | M1 |
| [FR-MEM-08](functional/memory.md#fr-mem-08) | Create a vault from the template | Must | approved | M1 |
| [FR-MEM-09](functional/memory.md#fr-mem-09) | Vault location is the adopter's choice | Must | approved | M1 |
| [FR-MEM-10](functional/memory.md#fr-mem-10) | Memory can be turned off | Must | approved | M1 |
| [FR-MEM-11](functional/memory.md#fr-mem-11) | Sync the vault | Must | approved | M1 |
| [FR-STL-01](functional/status-line.md#fr-stl-01) | Status line on opt-in | Must | approved | M1 |
| [FR-STL-02](functional/status-line.md#fr-stl-02) | Consent before changing settings | Must | approved | M1 |
| [FR-STL-03](functional/status-line.md#fr-stl-03) | Turning it off restores the previous setting | Should | approved | M1 |
| [FR-INS-01](functional/install.md#fr-ins-01) | Install through the harness's plugin mechanism | Must | approved | M1 |
| [FR-INS-02](functional/install.md#fr-ins-02) | Companions are installed with the plugin | Must | approved | M1 |
| [FR-INS-03](functional/install.md#fr-ins-03) | Choose the options | Must | approved | M1 |
| [FR-INS-04](functional/install.md#fr-ins-04) | Change a choice after installation | Should | approved | M1 |
| [FR-INS-05](functional/install.md#fr-ins-05) | Update keeps the choices | Must | approved | M1 |
| [FR-INS-06](functional/install.md#fr-ins-06) | Uninstall leaves nothing behind | Should | approved | M1 |
| [FR-INS-07](functional/install.md#fr-ins-07) | Guided migration from the pre-plugin install | Must | approved | M1 |
| [FR-INS-08](functional/install.md#fr-ins-08) | Install guide per harness | Must | approved | M1 |
| [NFR-FLEX-01](non-functional/flexibility.md#nfr-flex-01) | Claude Code is supported | Must | approved | M1 |
| [NFR-FLEX-02](non-functional/flexibility.md#nfr-flex-02) | Cursor is supported, editor and command line | Must | approved | M2 |
| [NFR-FLEX-03](non-functional/flexibility.md#nfr-flex-03) | Antigravity CLI is supported | Must | approved | M3 |
| [NFR-FLEX-04](non-functional/flexibility.md#nfr-flex-04) | Codex is supported | Must | approved | M4 |
| [NFR-FLEX-05](non-functional/flexibility.md#nfr-flex-05) | Three operating systems | Must | approved | M1 |
| [NFR-FLEX-06](non-functional/flexibility.md#nfr-flex-06) | Harness-neutral canonical content | Must | approved | M1 |
| [NFR-FLEX-07](non-functional/flexibility.md#nfr-flex-07) | A new harness leaves the canonical content untouched | Should | approved | M2 |
| [NFR-COMP-01](non-functional/compatibility.md#nfr-comp-01) | One vault across harnesses | Must | approved | M2 |
| [NFR-SEC-01](non-functional/security.md#nfr-sec-01) | No personal identity in shipped content | Must | approved | M1 |
| [NFR-SEC-02](non-functional/security.md#nfr-sec-02) | No secret and no vault content in the repository | Must | approved | M1 |
| [NFR-MNT-01](non-functional/maintainability.md#nfr-mnt-01) | One authored copy per piece | Must | approved | M1 |
| [NFR-REL-01](non-functional/reliability.md#nfr-rel-01) | A plugin failure never blocks the session | Must | approved | M1 |
| [NFR-PERF-01](non-functional/performance.md#nfr-perf-01) | Session-start overhead | Should | approved | M1 |
| [NFR-PERF-02](non-functional/performance.md#nfr-perf-02) | Per-prompt overhead | Should | approved | M1 |
| [BR-01](business-rules.md#br-01) | Harness order and milestones | Must | approved | M1 |
| [BR-02](business-rules.md#br-02) | One place to edit | Must | approved | M1 |
| [BR-03](business-rules.md#br-03) | English content | Must | approved | M1 |
| [BR-04](business-rules.md#br-04) | The companion plugins are required | Must | approved | M1 |
| [BR-05](business-rules.md#br-05) | Companion plugins own their rules | Must | approved | M1 |
