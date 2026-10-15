# Memory

**Milestone:** M1, epics #5 and #6

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

## The sync

`tightship vault sync` records the vault's changes in a commit, rebases them on the vault's git remote, and
pushes ([FR-MEM-11](../../requirements/functional/memory.md#fr-mem-11)). A note written on two machines with
the same id is renumbered on the machine that syncs last, and every link to it follows. The binary never
resolves a conflict: it leaves the rebase where git stopped it and names the next step, and the command `sync`
has the agent resolve it and sync again. Nothing syncs from a hook, since no hook uses the network.

## The handoff

The one note per scope that carries a session's state to the next session, on any machine
([FR-MEM-06](../../requirements/functional/memory.md#fr-mem-06),
[FR-MEM-07](../../requirements/functional/memory.md#fr-mem-07), M1, #6). The command `handoff release` has the
agent write its five parts, the goal, what is done, the next step, the open decisions, and the repository's
state; `tightship vault handoff release` writes it and removes the scope's earlier ones, and the sync pushes it.
The command `handoff claim` syncs, has `tightship vault handoff claim` print and remove every handoff of the
scope, syncs again, and has the agent name what the repository lacks. The project's repository is only read.
With memory off, the command says so and runs nothing; it reads the option through `{{option:memory}}`.

Every rule and message: the [epic #5 spec](../../specs/memory/keep-the-vault.md) and the
[epic #6 spec](../../specs/handoff/hand-off-the-session.md).
