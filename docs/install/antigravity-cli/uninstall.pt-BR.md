# Desinstalação

[English](uninstall.md) · Português

## 1. Restaure a sua status line

Pule esta etapa se você nunca ativou a status line do tightship.

Antes de remover o plugin, restaure sua status line anterior para que seu arquivo `settings.json` retorne ao
estado prévio. Defina `status_line = false` em `~/.gemini/config/plugin_data/tightship/options.toml` e execute:

```sh
~/.gemini/config/plugins/tightship/bin/tightship hook status-line
```

Isso recupera sua `statusLine` anterior em `~/.gemini/antigravity-cli/settings.json`, ou remove a chave caso
você não possuísse nenhuma anteriormente.

## 2. Remova o plugin

```sh
agy plugin uninstall tightship
rm -rf ~/.gemini/config/plugin_data/tightship
```

No Windows usando o PowerShell:

```powershell
agy plugin uninstall tightship
Remove-Item -Recurse -Force "$HOME\.gemini\config\plugin_data\tightship"
```

## 3. Opcionalmente remova os companheiros

Caso não utilize mais os companheiros em outros fluxos:

```sh
agy plugin uninstall ponytail
agy plugin uninstall i-have-adhd
rm -rf ~/.gemini/config/skills/humanizer
```

O seu vault nunca é tocado.
