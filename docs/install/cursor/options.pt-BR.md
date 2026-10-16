# As opções

[English](options.md) · Português

O Cursor mostra as opções do tightship em Customize → Configure a partir das `variables` do manifesto. Hooks
e o binário `tightship` ignoram esse diálogo e leem o estado só em `$HOME/.cursor/tightship/options.json`.

| Opção | Tipo | O que faz | Quando não definida |
| --- | --- | --- | --- |
| `contract` | ligada/desligada | Carrega as regras do workflow em toda sessão. Desligada: as regras do contrato não são injetadas | Ligada |
| `memory` | ligada/desligada | Guarda notas curadas no seu vault e passa o estado da sessão para a próxima | Ligada |
| `status_line` | ligada/desligada | Só no **CLI**, mostra a status line do tightship sob o prompt e devolve a sua quando desligada | Ligada |
| `vault_folder` | pasta | O seu vault de memória, um repositório git, como caminho absoluto ou `~`. A memória precisa dele | Sem vault: a memória não tem o que ler nem onde gravar |

Arquivo ausente ou chave ausente significa **ligada** para cada opção booleana.

## O binário tightship

Os comandos abaixo chamam `bin/tightship` na pasta onde o Cursor instalou o plugin. Defina `PLUGIN` para essa
pasta (Customize → Plugins mostra os plugins instalados; uma cópia local de teste fica em
`~/.cursor/plugins/local/tightship/`):

```sh
PLUGIN=~/.cursor/plugins/local/tightship   # exemplo — use o caminho da sua instalação
TIGHTSHIP="$PLUGIN/bin/tightship"
```

No Windows, use `"$PLUGIN/bin/tightship.cmd"` em vez de `"$PLUGIN/bin/tightship"` nos exemplos.

## Definir e listar opções

```sh
"$TIGHTSHIP" options set memory false
"$TIGHTSHIP" options set vault_folder ~/ai-memory
"$TIGHTSHIP" options list
"$TIGHTSHIP" options reset
```

`set` imprime `Set <name> to <value>.` e sai com `0`. `list` imprime JSON formatado, ou `{}` quando o arquivo
não existe. `reset` remove `options.json` e imprime `Removed <path>.`, com o caminho completo do arquivo.

Uma mudança vale a partir da próxima sessão ou da próxima execução de hook.

## O vault

A memória precisa de um vault, um repositório git de notas. Se você não tem um, rode
`"$TIGHTSHIP" vault init <folder>` (ou peça ao agente numa sessão, com o caminho do binário em
[o binário](#o-binário-tightship)). Defina a pasta com `options set vault_folder <path>` e comece uma nova
sessão. Para compartilhar o vault entre máquinas, envie-o para um remoto privado e clone em cada uma.

Com a memória ligada e sem `vault_folder`, a sessão fica sem vault, e o agente explica como criar um quando
você pergunta sobre memória.

## A status line

O editor Cursor não tem API de status line. Com `status_line` ligada, a próxima sessão interativa do **CLI**
grava o `statusLine` do tightship em `~/.cursor/cli-config.json` e guarda o seu de lado.

Desligue a opção para devolver o `statusLine` anterior no próximo início de sessão (ou remover a chave se
você não tinha nenhuma):

```sh
"$TIGHTSHIP" options set status_line false
```

Depois inicie uma sessão interativa do CLI para o hook de status line rodar, ou rode `"$TIGHTSHIP" hook
status-line`.
