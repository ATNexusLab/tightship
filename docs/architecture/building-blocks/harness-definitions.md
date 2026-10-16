# Harness definitions

**Lives in:** `harnesses/<harness>/harness.toml` · **Milestone:** M1, epic #1 (Claude Code); M3, epic #59
(Antigravity CLI); one more per milestone after it

The per-harness place of
[ADR 0001](../../decisions/0001-rust-build-and-tokenized-canonical-content.md): the only files that
may name a harness. Each one holds:

- The plugin's manifest values.
- How the harness names each kind of piece, such as `/{plugin}:{name}` for a command in Claude Code and
  `/{name}` in Antigravity CLI, and how it renders an option for a piece, `${user_config.{name}}` in Claude
  Code (M1, #6) and `{name}` in Antigravity CLI (M3, #59).
- The text of every capability token.
- The tools a read-only subagent gets (`Read, Grep, Glob, Bash` in Claude Code;
  `view_file, list_dir, find_by_name, grep_search, run_command` in Antigravity CLI, M3, #59).
- The companion plugins the plugin depends on, each with the catalog or repository that lists it (M1, #2).
- The denylist the neutrality check applies, across every harness in `harnesses/`, to the canonical content
  and to the binary's sources in `tightship/src/` (#78).
- `[contract] limit`: the most characters one hook may put in the context (M1, #4).

**It never** holds a piece's text. A sentence that only one harness needs is a capability, and its text
lives here.

**To change it safely:** a capability added to the content needs an entry in every harness's file, or the
build fails. Adding a harness adds a folder and a renderer, and leaves `content/` untouched
([NFR-FLEX-07](../../requirements/non-functional/flexibility.md#nfr-flex-07)).
