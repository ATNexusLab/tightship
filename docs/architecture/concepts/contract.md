# Contract

**Milestone:** M1, epic #4; M3, epic #59

The contract is the always-on text that tells the agent how to work
([FR-CTR-01](../../requirements/functional/contract.md#fr-ctr-01)). It has one authored copy,
`content/contract.md`, and each harness renders it into whatever puts text in a session's context.

## How it reaches a Claude Code session

Claude Code puts a `SessionStart` hook's output in the context, and caps each hook's text at 10,000
characters. Until M4 the contract was longer, so the build splits it; since M4 (#118) the build also fails when
the contract renders past 8,000 characters with every option on
([NFR-PERF-03](../../requirements/non-functional/performance.md#nfr-perf-03)), so it is one part plus its memory
section:

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

## How it reaches an Antigravity CLI session (M3, #59)

Antigravity CLI truncates each `rules/*.md` file at 24,000 bytes and loads `rules/` unconditionally. Instead,
`hooks.json` runs `tightship hook contract-part` at `SessionStart` for `part-1.md` and `memory.md` (with
`[contract] limit = 100000`), which reads `${TIGHTSHIP_DATA}/options.toml` and prints each enabled file
wrapped in `{"injectSteps":[{"systemMessage":{"systemMessage":"..."}}]}`. Because
`SessionStart` in Antigravity CLI fires only once at conversation start, the build also writes
`skills/contract/SKILL.md` and prepends a line to `contract/part-1.md` instructing the agent to load the
`contract` skill after `/compact`.

| Contract | Memory | The context holds |
| --- | --- | --- |
| On | On | Every part and the memory section |
| On | Off | Every part, none of which names the vault |
| Off | Any | Nothing from the contract |

Every rule and error message: the [epic #4 spec](../../specs/contract/load-the-contract.md) and the
[epic #59 spec](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md).
