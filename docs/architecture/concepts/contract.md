# Contract

**Milestone:** M1, epic #4

The contract is the always-on text that tells the agent how to work
([FR-CTR-01](../../requirements/functional/contract.md#fr-ctr-01)). It has one authored copy,
`content/contract.md`, and each harness renders it into whatever puts text in a session's context.

## How it reaches a Claude Code session

Claude Code puts a `SessionStart` hook's output in the context, and caps each hook's text at 10,000
characters. The contract is longer, so the build splits it:

1. It resolves the tokens, then packs the preamble and the sections, in file order, into parts of at most
   `[contract] limit` characters, the value in `harnesses/claude-code/harness.toml`. Each part opens with the
   contract's title and its number, such as `(1/3)`, because hooks run in parallel and their text arrives in
   the order they finish.
2. A section the front matter's `[gated]` table names goes to a file of its own option instead, such as
   `contract/memory.md`. That file is never split: over the limit, the build fails.
3. Each file becomes one hook: an `sh` line that prints it unless the contract option, or the file's own
   option, is exactly `false`.

The hooks fire on a new session, after `/clear`, and after each compaction. A resumed or forked session
already holds the copy placed before. A hook that fails costs only its own part; the session starts.

| Contract | Memory | The context holds |
| --- | --- | --- |
| On | On | Every part and the memory section |
| On | Off | Every part, none of which names the vault |
| Off | Any | Nothing from the contract |

Every rule and error message: the [epic #4 spec](../../specs/contract/load-the-contract.md).
