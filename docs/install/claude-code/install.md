# Installation

English · [Português](install.pt-BR.md)

Install tightship with the one-command installer, or add the catalog and install by hand.

## One-command installer

The installer detects your CLIs, asks for your options, adds tightship's catalog, installs the
plugin, and applies your options. Nothing is written until you choose Install on the summary.

On Linux and macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness claude-code
```

On Windows in PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness claude-code
```

From `cmd`:

```bat
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) -Harness claude-code"
```

Without `--harness`, the first question lists every CLI it found, so one run can install Cursor and
Antigravity CLI as well.

## Manual installation

Add tightship's catalog, which is this repository, then install:

```sh
claude plugin marketplace add ATNexusLab/tightship
claude plugin install tightship@tightship
```

```text
✔ Successfully installed plugin: tightship@tightship (scope: user)
```

The install asks for no option from the shell. Set them as you install with `--config`, or in the next
session's dialog: see [the options](options.md).

## Check it

Run `claude plugin list`. `tightship@tightship` shows with no error.
