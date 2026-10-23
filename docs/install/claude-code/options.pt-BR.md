# As opções

[English](options.md) · Português

O Claude Code guarda as opções do tightship nas suas configurações de usuário. Uma nova versão mantém os
valores que você definiu, e a desinstalação os apaga.

| Opção | Tipo | O que faz | Quando não definida |
| --- | --- | --- | --- |
| `contract` | ligada/desligada | Carrega as regras do workflow em toda sessão. Desligada: veja [sem o contrato](without-the-contract.pt-BR.md) | Ligada |
| `memory` | ligada/desligada | Guarda notas curadas no seu vault e passa o estado da sessão para a próxima | Ligada |
| `status_line` | ligada/desligada | Mostra a status line do tightship sob o prompt e devolve a sua quando desligada | Ligada |
| `auto_update` | ligada/desligada | Liga a atualização automática do Claude Code para o catálogo do tightship, e cada release nova se instala sozinha. Veja [atualização](update.pt-BR.md) | Ligada |
| `vault_folder` | pasta | O seu vault de memória, um repositório git, como caminho absoluto. A memória precisa dele | Sem vault: a memória não tem o que ler nem onde gravar |

## Defina ao instalar

No shell, passe um `--config` por opção:

```sh
claude plugin install tightship@tightship --config memory=true --config vault_folder=/home/you/vault
```

Numa sessão, `/plugin install tightship@tightship` instala e depois abre um diálogo para cada opção ainda não
definida. Uma instalação pelo shell sem `--config` informa `5 userConfig options not yet set` e deixa
ligada cada opção ligada/desligada.

## Mude depois

- Numa sessão: `/plugin configure tightship@tightship` abre o mesmo diálogo.
- No shell: `claude plugin configure tightship@tightship` lista cada opção e se ela está definida. Para
  mudar algumas, passe um objeto JSON para ele pelo pipe; as opções que você deixar de fora mantêm os
  valores:

  ```sh
  echo '{"memory":"false"}' | claude plugin configure tightship@tightship --values-stdin
  ```

Uma mudança vale a partir da próxima sessão.

## O vault

A memória precisa de um vault, um repositório git de notas. Se você não tem um, peça ao agente numa sessão
que rode `tightship vault init <folder>`; ele cria um a partir do template. Defina essa pasta como
`vault_folder` e comece uma nova sessão. Para compartilhar o vault entre máquinas, envie-o para um remoto
privado e clone em cada uma.

Com a memória ligada e sem `vault_folder`, a sessão fica sem vault, e o agente explica como criar um quando
você pergunta sobre memória.

## A status line

Com `status_line` ligada, o tightship mostra a sua status line sob o prompt a partir do próximo início de
sessão. Ela substitui o `statusLine` das suas configurações de usuário, seja qual for, e guarda o anterior de
lado.

Desligar `status_line` devolve a sua status line anterior no próximo início de sessão, ou remove a chave se
você não tinha uma.
