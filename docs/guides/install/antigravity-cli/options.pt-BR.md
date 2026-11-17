# As opções

[English](options.md) · Português

O Antigravity CLI lê as opções do tightship na sua pasta de dados de plugins, no arquivo `options.toml`.
Uma atualização preserva os valores configurados, e uma desinstalação pode removê-los.

No Linux e macOS, o arquivo fica em:
`~/.gemini/config/plugin_data/tightship/options.toml`

No Windows:
`%USERPROFILE%\.gemini\config\plugin_data\tightship\options.toml`

| Opção | Tipo | O que faz | Quando não definida |
| --- | --- | --- | --- |
| `contract` | liga/desliga | Carrega as regras do fluxo em cada sessão. Desligado: veja [sem o contrato](without-the-contract.pt-BR.md) | Ligado (`true`) |
| `memory` | liga/desliga | Mantém notas curadas no vault e repassa o estado para a próxima sessão | Ligado (`true`) |
| `status_line` | liga/desliga | Exibe a status line do tightship abaixo do prompt, e restaura a sua quando desligada | Ligado (`true`) |
| `auto_update` | liga/desliga | Instala cada release nova sozinha e avisa na sessão seguinte. Veja [atualização](update.pt-BR.md) | Ligado (`true`) |
| `vault_folder` | pasta | O seu vault de memória, um repositório git, como caminho (suporta `~`). A memória precisa dele | Sem vault: a memória não tem onde ler ou gravar |

## Variáveis de ambiente

Variáveis de ambiente têm prioridade sobre o `options.toml` quando definidas:

- `TIGHTSHIP_CONTRACT=false` sobrepõe `contract`
- `TIGHTSHIP_MEMORY=false` sobrepõe `memory`
- `TIGHTSHIP_STATUS_LINE=false` sobrepõe `status_line`
- `TIGHTSHIP_AUTO_UPDATE=false` sobrepõe `auto_update`
- `TIGHTSHIP_VAULT=/caminho/do/vault` sobrepõe `vault_folder`
- `TIGHTSHIP_DATA=/caminho/dos/dados` define uma pasta personalizada com o `options.toml`

## Alteração de opções

Você pode editar o arquivo `options.toml` diretamente com qualquer editor de texto, ou executar novamente o
`install.sh` / `install.ps1`. Valores existentes são mantidos como padrão ao rodar novamente o instalador.

```toml
contract = true
memory = true
status_line = true
auto_update = true
vault_folder = "~/ai-memory"
```

A alteração passa a valer na próxima sessão.

## O vault

A memória requer um vault, um repositório git para notas. Se ainda não possui um, execute
`tightship vault init <pasta>` (ou permita que o instalador crie para você). Para compartilhar o vault entre
máquinas, envie-o a um repositório remoto privado e clone-o em cada computador.

Com a memória ligada e sem `vault_folder`, a sessão opera sem vault, e o agente orienta como configurá-lo
quando você perguntar sobre memória.

## A status line

Com `status_line` ligado, o tightship registra o comando de status line em
`~/.gemini/antigravity-cli/settings.json` e salva qualquer configuração anterior em `status-line.json`.

Ao desligar a `status_line`, a sua status line anterior é restaurada no início da próxima sessão, ou a chave
é removida se você não possuía nenhuma anteriormente.
