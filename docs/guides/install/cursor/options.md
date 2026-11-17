# The options

English · [Português](options.pt-BR.md)

Cursor shows tightship's options in Customize → Configure from the manifest `variables`. Hooks and the
`tightship` binary ignore that dialog and read state only from `$HOME/.cursor/tightship/options.json`.

| Option | Type | What it does | When not set |
| --- | --- | --- | --- |
| `contract` | on/off | Loads the workflow's rules into every session. Off: contract rules are not injected | On |
| `memory` | on/off | Keeps curated notes in your vault and hands session state to the next session | On |
| `status_line` | on/off | In the **CLI only**, shows tightship's status line under the prompt and restores yours when turned off | On |
| `auto_update` | on/off | Installs each new release by itself and says so at the next session. See [update](update.md) | On |
| `vault_folder` | folder | Your memory vault, a git repository, as an absolute path or `~` path. Memory needs it | No vault: memory has nothing to read or write |

Absent file or absent key means **on** for each boolean option.

## The tightship binary

Commands below call `bin/tightship` inside the folder where Cursor installed the plugin. Set `PLUGIN` to that
folder (Customize → Plugins shows installed plugins; a local try-it copy lives at
`~/.cursor/plugins/local/tightship/`):

```sh
PLUGIN=~/.cursor/plugins/local/tightship   # example — use your install path
TIGHTSHIP="$PLUGIN/bin/tightship"
```

On Windows, use `"$PLUGIN/bin/tightship.cmd"` instead of `"$PLUGIN/bin/tightship"` in the examples.

## Set and list options

```sh
"$TIGHTSHIP" options set memory false
"$TIGHTSHIP" options set vault_folder ~/ai-memory
"$TIGHTSHIP" options list
"$TIGHTSHIP" options reset
```

`set` prints `Set <name> to <value>.` and exit `0`. `list` prints pretty-printed JSON, or `{}` when the file
does not exist. `reset` removes `options.json` and prints `Removed <path>.`, with the file's full path.

A change applies from the next session or the next hook run.

## The vault

Memory needs a vault, a git repository of notes. If you have none, run `"$TIGHTSHIP" vault init <folder>` (or
ask the agent in a session, using the plugin binary path from
[the binary](#the-tightship-binary)). Set that folder with `options set vault_folder <path>` and start a new
session. To share the vault across machines, push it to a private remote and clone it on each.

With memory on and no `vault_folder`, the session has no vault, and the agent tells you how to create one
when you ask about memory.

## The status line

The Cursor editor has no status line API. With `status_line` on, the next interactive **CLI** session writes
tightship's `statusLine` into `~/.cursor/cli-config.json` and keeps yours aside.

Turn the option off to restore your previous `statusLine` at the next session start (or remove the key when
you had none):

```sh
"$TIGHTSHIP" options set status_line false
```

Then start one interactive CLI session so the status-line hook runs, or run `"$TIGHTSHIP" hook status-line`.
