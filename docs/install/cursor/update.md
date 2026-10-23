# Update

English · [Português](update.pt-BR.md)

tightship updates itself from GitHub Releases, in the Cursor editor and in the `agent` CLI alike. Your
options in `~/.cursor/tightship/options.json` keep their values across an update.

## Automatically

With the `auto_update` option on, which is its default, each session start checks for a new release, at most
once a day, without making the session wait. A newer release is downloaded in the background, its sha256
checked against the one the release published, and put in place in the plugin's folder. The next session
tells you, once: "tightship was updated from 1.1.0 to 1.2.0."

A download whose sha256 does not match is not installed, and the next session says so. A network failure
changes nothing and is tried again the next day.

To check now, run the updater yourself, with `PLUGIN` set as in [the options](options.md#the-tightship-binary):

```sh
"$PLUGIN/bin/tightship" update
```

To turn it off:

```sh
"$PLUGIN/bin/tightship" options set auto_update false
```

## Using the installer

Rerunning the installer replaces tightship and its companions with their latest versions, and keeps your
options:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness cursor --yes
```

On Windows in PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness cursor -Yes
```

The companions are updated only this way.

## By hand

Download `tightship-cursor.zip` from the latest release, extract it, and replace the contents of
`~/.cursor/plugins/local/tightship/` with it. Reload the editor window.
