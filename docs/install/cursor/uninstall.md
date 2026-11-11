# Uninstall

English · [Português](uninstall.pt-BR.md)

## 1. Put your CLI status line back

Skip this step if you never turned tightship's status line on in the CLI.

Set `PLUGIN` to your tightship install folder (see [the options](options.md#the-tightship-binary)), then:

```sh
export TIGHTSHIP_SETTINGS="$HOME/.cursor/cli-config.json"
export TIGHTSHIP_DATA="$HOME/.cursor/tightship"
export TIGHTSHIP_ROOT="$PLUGIN"
TIGHTSHIP="$PLUGIN/bin/tightship"

"$TIGHTSHIP" options set status_line false
"$TIGHTSHIP" hook status-line
```

That restores your previous `statusLine` in `~/.cursor/cli-config.json`, or removes the key when you had none.

## 2. Remove tightship

```sh
rm -rf ~/.cursor/plugins/local/tightship
rm -rf ~/.cursor/tightship
```

On Windows in PowerShell:

```powershell
Remove-Item -Recurse -Force "$HOME\.cursor\plugins\local\tightship"
Remove-Item -Recurse -Force "$HOME\.cursor\tightship"
```

Removing the plugin under Customize → Plugins is equivalent when it is installed as **Local**.

Your vault is never touched.
