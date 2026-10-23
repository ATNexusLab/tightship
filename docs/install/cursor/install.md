# Installation

English · [Português](install.pt-BR.md)

Install tightship using the interactive one-command installer, or run each step by hand.

Do not install tightship as a Claude Code plugin on the same machine. Cursor can load Claude Code plugins as
**Imported**, and the same pieces would load twice.

## One-command installer

Run the installer directly from GitHub Releases. It detects your installed CLIs, prompts for your options,
installs the companions and tightship into `~/.cursor/plugins/local/`, writes your `options.json`, and
configures your CLI status line.

On Linux and macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness cursor
```

On Windows in PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness cursor -Yes
```

Every question starts on its default, so pressing Enter through all of them installs with the defaults. On a
yes or no question, an arrow switches the answer. Nothing is written until you choose Install on the summary:

```text
┌  tightship v1.1.1
│
│  Options
◇  Load the working contract in every session?
│  Yes
│
◇  Turn on the memory layer?
│  Yes
│
◇  Show tightship's status line?
│  Yes
│
│  Vault
◇  Where is your vault?
│  not set
│
◇  Summary
│  Harnesses       Cursor
│  Contract        on
│  Memory          on
│  Vault           not set
│  Status line     on
│
◆  Install with these choices?
│  ● Install / ○ Cancel
└
```

To run unattended, pass options via flags:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness cursor --vault-folder ~/ai-memory --yes
```

Some Cursor accounts block local plugin folders (`userLocal=false`). The installer cannot change that policy;
ask your administrator to allow local plugin imports, or use a personal Cursor account.

## Manual installation

If you prefer to run each step by hand, follow [The companions](companions.md) first, then:

### 1. Download and install tightship

Download `tightship-cursor.zip` from the latest release, extract it, and copy the contents to
`~/.cursor/plugins/local/tightship/`.

### 2. Write your options

Create `~/.cursor/tightship/options.json`:

```json
{
  "contract": true,
  "memory": true,
  "status_line": true,
  "vault_folder": "~/ai-memory"
}
```

Or use [tightship options](options.md).

### 3. Enable the CLI status line

```sh
export TIGHTSHIP_DATA="$HOME/.cursor/tightship"
export TIGHTSHIP_SETTINGS="$HOME/.cursor/cli-config.json"
export TIGHTSHIP_ROOT="$HOME/.cursor/plugins/local/tightship"
"$TIGHTSHIP_ROOT/bin/tightship" hook status-line
```

Reload the editor window after copying the plugin folder.

## Check it

Under Customize → Plugins, tightship shows as **Local** with its skills and commands listed. ponytail,
i-have-adhd, and humanizer are installed under **Local** as well.

In an interactive `agent` session, ask which slash commands are available. The answer should list `/spec` and
the other workflow commands.

After a local `cargo xtask build`, you can confirm the same with `agent --plugin-dir dist/cursor` instead of
the release zip.
