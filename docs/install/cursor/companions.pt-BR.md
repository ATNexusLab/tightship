# Os companheiros

[English](companions.md) · Português

O tightship é usado com três plugins de terceiros. O manifesto do plugin no Cursor não tem campo
`dependencies`, então o editor e o CLI não instalam os companheiros por você.

| Companheiro | Repositório | O que traz |
| --- | --- | --- |
| ponytail | `DietrichGebert/ponytail` | Empurra o agente para o código mais simples que funciona: nada de abstração especulativa, a biblioteca padrão primeiro |
| i-have-adhd | `ayghri/i-have-adhd` | Molda as respostas para você agir sobre elas: a próxima ação primeiro, passos numerados, sem preâmbulo |
| humanizer | `blader/humanizer` | Reescreve a prosa feita para pessoas, para que ela não pareça texto gerado |

ponytail e i-have-adhd trazem só `.claude-plugin/`, que o Cursor também lê: em `~/.cursor/plugins/local/`
eles aparecem como **Local**, como o humanizer, que traz também `.cursor-plugin/`.

## Com o instalador unificado

A [instalação](install.pt-BR.md) instala os três companheiros em `~/.cursor/plugins/local/<nome>/` antes do
tightship. Você não adiciona marketplaces nem roda comandos de instalação separados.

## Instalação manual

Baixe do GitHub o repositório que a tabela indica para cada companheiro, extraia e copie a árvore para
`~/.cursor/plugins/local/<nome>/` (por exemplo `~/.cursor/plugins/local/ponytail/`).

Confira que cada um aparece como **Local** em Customize → Plugins antes de instalar o tightship.

Rodar o instalador de novo troca a pasta de cada companheiro por uma cópia nova. A versão de nenhum
companheiro é fixada: você recebe a mais recente do branch padrão de cada um.

## Confira

Em Customize → Plugins, ponytail, i-have-adhd e humanizer aparecem como **Local**. Só então continue na
[instalação](install.pt-BR.md).
