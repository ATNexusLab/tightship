# Memory

**Milestone:** M1, epics #5 and #6; M3, epic #59

The memory layer is an option ([FR-MEM-10](../../requirements/functional/memory.md#fr-mem-10)). With it on, a
session works with the adopter's vault, a git repository of curated notes in the folder the adopter set
([FR-MEM-09](../../requirements/functional/memory.md#fr-mem-09)). The notes' format is in the
[data model](../../data-model/vault.md).

## The vault folder

The adopter sets the folder in the plugin's `vault_folder` setting, and no default exists. Claude Code hands
the setting only to hooks, so the session-start hook exports it, as `TIGHTSHIP_VAULT`, to the session's
environment file, which Claude Code runs before each command of the agent's shell. In Antigravity CLI
(M3, #59), `vault_folder` lives in `${PLUGIN_DATA}/options.toml`, and `tightship` reads it from
`${TIGHTSHIP_DATA}/options.toml` whenever `TIGHTSHIP_VAULT` is unset. With no folder set, or a folder that is
gone, the session starts with a line saying it has no vault, and the agent tells the adopter how to make one
with `tightship vault init <folder>`.

## The project's scope

A note belongs to one scope: a project's name or `all`. A session's scope is the name of its git repository's
top folder, lowercased. The session start finds the project in the hook's working directory, or, in a harness
that runs hooks in the plugin's folder, at the JSON pointer `TIGHTSHIP_PROJECT_POINTER` names in the hook's
input. When the vault has no folder of that name, the longest scope that prefixes it,
followed by `-` or `_`, is used instead, so `tightship-docs` reads the notes of `tightship`.

## What the session start lists

Only the pinned notes of the project's scope and of `all`, the project's handoff, and, since M4 (#118), the
project's debts, within 9,000 characters. A debt is a note of the project's scope tagged `to-repo`, listed
with the file its `**Moves to:**` line names, ahead of the pinned notes. Every other note is reached by
`tightship vault search`. The rest of the vault is counted in one line, so the agent knows a search may find
more.

| Source | Memory context | Export of the vault folder |
| --- | --- | --- |
| `startup`, `clear`, `compact` | Printed | Yes |
| `resume`, `fork` | Not printed: the context already holds it | Yes |

## The checkpoint

A reminder to record what the session learned, printed once after each compaction
([FR-MEM-02](../../requirements/functional/memory.md#fr-mem-02)). No harness hook adds text before a
compaction, so the checkpoint comes right after it (in Claude Code, which fires `SessionStart` on `compact`).
Until M4 (#118) it also came on every 10th prompt, which pushed the agent to write notes nobody asked for.

## The sync

`tightship vault sync` records the vault's changes in a commit, rebases them on the vault's git remote, and
pushes ([FR-MEM-11](../../requirements/functional/memory.md#fr-mem-11)). A note written on two machines with
the same id is renumbered on the machine that syncs last, and every link to it follows. The binary never
resolves a conflict: it leaves the rebase where git stopped it and names the next step, and the command `sync`
has the agent resolve it and sync again. Nothing syncs from a hook: the only network a hook starts is the
self-update of [ADR 0003](../../decisions/0003-the-plugin-updates-itself-from-github-releases.md), which
never touches the vault.

Since M4 (#118), after the rebase and before the push, the sync removes each note past its lifetime: a `gotcha`
or `project` that the user did not state and that is not pinned, 30 days after its `updated`
([FR-MEM-12](../../requirements/functional/memory.md#fr-mem-12)). Each link to it becomes plain text. Then it
rewrites the plugin's block of the vault's `AGENTS.md` to the installed template and leaves the rest of the
file alone ([FR-MEM-13](../../requirements/functional/memory.md#fr-mem-13)). Each step is its own commit, so
the vault's history keeps what was removed.

## The handoff

The one note per scope that carries a session's state to the next session, on any machine
([FR-MEM-06](../../requirements/functional/memory.md#fr-mem-06),
[FR-MEM-07](../../requirements/functional/memory.md#fr-mem-07), M1, #6). The command `handoff release` has the
agent write its five parts, the goal, what is done, the next step, the open decisions, and the repository's
state; `tightship vault handoff release` writes it and removes the scope's earlier ones, and the sync pushes it.
The command `handoff claim` syncs, has `tightship vault handoff claim` print and remove every handoff of the
scope, syncs again, and has the agent name what the repository lacks. The project's repository is only read.
With memory off, the command says so and runs nothing; it reads the option through `{{option:memory}}`.

Every rule and message: the [epic #5 spec](../../specs/memory/keep-the-vault.md), the
[epic #6 spec](../../specs/handoff/hand-off-the-session.md), and the
[epic #59 spec](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md).
