# tightship binary

**Lives in:** `tightship/` · **Technology:** Rust · **Milestone:** M1, from epic #5

The one program the plugin ships, with a subcommand per function. Epic #5 builds the first two:

| Subcommand | Run by | Does |
| --- | --- | --- |
| `tightship hook start\|export\|checkpoint\|prompt` | The plugin's hooks | Puts the [memory](../concepts/memory.md) context and the checkpoints in the session, and exports the vault's folder to it |
| `tightship vault path\|init\|search\|new\|lint\|sync` | The agent, through its shell | Reads, writes, lints, and syncs the adopter's vault, and creates one from the template |

The status line arrives with epic #7. The binary reads only `TIGHTSHIP_*` variables, which the hook lines of
each harness set from that harness's own, so it names no harness.

`bin/tightship` is a `sh` launcher that runs `bin/<platform>/tightship` for the machine, among `macos`,
`linux-x64`, `linux-arm64`, and `windows-x64`. The plugin's `bin/` is on the `PATH` of the agent's shell tool,
so a piece calls `tightship` by name. Until epic #8, `cargo xtask build` compiles only the platform of the
machine that builds.

Inside, the code is split by domain, `vault/` and `hook/`, with a pure core and the I/O at the edges, and it
depends on `clap`, `jiff`, `serde_json`, and `deunicode`
([ADR 0002](../../decisions/0002-tightship-binary-modules-and-dependencies.md)).

**It never** connects to any host other than the vault's git remote, and only from `vault sync`, never from a
hook. It writes only where [security](../concepts/security.md) lists, and never exits with 2, the code with
which a hook blocks a prompt. A failure never blocks the session ([NFR-REL-01](../../requirements/non-functional/reliability.md#nfr-rel-01)).

**To change it safely:** every rule, message, and exit code is in the
[epic #5 spec](../../specs/memory/keep-the-vault.md), and the tests in `tightship/` lock them.
