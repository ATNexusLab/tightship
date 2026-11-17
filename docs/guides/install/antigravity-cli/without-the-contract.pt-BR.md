# Sem o contrato

[English](without-the-contract.md) · Português

Com a opção `contract` desligada, a sessão deixa de carregar as regras do workflow. Toda skill e todo comando
continuam carregando e funcionando. Cada um traz as regras do seu próprio passo, então funciona igual com o
contrato desligado. A sessão perde as regras que toda tarefa precisa: as hard lines, como uma convenção cede
à sua decisão, as regras de código e de pronto, e as regras de memória. O agente passa a seguir o `AGENTS.md`
do seu repositório e o próprio julgamento.

Uma peça ainda aponta para o contrato em vez de repeti-lo:

| Peça | A que ela se refere | O que faz com o contrato desligado |
| --- | --- | --- |
| `bootstrap` | O contrato inteiro: o `AGENTS.md` que ela escreve complementa o contrato e nunca o repete | Escreve o mesmo `AGENTS.md`, que deixa de fora tudo o que o contrato cobre. Essas regras então não existem em lugar nenhum, a menos que você as adicione ao repositório |
