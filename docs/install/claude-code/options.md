# The options

English · [Português](options.pt-BR.md)

Claude Code keeps tightship's options in your user settings. A new version keeps the values you set, and the
uninstall deletes them.

| Option | Type | What it does | When not set |
| --- | --- | --- | --- |
| `contract` | on/off | Loads the workflow's rules into every session. Off: see [without the contract](without-the-contract.md) | On |
| `memory` | on/off | Keeps curated notes in your vault and hands session state to the next session | On |
| `status_line` | on/off | Shows tightship's status line under the prompt, and puts yours back when turned off | On |
| `auto_update` | on/off | Turns on Claude Code's auto-update for tightship's catalog, so each new release installs by itself. See [update](update.md) | On |
| `vault_folder` | folder | Your memory vault, a git repository, as an absolute path. Memory needs it | No vault: memory has nothing to read or write |

## Set them as you install

From the shell, pass one `--config` per option:

```sh
claude plugin install tightship@tightship --config memory=true --config vault_folder=/home/you/vault
```

From a session, `/plugin install tightship@tightship` installs and then opens a dialog for every option not
set yet. An install from the shell without `--config` reports `5 userConfig options not yet set` and leaves
each on/off option on.

## Change them later

- In a session: `/plugin configure tightship@tightship` opens the same dialog.
- From the shell: `claude plugin configure tightship@tightship` lists each option and whether it is set.
  To change some, pipe a JSON object to it; the options you leave out keep their values:

  ```sh
  echo '{"memory":"false"}' | claude plugin configure tightship@tightship --values-stdin
  ```

A change applies from the next session.

## The vault

Memory needs a vault, a git repository of notes. If you have none, ask the agent in a session to run
`tightship vault init <folder>`; it creates one from the template. Set that folder as `vault_folder` and start
a new session. To share the vault across machines, push it to a private remote and clone it on each.

With memory on and no `vault_folder`, the session has no vault, and the agent tells you how to create one
when you ask about memory.

## The status line

With `status_line` on, tightship shows its status line under the prompt from the next session start. It
replaces the `statusLine` of your user settings, whatever it held, and keeps yours aside.

Turning `status_line` off puts your previous status line back at the next session start, or removes the key
when you had none.
