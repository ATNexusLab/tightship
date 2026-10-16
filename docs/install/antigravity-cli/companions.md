# The companions

English · [Português](companions.pt-BR.md)

tightship is used alongside three companion plugins and skills.

| Companion | Type | What it brings |
| --- | --- | --- |
| ponytail | Plugin (`DietrichGebert/ponytail`) | Pushes the agent toward the simplest code that works: no speculative abstraction, standard library first |
| i-have-adhd | Plugin (`ayghri/i-have-adhd`) | Shapes answers to be acted on: next action first, numbered steps, no preamble |
| humanizer | Skill (`blader/humanizer`) | Rewrites prose meant for people so it does not read as generated text |

## Installation locations

- **ponytail:** installed into `~/.gemini/config/plugins/ponytail/` via `agy plugin install`.
- **i-have-adhd:** installed into `~/.gemini/config/plugins/i-have-adhd/` via `agy plugin install`.
- **humanizer:** installed as a global skill at `~/.gemini/config/skills/humanizer/SKILL.md`.

A companion you already had installed is reused, not installed a second time. No companion's version is
pinned: you get each one's latest.

On uninstall, removing tightship does not delete companions you wish to keep. See [uninstall](uninstall.md).
