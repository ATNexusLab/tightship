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
```

### 3. Escreva suas opções

Crie o arquivo `~/.gemini/config/plugin_data/tightship/options.toml`:

```toml
contract = true
memory = true
status_line = true
vault_folder = "~/ai-memory"
```

### 4. Ative a status line

```sh
~/.gemini/config/plugins/tightship/bin/tightship hook status-line
```

## Conferência

Execute `agy plugin list`. `tightship`, `ponytail` e `i-have-adhd` aparecem como instalados.
