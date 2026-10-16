# Update

English · [Português](update.pt-BR.md)

Antigravity CLI updates tightship from GitHub Releases. Your options in `options.toml` keep their values
across an update.

## Using the installer

Run the installer with `--yes` to fetch and install the latest release without changing your existing options:

On Linux and macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness antigravity-cli --yes
```

On Windows in PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness antigravity-cli -Yes
```

## Updating by hand

Download `tightship-antigravity-cli.zip` from the latest release, extract it, and reinstall:

```sh
mkdir -p /tmp/tightship && cd /tmp/tightship
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/tightship-antigravity-cli.zip \
  -o tightship.zip
unzip -q tightship.zip -d extracted
agy plugin install extracted
```

## Updating companions

To update the companion plugins:

```sh
agy plugin update ponytail
agy plugin update i-have-adhd
curl -fsSL https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md \
  -o ~/.gemini/config/skills/humanizer/SKILL.md
```

The new versions load in your next session.
