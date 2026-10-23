# Update

English · [Português](update.pt-BR.md)

Claude Code finds a new version through the catalog you added from the release URL. Your options keep their
values across an update.

## Automatically

With the `auto_update` option on, which is its default, tightship turns on Claude Code's auto-update for its
catalog at each session start. Claude Code then installs each new release by itself, and the session after
an update loads it.

Turn the option off to stop it; the next session start turns the catalog's auto-update off again:

```sh
echo '{"auto_update":"false"}' | claude plugin configure tightship@tightship --values-stdin
```

## Using the installer

Rerunning the installer updates tightship and its three companions, and keeps the options you chose before:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness claude-code --yes
```

On Windows in PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness claude-code -Yes
```

## By hand

```sh
claude plugin marketplace update tightship
claude plugin update tightship@tightship
```

The new version loads in the next session.
