# Glossary

| Term | Meaning |
| --- | --- |
| Adopter | A person who installs the plugin in a harness. |
| Baseline | Commit `1f41699` of the maintainer's private configuration, the pre-plugin workflow every parity requirement refers to. |
| Canonical content | The single authored copy of every workflow piece in this repository, from which each harness's plugin is built. |
| Checkpoint | A reminder, placed in the agent's context, to record in the vault what the session learned. |
| Command | A workflow step the adopter invokes by name, such as the one that writes a spec. |
| Companion plugins | ponytail, i-have-adhd, and humanizer: third-party plugins the workflow requires and whose rules it never restates. |
| Contract | The always-on text that tells the agent how to work in every session. |
| Handoff | The one vault note per project that carries a session's state to the next session, on any machine or harness. |
| Harness | An agent tool that runs a coding agent: Claude Code, Cursor, Antigravity CLI, or Codex. |
| Option | A part of the plugin the adopter turns on or off: the contract, the memory layer, or the status line. |
| Per-harness place | The folder of the repository where one harness's names, paths, and tools may appear: `harnesses/<harness>/`. |
| Pinned note | A vault note marked to be listed at every session start. |
| Pre-plugin install | The workflow installed by merging files into a harness's user directory, as the baseline does. |
| Scope | The vault folder a note belongs to: one project's name, or `all`. |
| Skill | Instructions the agent loads on demand when a task matches its description. |
| Status line | The line a harness shows under the prompt with the session's state. |
| tightship | The plugin's name. Harnesses that namespace plugin pieces prefix each command with it, as in `/tightship:spec`. |
| Vault | A git repository of curated notes shared across machines and harnesses. |
| Verifier | A read-only subagent that tries to refute a finding, a fix, or a claim that work is done. |
