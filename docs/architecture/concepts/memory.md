# Memory

**Milestone:** M1, epic #5

The memory layer is an option ([FR-MEM-10](../../requirements/functional/memory.md#fr-mem-10)). With it on, a
session works with the adopter's vault, a git repository of curated notes in the folder the adopter set
([FR-MEM-09](../../requirements/functional/memory.md#fr-mem-09)). The notes' format is in the
[data model](../../data-model/vault.md).

## The vault folder

The adopter sets the folder in the plugin's `vault_folder` setting, and no default exists. Claude Code hands
the setting only to hooks, so the session-start hook exports it, as `TIGHTSHIP_VAULT`, to the session's
environment file, which Claude Code runs before each command of the agent's shell. `tightship vault` finds the
vault there. With no folder set, or a folder that is gone, the session starts with a line saying it has no
vault, and the agent tells the adopter how to make one with `tightship vault init <folder>`.

## The project's scope

A note belongs to one scope: a project's name or `all`. A session's scope is the name of its git repository's
top folder, lowercased. When the vault has no folder of that name, the longest scope that prefixes it,
followed by `-` or `_`, is used instead, so `tightship-docs` reads the notes of `tightship`.

## What the session start lists

Only the pinned notes of the project's scope and of `all`, and the project's handoff, within 9,000
characters. Every other note is reached by `tightship vault search`. The rest of the vault is counted in one
line, so the agent knows a search may find more.

| Source | Memory context | Export of the vault folder |
| --- | --- | --- |
| `startup`, `clear`, `compact` | Printed | Yes |
| `resume`, `fork` | Not printed: the context already holds it | Yes |

## The checkpoint

A reminder to record what the session learned, printed on every 10th prompt of a session and once after each
compaction ([FR-MEM-02](../../requirements/functional/memory.md#fr-mem-02)). No harness hook adds text before
a compaction, so the checkpoint comes right after it. The prompt count of each session is a file in the
plugin's data folder, deleted after 7 days without a prompt.

Every rule and message: the [epic #5 spec](../../specs/memory/keep-the-vault.md).
