# Install tightship in Cursor

English · [Português](README.pt-BR.md)

How to install, configure, and remove the tightship plugin in the Cursor editor and in the `agent` command-line
agent.

## Before you start

- Cursor editor **3.25.6** or later.
- The `agent` CLI **`2026.10.14`** or later. Check with `agent --version`.
- On Windows, PowerShell 7 for hook commands.
- Git, when memory is on, since the vault is a git repository.

## Pages

| Page | What it covers |
| --- | --- |
| [The companions](companions.md) | The three plugins to install before tightship |
| [Installation](install.md) | The unified installer and a manual local-folder path |
| [The options](options.md) | `options.json`, the Configure dialog, and the `tightship options` commands |
| [Uninstall](uninstall.md) | Removing tightship, your options folder, and optional companion cleanup |

## Try it from a clone

Maintainers and contributors who build from source can load the plugin without the marketplace:

```sh
cargo xtask build
agent --plugin-dir dist/cursor
```

Or copy `dist/cursor/` to `~/.cursor/plugins/local/tightship/`, reload the editor window, and use the CLI
against that folder. Adopters should follow [Installation](install.md) instead.
