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
agent plugin uninstall tightship@tightship
rm -rf ~/.cursor/tightship
```

On Windows in PowerShell:

```powershell
agent plugin uninstall tightship@tightship
Remove-Item -Recurse -Force "$HOME\.cursor\tightship"
```

Uninstalling from the editor under Customize → Plugins is equivalent to the `agent plugin uninstall` step.

## 3. Optionally remove companions

You installed ponytail, i-have-adhd, and humanizer separately in [The companions](companions.md). Remove them
only when you no longer use them with other plugins:

```sh
agent plugin uninstall ponytail@ponytail
agent plugin uninstall i-have-adhd@i-have-adhd
agent plugin uninstall humanizer@humanizer
```

## 4. Remove the catalogs

```sh
agent plugin marketplace remove tightship
agent plugin marketplace remove ponytail
agent plugin marketplace remove i-have-adhd
agent plugin marketplace remove humanizer
```

Keep a companion's catalog if you still use that companion.

Your vault is never touched.
