# Installation

English · [Português](install.pt-BR.md)

Complete [The companions](companions.md) first.

Do not run a workflow command until ponytail, i-have-adhd, and humanizer each show **Installed** under
Customize → Plugins.

## Before you start

- Cursor **3.25.6** or later in the editor. Check with **Help → About** in the editor.
- The Cursor CLI **`agent` 2026.10.14** or later when you use the command-line agent. Check with
  `agent --version`.
- Git, when memory is on, since the vault is a git repository.

Do not install tightship in Claude Code and in Cursor on the same machine. Cursor imports Claude Code plugins
as **Imported**, and the same pieces would load twice.

## Add the tightship catalog

If you followed [The companions](companions.md), the four catalogs are already added. Otherwise:

```sh
agent plugin marketplace add DietrichGebert/ponytail
agent plugin marketplace add ayghri/i-have-adhd
agent plugin marketplace add blader/humanizer
agent plugin marketplace add ATNexusLab/tightship
```

## Install tightship

With all three companions already **Installed**:

```sh
agent plugin install tightship@tightship
```

You can install from Customize → Plugins in the editor instead of the command above.

## Check it

Under Customize → Plugins, ponytail, i-have-adhd, humanizer, and tightship each show **Installed**, and
tightship's skills and commands appear there.
