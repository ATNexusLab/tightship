# Instalação

[English](install.md) · Português

Instale o tightship usando o instalador unificado interativo, ou execute cada passo manualmente.

## Instalador de um comando

Execute o instalador diretamente a partir dos lançamentos no GitHub. Ele detecta seus CLIs instalados,
pergunta pelas suas opções, instala o tightship, grava seu `options.toml` e configura a sua status line.

No Linux e macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | sh
```

No Windows usando o PowerShell:

```powershell
irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1 | iex
```

No `cmd`:

```bat
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1 | iex"
```

Cada pergunta já começa no padrão, então apertar Enter em todas instala com os padrões, inclusive o
vault em `~/ai-memory`; um `-` na pergunta do vault o deixa para depois. A primeira lista os
CLIs encontrados, já marcados: mova com ↑ e ↓, marque ou desmarque com espaço e confirme com Enter. Numa
pergunta de sim ou não, uma seta troca a resposta. Nada é gravado antes de você escolher Install no resumo:

```text
+  tightship v1.3.0
|
|  Harnesses
*  Where should tightship be installed?
|  Antigravity CLI
|
|  Options
*  Load the working contract in every session?
|  Yes
|
*  Turn on the memory layer?
|  Yes
|
*  Show tightship's status line?
|  Yes
|
*  Keep tightship updated automatically?
|  Yes
|
|  Vault
*  Where is your vault?
|  ~/ai-memory
|
*  ~/ai-memory does not exist. Create it now?
|  Yes
|
*  Summary
|  Harnesses       Antigravity CLI
|  Contract        on
|  Memory          on
|  Vault           ~/ai-memory, created now
|  Status line     on
|  Auto-update     on
|
?  Install with these choices?
|  (*) Install / ( ) Cancel
+
```

Para executar de forma não interativa, passe opções pelas flags:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness antigravity-cli --vault-folder ~/ai-memory --yes
```

## Instalação manual

Se preferir executar cada passo manualmente:

### 1. Baixe e instale o tightship

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

### 2. Escreva suas opções

Crie o arquivo `~/.gemini/config/plugin_data/tightship/options.toml`:

```toml
contract = true
memory = true
status_line = true
auto_update = true
vault_folder = "~/ai-memory"
```

### 3. Ative a status line

```sh
~/.gemini/config/plugins/tightship/bin/tightship hook status-line
```

## Conferência

Execute `agy plugin list`. `tightship` aparece como instalado.

## Superfícies

Quais peças carregam em cada superfície está na [matriz de suporte](../README.pt-BR.md).

**CLI.** O instalador ou `agy plugin install`, como acima.

**IDE.** Ela lê `~/.gemini/config/plugins/`, então a instalação da CLI chega a ela sem passo além dos de
cima. Todas as peças menos a status line carregam, checadas em uso: o Antigravity desenha `statusLine` só
na CLI.

**App desktop.** Igual à IDE: ele lê `~/.gemini/config/plugins/`, e todas as peças menos a status line
carregam, checadas em uso.

O terminal do Zed é um terminal comum. Rode `agy` nele e cada peça funciona como em qualquer outro
terminal, sem passo além deste guia.

O agent panel do Zed, que roda o Antigravity CLI pela entrada `antigravity-acp` do registro de agentes do
Zed, ainda não foi conferido ([#163](https://github.com/ATNexusLab/tightship/issues/163)). Até ser, rode
o Antigravity CLI no terminal.
