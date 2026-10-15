# Tokens

A harness renames plugin pieces and has capabilities of its own, so the canonical content never writes a
harness's literal names
([ADR 0001](../../decisions/0001-rust-build-and-tokenized-canonical-content.md)). It writes a token,
and the build replaces it per harness from that harness's definition.

| Token | Renders in Claude Code as |
| --- | --- |
| `{{skill:spec-writing}}` | `tightship:spec-writing` |
| `{{command:spec}}` | `/tightship:spec` |
| `{{agent:adversarial-verifier}}` | `tightship:adversarial-verifier` |
| `{{capability:plan-mode}}` | `plan mode` |
| `{{option:memory}}` (M1, #6) | `${user_config.memory}`, which Claude Code fills in with the option's value |

A token that names no piece, a capability a harness does not define, or an option that is not a boolean of
`content/options.toml`, fails the build. An option token works only in a skill, a command, or an agent, whose
text the harness fills in; the contract is gated by sections instead. That check is how
every reference resolves ([FR-SKL-03](../../requirements/functional/skills.md#fr-skl-03)). A literal
reference to a known piece fails too, so the check cannot be bypassed by writing the name directly.

A harness without a capability defines its token as the plain instruction that replaces it, never as an
empty string, so the piece still reads as a whole sentence.

Every rule and error message: the [epic #1 spec](../../specs/canonical-source-and-build/build-the-plugin.md),
and for `option`, the [epic #6 spec](../../specs/handoff/hand-off-the-session.md).
