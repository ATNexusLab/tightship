# Desinstalação

[English](uninstall.md) · Português

## 1. Devolva a status line do CLI

Pule esta etapa se você nunca ligou a status line do tightship no CLI.

Defina `PLUGIN` para a pasta de instalação do tightship (veja [as opções](options.pt-BR.md#o-programa-tightship)),
depois:

```sh
export TIGHTSHIP_SETTINGS="$HOME/.cursor/cli-config.json"
export TIGHTSHIP_DATA="$HOME/.cursor/tightship"
export TIGHTSHIP_ROOT="$PLUGIN"
TIGHTSHIP="$PLUGIN/bin/tightship"

"$TIGHTSHIP" options set status_line false
"$TIGHTSHIP" hook status-line
```

Isso devolve o `statusLine` anterior em `~/.cursor/cli-config.json`, ou remove a chave se você não tinha
nenhuma.

## 2. Remova o tightship

```sh
agent plugin uninstall tightship@tightship
rm -rf ~/.cursor/tightship
```

No Windows no PowerShell:

```powershell
agent plugin uninstall tightship@tightship
Remove-Item -Recurse -Force "$HOME\.cursor\tightship"
```

Desinstalar pelo editor em Customize → Plugins equivale ao passo `agent plugin uninstall`.

## 3. Remova os companheiros, se quiser

Você instalou ponytail, i-have-adhd e humanizer separadamente em [os companheiros](companions.pt-BR.md).
Remova-os só quando não os usar mais com outros plugins:

```sh
agent plugin uninstall ponytail@ponytail
agent plugin uninstall i-have-adhd@i-have-adhd
agent plugin uninstall humanizer@humanizer
```

## 4. Remova os catálogos

```sh
agent plugin marketplace remove tightship
agent plugin marketplace remove ponytail
agent plugin marketplace remove i-have-adhd
agent plugin marketplace remove humanizer
```

Mantenha o catálogo de um companheiro se você ainda usa esse companheiro.

O seu vault nunca é apagado.
