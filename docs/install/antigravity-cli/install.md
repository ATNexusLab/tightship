# Installation

English · [Português](install.pt-BR.md)

Install tightship using the interactive one-command installer, or run each step by hand.

## One-command installer

Run the installer directly from GitHub Releases. It detects your installed CLIs, prompts for your options,
installs the companions and tightship, writes your `options.toml`, and configures your status line.

On Linux and macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | sh
```

On Windows in PowerShell:

```powershell
irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1 | iex
```

Every question starts on its default, so pressing Enter through all of them installs with the defaults. The
first question lists the CLIs it found, already checked: move with ↑ and ↓, check or uncheck one with space,
and confirm with Enter. On a yes or no question, an arrow switches the answer. Nothing is written until you
choose Install on the summary:

```text
┌  tightship v1.1.1
│
│  Harnesses
◇  Where should tightship be installed?
│  Antigravity CLI
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
│  Harnesses       Antigravity CLI
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
  sh -s -- --harness antigravity-cli --vault-folder ~/ai-memory --yes
```

## Manual installation

If you prefer to run each step by hand:

### 1. Install the companions

```sh
agy plugin install https://github.com/DietrichGebert/ponytail
agy plugin install https://github.com/ayghri/i-have-adhd
mkdir -p ~/.gemini/config/skills/humanizer
curl -fsSL https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md \
  -o ~/.gemini/config/skills/humanizer/SKILL.md
```

### 2. Download and install tightship

Download `tightship-antigravity-cli.zip` from the latest release, extract it, and install:

```sh
mkdir -p /tmp/tightship && cd /tmp/tightship
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/tightship-antigravity-cli.zip \
  -o tightship.zip
unzip -q tightship.zip -d extracted
agy plugin install extracted
rm -f ~/.gemini/config/plugins/tightship/hooks.windows.json
```

In Git Bash on Windows, replace the last line with the Windows hooks:

```sh
mv -f ~/.gemini/config/plugins/tightship/hooks.windows.json ~/.gemini/config/plugins/tightship/hooks.json
```

### 3. Write your options

Create `~/.gemini/config/plugin_data/tightship/options.toml`:

```toml
contract = true
memory = true
status_line = true
vault_folder = "~/ai-memory"
```

### 4. Enable the status line

```sh
~/.gemini/config/plugins/tightship/bin/tightship hook status-line
```

## Check it

Run `agy plugin list`. `tightship`, `ponytail`, and `i-have-adhd` show as installed.
