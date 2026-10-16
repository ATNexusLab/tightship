# Uninstall

English · [Português](uninstall.pt-BR.md)

## 1. Put your status line back

Skip this step if you never turned tightship's status line on.

Before deleting the plugin, restore your previous status line so your `settings.json` is returned to its
prior state. Set `status_line = false` in `~/.gemini/config/plugin_data/tightship/options.toml` and run:

```sh
~/.gemini/config/plugins/tightship/bin/tightship hook status-line
```

This restores your previous `statusLine` in `~/.gemini/antigravity-cli/settings.json`, or removes the key if
you had none.

## 2. Remove the plugin

```sh
agy plugin uninstall tightship
rm -rf ~/.gemini/config/plugin_data/tightship
```

On Windows in PowerShell:

```powershell
agy plugin uninstall tightship
Remove-Item -Recurse -Force "$HOME\.gemini\config\plugin_data\tightship"
```

## 3. Optionally remove companions

If you no longer use the companions with other plugins:

```sh
agy plugin uninstall ponytail
agy plugin uninstall i-have-adhd
rm -rf ~/.gemini/config/skills/humanizer
```

Your vault is never touched.
