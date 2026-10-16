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
