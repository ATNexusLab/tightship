# Os companheiros

[English](companions.md) · Português

O tightship é utilizado em conjunto com três plugins e skills companheiros.

| Companheiro | Tipo | O que traz |
| --- | --- | --- |
| ponytail | Plugin (`DietrichGebert/ponytail`) | Conduz o agente para o código mais simples que funciona: sem abstração especulativa, biblioteca padrão primeiro |
| i-have-adhd | Plugin (`ayghri/i-have-adhd`) | Formata respostas para ação imediata: próxima ação primeiro, passos numerados, sem introduções |
| humanizer | Skill (`blader/humanizer`) | Reescreve prosa voltada a pessoas para não soar como texto gerado por IA |

## Locais de instalação

- **ponytail:** instalado em `~/.gemini/config/plugins/ponytail/` via `agy plugin install`.
- **i-have-adhd:** instalado em `~/.gemini/config/plugins/i-have-adhd/` via `agy plugin install`.
- **humanizer:** instalado como uma skill global em `~/.gemini/config/skills/humanizer/SKILL.md`.

Um companheiro que você já possuía instalado é reutilizado, sem instalação duplicada. Nenhuma versão de
companheiro é fixada: você obtém a versão mais recente de cada um.

Na desinstalação, remover o tightship não apaga companheiros que você deseje manter. Veja
[desinstalação](uninstall.pt-BR.md).
