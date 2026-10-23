# The options

English · [Português](options.pt-BR.md)

Antigravity CLI reads tightship's options from your plugin data folder in `options.toml`. An update keeps the
values you configured, and an uninstall can remove them.

On Linux and macOS, the file is at:
`~/.gemini/config/plugin_data/tightship/options.toml`

On Windows:
`%USERPROFILE%\.gemini\config\plugin_data\tightship\options.toml`

| Option | Type | What it does | When not set |
| --- | --- | --- | --- |
| `contract` | on/off | Loads the workflow's rules into every session. Off: see [without the contract](without-the-contract.md) | On (`true`) |
| `memory` | on/off | Keeps curated notes in your vault and hands session state to the next session | On (`true`) |
| `status_line` | on/off | Shows tightship's status line under the prompt, and puts yours back when turned off | On (`true`) |
| `auto_update` | on/off | Installs each new release by itself and says so at the next session. See [update](update.md) | On (`true`) |
| `vault_folder` | folder | Your memory vault, a git repository, as a path (supports `~`). Memory needs it | No vault: memory has nothing to read or write |

## Environment variable overrides

Environment variables take precedence over `options.toml` when set:

- `TIGHTSHIP_CONTRACT=false` overrides `contract`
- `TIGHTSHIP_MEMORY=false` overrides `memory`
- `TIGHTSHIP_STATUS_LINE=false` overrides `status_line`
- `TIGHTSHIP_AUTO_UPDATE=false` overrides `auto_update`
- `TIGHTSHIP_VAULT=/path/to/vault` overrides `vault_folder`
- `TIGHTSHIP_DATA=/path/to/data` points to a custom directory holding `options.toml`

## Changing options

You can edit `options.toml` directly with any text editor, or re-run `install.sh` / `install.ps1`. Existing
values are preserved as defaults when re-running the installer.

```toml
contract = true
memory = true
status_line = true
auto_update = true
vault_folder = "~/ai-memory"
```

A change takes effect in the next session.

## The vault

Memory needs a vault, a git repository of notes. If you have none, run `tightship vault init <folder>` (or
let the installer create it for you). To share the vault across machines, push it to a private git remote
and clone it on each machine.

With memory on and no `vault_folder`, the session has no vault, and the agent explains how to set one up when
you ask about memory.

## The status line

With `status_line` on, tightship writes its status line command into
`~/.gemini/antigravity-cli/settings.json` and preserves any previous status line configuration in
`status-line.json`.

Turning `status_line` off restores your previous status line at the next session start, or removes the key
if you did not have one previously.
