# Instalação

[English](install.md) · Português

Instale o tightship usando o instalador unificado interativo, ou execute cada passo manualmente.

## Instalador de um comando

Execute o instalador diretamente a partir dos lançamentos no GitHub. Ele detecta seus CLIs instalados,
pergunta pelas suas opções, instala os companheiros e o tightship, grava seu `options.toml` e configura a sua
status line.

No Linux e macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | sh
```

No Windows usando o PowerShell:

```powershell
irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1 | iex
```

Cada pergunta já começa no padrão, então apertar Enter em todas instala com os padrões. A primeira lista os
CLIs encontrados, já marcados: mova com ↑ e ↓, marque ou desmarque com espaço e confirme com Enter. Numa
pergunta de sim ou não, uma seta troca a resposta. Nada é gravado antes de você escolher Install no resumo:

```text
┌  tightship v1.2.0
│
│  Harnesses
◇  Where should tightship be installed?
│  Antigravity CLI
│
│  Options
◇  Load the working contract in every session?
│  Yes
│
◇  Turn on the memory layer?
│  Yes
│
◇  Show tightship's status line?
│  Yes
│
◇  Keep tightship updated automatically?
│  Yes
│
│  Vault
◇  Where is your vault?
│  not set
│
◇  Summary
│  Harnesses       Antigravity CLI
│  Contract        on
│  Memory          on
│  Vault           not set
│  Status line     on
│  Auto-update     on
│
◆  Install with these choices?
│  ● Install / ○ Cancel
└
```

Para executar de forma não interativa, passe opções pelas flags:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness antigravity-cli --vault-folder ~/ai-memory --yes
```

## Instalação manual

Se preferir executar cada passo manualmente:

### 1. Instale os companheiros

```sh
agy plugin install https://github.com/DietrichGebert/ponytail
agy plugin install https://github.com/ayghri/i-have-adhd
mkdir -p ~/.gemini/config/skills/humanizer
curl -fsSL https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md \
  -o ~/.gemini/config/skills/humanizer/SKILL.md
```

### 2. Baixe e instale o tightship

Baixe o `tightship-antigravity-cli.zip` do release mais recente, extraia e instale:

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

### 3. Escreva suas opções

Crie o arquivo `~/.gemini/config/plugin_data/tightship/options.toml`:

```toml
contract = true
memory = true
status_line = true
auto_update = true
vault_folder = "~/ai-memory"
```

### 4. Ative a status line

```sh
~/.gemini/config/plugins/tightship/bin/tightship hook status-line
```

## Conferência

Execute `agy plugin list`. `tightship`, `ponytail` e `i-have-adhd` aparecem como instalados.
