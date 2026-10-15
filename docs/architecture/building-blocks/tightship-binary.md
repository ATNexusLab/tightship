# tightship binary

**Lives in:** `tightship/`, not built yet · **Technology:** Rust · **Milestone:** M1, from epic #5

The one program the plugin ships, with a subcommand per function: hooks, memory, and the status line. The
crate appears with the first epic that needs a hook to run logic; the contract's hooks only print a file
(M1, #4).

Claude Code runs it through `bin/tightship`, a `sh` launcher that picks the binary built for the machine.
The `bin/` folder is also on the `PATH` of the agent's shell tool, so a piece calls `tightship` by name.

**It never** connects to any host except the git remote of the adopter's vault, and writes only where
[security](../concepts/security.md) lists. A failure never blocks the session
([NFR-REL-01](../../requirements/non-functional/reliability.md#nfr-rel-01)).
