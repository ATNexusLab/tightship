# Sem o contrato

[English](without-the-contract.md) · Português

Com a opção `contract` desligada, a sessão deixa de carregar as regras do workflow. Toda skill e todo comando
continuam carregando e funcionando. Os listados abaixo apontam para uma regra do contrato em vez de
repeti-la, então com o contrato desligado essa regra não está na sessão. Cada um recorre ao que o
`AGENTS.md` do seu repositório diz, ou ao julgamento do próprio agente.

As regras aparecem com o nome em inglês, como estão no contrato.

| Peça | Regra do contrato que ela cita | O que faz com o contrato desligado |
| --- | --- | --- |
| `audit` | **Code I write**, para o eixo de manutenibilidade; **Security**, para onde um trade-off deliberado é registrado; o ciclo de desenvolvimento, para como uma correção é feita | Julga a manutenibilidade pelo `AGENTS.md` do repositório e pela prática comum; não indica onde registrar um trade-off; uma correção não segue ordem definida |
| `bootstrap` | O contrato inteiro: o `AGENTS.md` que ela escreve complementa o contrato e nunca o repete | Escreve o mesmo `AGENTS.md`, que deixa de fora tudo o que o contrato cobria. Essas regras então não existem em lugar nenhum, a menos que você as adicione ao repositório |
| `closeout` | **Process**, para a cadeia do requisito ao PR e para o que pode ser adiado; **Git**, para branches, commits e PRs | Confere a rastreabilidade e a dívida de segurança só contra o `AGENTS.md` do repositório; nada define a cadeia nem o que conta como dívida |
| `epic` | **Git**, para a branch, os commits e o PR do épico | Cria o épico e as sub-issues do mesmo jeito; os nomes de branch e de commit vêm do repositório ou do padrão do agente |
| `testing-contract` | **Testing**, que decide se um teste existe e o que a suíte contém | Ainda diz como escrever um teste; nada decide quais testes escrever |
| `security-audit` | **Security**, para onde um trade-off deliberado é registrado | Ainda separa um trade-off registrado de um achado; não indica onde registrá-lo |
| `frontend-architecture` | **Security**, para as variáveis de ambiente enviadas ao navegador; **UI**, para o tamanho de um estado de carregamento | Deixa de fora as duas checagens, que ela não repete |
| `performance-analysis` | **UI**, para o tamanho de um estado de carregamento | Ainda trata layout shift como bug; a regra de que um skeleton mantém o tamanho do conteúdo final não está na sessão |
