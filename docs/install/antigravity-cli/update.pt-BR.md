# Atualização

[English](update.md) · Português

O Antigravity CLI atualiza o tightship a partir dos lançamentos no GitHub. As suas opções no `options.toml`
mantêm seus valores durante uma atualização.

## Usando o instalador

Execute o instalador com `--yes` para baixar e instalar a versão mais recente sem alterar suas opções:

No Linux e macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness antigravity-cli --yes
```

No Windows usando o PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness antigravity-cli -Yes
```

## Atualização manual

Baixe o `tightship-antigravity-cli.zip` do release mais recente, extraia e reinstale:

```sh
mkdir -p /tmp/tightship && cd /tmp/tightship
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/tightship-antigravity-cli.zip \
  -o tightship.zip
unzip -q tightship.zip -d extracted
agy plugin install extracted
rm -f ~/.gemini/config/plugins/tightship/hooks.windows.json
```

No Git Bash do Windows, troque a última linha pelos hooks do Windows:

```sh
mv -f ~/.gemini/config/plugins/tightship/hooks.windows.json ~/.gemini/config/plugins/tightship/hooks.json
```

## Atualização dos companheiros

Para atualizar os plugins companheiros:

```sh
agy plugin update ponytail
agy plugin update i-have-adhd
curl -fsSL https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md \
  -o ~/.gemini/config/skills/humanizer/SKILL.md
```

As novas versões passam a valer na próxima sessão.
