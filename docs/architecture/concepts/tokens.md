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

A token that names no piece, or a capability a harness does not define, fails the build. That check is how
every reference resolves ([FR-SKL-03](../../requirements/functional/skills.md#fr-skl-03)). A literal
reference to a known piece fails too, so the check cannot be bypassed by writing the name directly.

A harness without a capability defines its token as the plain instruction that replaces it, never as an
empty string, so the piece still reads as a whole sentence.

Every rule and error message: the [epic #1 spec](../../specs/canonical-source-and-build/build-the-plugin.md).
