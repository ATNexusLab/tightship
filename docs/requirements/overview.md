# Overview

## Stakeholders

| Role | Who | Interest |
| --- | --- | --- |
| Maintainer | The author of the workflow and the only editor of this repository | Edits each piece once, in this repository, and uses the same workflow in all four harnesses |
| Adopter | Anyone who installs the plugin from this public repository, the maintainer included | Installs with the harness's own mechanism, chooses the options, and gets nothing of the maintainer's identity or data |
| Harness vendors | Anthropic, Cursor, Google, OpenAI | Define the plugin formats and limits the product must follow; not consulted |
| Companion maintainers | The authors of ponytail, i-have-adhd, and humanizer | Own the rules their plugins carry; not consulted |

## Operating environment

- **Operating systems:** macOS, Linux, and Windows without WSL.
- **Harnesses**, with the version observed on the maintainer's machine on 2026-10-14:

  | Harness | Version observed | Milestone |
  | --- | --- | --- |
  | Claude Code | 2.1.299 | M1 |
  | Cursor | command-line agent 2026.10.14; editor not installed | M2 |
  | Antigravity CLI | not installed | M3 |
  | Codex | 0.162.1 | M4 |

  The minimum supported version of each harness is [OQ-04](open-questions.md#oq-04).
- **Adopter prerequisites:** nothing on macOS and Linux; Git for Windows on Windows, so that the plugin's
  hooks run under Git Bash. The vault is a git repository, so the memory option needs git on every system
  ([ADR 0001](../decisions/0001-rust-build-and-tokenized-canonical-content.md)).

## Constraints

- The plugin connects only to the git remote of the adopter's vault, and writes only to the vault, the
  harness's persistent plugin data folder, and the harness's settings file for the status line, with consent
  ([ADR 0001](../decisions/0001-rust-build-and-tokenized-canonical-content.md)).
- Each harness's plugin limits are read in its own documentation and checked against the installed version
  by that harness's spec.

## Assumptions and dependencies

- **Baseline of the pre-plugin workflow:** commit `1f41699` (2026-10-14) of the maintainer's private
  configuration repository. It holds the contract, 13 skills, 9 commands, the verifier, the memory
  script, the status line, and the vault template. Every parity requirement refers to it.
- Each harness keeps the plugin format its documentation describes on the day its milestone starts.
- An adopter can reach this public repository.
- The companion plugins are maintained by third parties and can change without notice.
