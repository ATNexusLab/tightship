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
rm -rf ~/.cursor/plugins/local/tightship
rm -rf ~/.cursor/tightship
```

No Windows no PowerShell:

```powershell
Remove-Item -Recurse -Force "$HOME\.cursor\plugins\local\tightship"
Remove-Item -Recurse -Force "$HOME\.cursor\tightship"
```

Remover o plugin em Customize → Plugins equivale quando ele está instalado como **Local**.

## 3. Remova os companheiros, se quiser

Você instalou ponytail, i-have-adhd e humanizer separadamente em [os companheiros](companions.pt-BR.md).
Remova-os só quando não os usar mais com outros plugins:

```sh
rm -rf ~/.cursor/plugins/local/ponytail
rm -rf ~/.cursor/plugins/local/i-have-adhd
rm -rf ~/.cursor/plugins/local/humanizer
```

O seu vault nunca é apagado.
