# Instalação

[English](install.md) · Português

Instale o tightship com o instalador interativo de um comando, ou siga cada passo manualmente.

Não instale o tightship como plugin do Claude Code na mesma máquina. O Cursor pode carregar plugins do Claude
Code como **Imported**, e as mesmas peças seriam carregadas duas vezes.

## Instalador de um comando

Rode o instalador direto dos GitHub Releases. Ele detecta os CLIs instalados, pergunta suas opções, instala
os companheiros e o tightship em `~/.cursor/plugins/local/`, grava seu `options.json` e configura a status
line do CLI.

No Linux e no macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness cursor
```

No Windows no PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness cursor -Yes
```

Cada pergunta já começa no padrão, então apertar Enter em todas instala com os padrões. Numa pergunta de sim
ou não, uma seta troca a resposta. Nada é gravado antes de você escolher Install no resumo:

```text
┌  tightship v1.2.0
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
│  Harnesses       Cursor
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

Para rodar sem prompts, passe as opções por flags:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness cursor --vault-folder ~/ai-memory --yes
```

Algumas contas Cursor bloqueiam pastas locais de plugin (`userLocal=false`). O instalador não altera essa
política; peça ao administrador para permitir importação local de plugins, ou use uma conta Cursor pessoal.

## Instalação manual

Se preferir cada passo, conclua [os companheiros](companions.pt-BR.md) primeiro e depois:

### 1. Baixe e instale o tightship

Baixe `tightship-cursor.zip` do release mais recente, extraia e copie o conteúdo para
`~/.cursor/plugins/local/tightship/`.

### 2. Escreva suas opções

Crie `~/.cursor/tightship/options.json`:

```json
{
  "contract": true,
  "memory": true,
  "status_line": true,
  "auto_update": true,
  "vault_folder": "~/ai-memory"
}
```

Ou use [tightship options](options.pt-BR.md).

### 3. Ative a status line do CLI

```sh
export TIGHTSHIP_DATA="$HOME/.cursor/tightship"
export TIGHTSHIP_SETTINGS="$HOME/.cursor/cli-config.json"
export TIGHTSHIP_ROOT="$HOME/.cursor/plugins/local/tightship"
"$TIGHTSHIP_ROOT/bin/tightship" hook status-line
```

Recarregue a janela do editor depois de copiar a pasta do plugin.

## Confira

Em Customize → Plugins, o tightship aparece como **Local** com skills e comandos listados. ponytail,
i-have-adhd e humanizer também ficam em **Local**.

Numa sessão interativa do `agent`, pergunte quais comandos slash estão disponíveis. A resposta deve listar
`/spec` e os outros comandos do workflow.

Depois de um `cargo xtask build` local, você pode confirmar o mesmo com `agent --plugin-dir dist/cursor` em
vez do zip do release.
