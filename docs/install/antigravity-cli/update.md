# Update

English · [Português](update.pt-BR.md)

tightship updates itself from GitHub Releases. Your options in `options.toml` keep their values across an
update.

## Automatically

With the `auto_update` option on, which is its default, each session start checks for a new release, at most
once a day, without making the session wait. A newer release is downloaded in the background, its sha256
checked against the one the release published, and put in place. The next session tells you, once:
"tightship was updated from 1.1.0 to 1.2.0."

A download whose sha256 does not match is not installed, and the next session says so. A network failure
changes nothing and is tried again the next day.

To check now, run the updater yourself:

```sh
~/.gemini/config/plugins/tightship/bin/tightship update
```

To turn it off, set `auto_update = false` in `options.toml`, or rerun the installer with
`--auto-update false`.

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

The installer also reinstalls the companions from their repositories, which is how they get updated.

## Updating by hand

Download `tightship-antigravity-cli.zip` from the latest release, extract it, and reinstall:

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

## Updating companions

Antigravity CLI has no update command for a plugin outside its official catalog, so reinstall each companion
from its repository:

```sh
agy plugin install https://github.com/DietrichGebert/ponytail
agy plugin install https://github.com/ayghri/i-have-adhd
curl -fsSL https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md \
  -o ~/.gemini/config/skills/humanizer/SKILL.md
```

The new versions load in your next session.
