# Os companheiros

[English](companions.md) · Português

O tightship é usado com três plugins de terceiros. O manifesto do plugin no Cursor não tem campo
`dependencies`, então o editor e o CLI não instalam os companheiros por você. Adicione cada catálogo,
instale cada companheiro e confira que aparece **Installed** em Customize → Plugins antes de seguir a
[instalação](install.pt-BR.md).

| Companheiro | Catálogo | O que traz |
| --- | --- | --- |
| ponytail | `DietrichGebert/ponytail` | Empurra o agente para o código mais simples que funciona: nada de abstração especulativa, a biblioteca padrão primeiro |
| i-have-adhd | `ayghri/i-have-adhd` | Molda as respostas para você agir sobre elas: a próxima ação primeiro, passos numerados, sem preâmbulo |
| humanizer | `blader/humanizer` | Reescreve a prosa feita para pessoas, para que ela não pareça texto gerado |

Um companheiro que você já tinha instalado é reaproveitado, não instalado de novo. A versão de nenhum
companheiro é fixada: você recebe a mais recente de cada um.

## Adicione os catálogos

Rode uma vez por repositório, os companheiros primeiro; o do tightship entra quando chegar na
[instalação](install.pt-BR.md):

```sh
agent plugin marketplace add DietrichGebert/ponytail
agent plugin marketplace add ayghri/i-have-adhd
agent plugin marketplace add blader/humanizer
agent plugin marketplace add ATNexusLab/tightship
```

No editor, você pode adicionar as mesmas URLs git em Customize → Plugins → marketplaces em vez dos comandos
acima.

## Instale os três companheiros

Com os catálogos adicionados, instale só os companheiros (ainda não o tightship):

```sh
agent plugin install ponytail@ponytail
agent plugin install i-have-adhd@i-have-adhd
agent plugin install humanizer@humanizer
```

Se faltar um catálogo, a instalação falha ou o plugin não aparece. Adicione o catálogo que falta e rode a
instalação de novo.

## Confira

Em Customize → Plugins, ponytail, i-have-adhd e humanizer aparecem como **Installed**. Só então continue na
[instalação](install.pt-BR.md).
