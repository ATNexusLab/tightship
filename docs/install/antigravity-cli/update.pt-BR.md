# Atualização

[English](update.md) · Português

O tightship se atualiza sozinho a partir dos lançamentos no GitHub. As suas opções no `options.toml` mantêm
seus valores durante uma atualização.

## Automática

Com a opção `auto_update` ligada, que é o padrão, cada início de sessão procura uma release nova, no máximo
uma vez por dia, sem fazer a sessão esperar. Uma release mais nova é baixada em segundo plano, tem o sha256
conferido com o que a release publicou e é colocada no lugar. A sessão seguinte avisa, uma única vez:
"tightship was updated from 1.1.0 to 1.2.0."

Um download cujo sha256 não confere não é instalado, e a sessão seguinte avisa. Uma falha de rede não muda
nada e é tentada de novo no dia seguinte.

Para procurar agora, rode o atualizador você mesmo:

```sh
~/.gemini/config/plugins/tightship/bin/tightship update
```

Para desligar, ponha `auto_update = false` no `options.toml`, ou rode o instalador de novo com
`--auto-update false`.

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

O instalador também reinstala os companheiros a partir dos repositórios deles, e é assim que eles se
atualizam.

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

O Antigravity CLI não tem comando de atualização para um plugin de fora do seu catálogo oficial, então
reinstale cada companheiro a partir do repositório dele:

```sh
agy plugin install https://github.com/DietrichGebert/ponytail
agy plugin install https://github.com/ayghri/i-have-adhd
curl -fsSL https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md \
  -o ~/.gemini/config/skills/humanizer/SKILL.md
```

As novas versões passam a valer na próxima sessão.
