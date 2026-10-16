# Installation

English · [Português](install.pt-BR.md)

Complete [The companions](companions.md) first.

Do not run a workflow command until ponytail, i-have-adhd, and humanizer each show **Installed** under
Customize → Plugins.

Do not install tightship as a Claude Code plugin on the same machine. Cursor can load Claude Code plugins as
**Imported**, and the same pieces would load twice.

With the four catalogs already added (companions first, then this repository), install tightship:

```sh
agent plugin install tightship@tightship
```

In the editor, you can install from Customize → Plugins after the catalogs are present.

The install asks for no option from the shell. Hooks and the binary read options only from
`~/.cursor/tightship/options.json`, not from the values shown in Customize → Configure. Set options with
[tightship options](options.md) after install.

## Check it

Under Customize → Plugins, tightship shows **Installed** with its skills and commands listed. ponytail,
i-have-adhd, and humanizer are still installed from the companions step.

In an interactive `agent` session, ask which slash commands are available. The answer should list `/spec` and
the other workflow commands.

After a local `cargo xtask build`, you can confirm the same with `agent --plugin-dir dist/cursor` instead of
the marketplace install.
