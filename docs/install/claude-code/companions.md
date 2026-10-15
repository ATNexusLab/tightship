# The companions

English · [Português](companions.pt-BR.md)

tightship is used with three third-party plugins. It declares them as dependencies, so
`claude plugin install tightship@tightship` installs them too, once their catalogs are added.

| Companion | Catalog | What it brings |
| --- | --- | --- |
| ponytail | `DietrichGebert/ponytail` | Pushes the agent toward the simplest code that works: no speculative abstraction, the standard library first |
| i-have-adhd | `ayghri/i-have-adhd` | Shapes answers to be acted on: the next action first, numbered steps, no preamble |
| humanizer | `blader/humanizer` | Rewrites prose meant for people so it does not read as generated text |

A companion you already had installed is reused, not installed a second time. No companion's version is
pinned: you get each one's latest.

On uninstall, `--prune` removes only the companions the tightship install added. One you installed yourself
stays. See [uninstall](uninstall.md).
