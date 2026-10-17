# The companions

English · [Português](companions.pt-BR.md)

tightship is used with three third-party plugins. Cursor's plugin manifest has no `dependencies` field, so
the editor and the CLI do not install companions for you.

| Companion | What it brings |
| --- | --- |
| ponytail | Pushes the agent toward the simplest code that works: no speculative abstraction, the standard library first |
| i-have-adhd | Shapes answers to be acted on: the next action first, numbered steps, no preamble |
| humanizer | Rewrites prose meant for people so it does not read as generated text |

ponytail and i-have-adhd ship only `.claude-plugin/`; Cursor loads them as **Imported** when they sit under
`~/.cursor/plugins/local/`. humanizer also ships `.cursor-plugin/`.

## With the unified installer

[Installation](install.md) installs all three companions into `~/.cursor/plugins/local/<name>/` before it
installs tightship. You do not add marketplaces or run separate install commands.

## Manual installation

Download each companion's repository from GitHub, extract it, and copy the tree into
`~/.cursor/plugins/local/<name>/` (for example `~/.cursor/plugins/local/ponytail/`).

Confirm each one shows **Local** under Customize → Plugins before you install tightship.

A companion you already had installed is reused, not installed a second time. No companion's version is
pinned: you get each one's latest default branch.

## Check it

Under Customize → Plugins, ponytail, i-have-adhd, and humanizer each show **Local**. Only then continue with
[Installation](install.md).
