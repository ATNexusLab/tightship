# Os companheiros

[English](companions.md) · Português

O tightship é usado com três plugins de terceiros. Ele os declara como dependências, então
`claude plugin install tightship@tightship` também os instala, desde que os catálogos deles tenham sido
adicionados.

| Companheiro | Catálogo | O que traz |
| --- | --- | --- |
| ponytail | `DietrichGebert/ponytail` | Empurra o agente para o código mais simples que funciona: nada de abstração especulativa, a biblioteca padrão primeiro |
| i-have-adhd | `ayghri/i-have-adhd` | Molda as respostas para você agir sobre elas: a próxima ação primeiro, passos numerados, sem preâmbulo |
| humanizer | `blader/humanizer` | Reescreve a prosa feita para pessoas, para que ela não pareça texto gerado |

Um companheiro que você já tinha instalado é reaproveitado, não instalado de novo. A versão de nenhum
companheiro é fixada: você recebe a mais recente de cada um.

Na desinstalação, `--prune` remove só os companheiros que a instalação do tightship adicionou. Um que você
mesmo instalou fica. Veja a [desinstalação](uninstall.pt-BR.md).
