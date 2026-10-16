# The companions

English · [Português](companions.pt-BR.md)

tightship is used with three third-party plugins. Cursor's plugin manifest has no `dependencies` field, so
the editor and the CLI do not install companions for you. Add each catalog, install each companion, and
confirm it shows **Installed** under Customize → Plugins before you follow
[Installation](install.md).

| Companion | Catalog | What it brings |
| --- | --- | --- |
| ponytail | `DietrichGebert/ponytail` | Pushes the agent toward the simplest code that works: no speculative abstraction, the standard library first |
| i-have-adhd | `ayghri/i-have-adhd` | Shapes answers to be acted on: the next action first, numbered steps, no preamble |
| humanizer | `blader/humanizer` | Rewrites prose meant for people so it does not read as generated text |

A companion you already had installed is reused, not installed a second time. No companion's version is
pinned: you get each one's latest.

## Add the catalogs

Run once per repository, companions first, then tightship when you reach [Installation](install.md):

```sh
agent plugin marketplace add DietrichGebert/ponytail
agent plugin marketplace add ayghri/i-have-adhd
agent plugin marketplace add blader/humanizer
agent plugin marketplace add ATNexusLab/tightship
```

In the editor, you can add the same git URLs from Customize → Plugins → marketplaces instead of the commands
above.

## Install the three companions

After the catalogs are added, install only the companions (not tightship yet):

```sh
agent plugin install ponytail@ponytail
agent plugin install i-have-adhd@i-have-adhd
agent plugin install humanizer@humanizer
```

If a catalog is missing, the install fails or the plugin does not appear. Add the missing catalog and run the
install again.

## Check it

Under Customize → Plugins, ponytail, i-have-adhd, and humanizer each show **Installed**. Only then continue
with [Installation](install.md).
