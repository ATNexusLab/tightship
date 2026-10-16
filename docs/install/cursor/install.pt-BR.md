# Instalação

[English](install.md) · Português

Conclua [os companheiros](companions.pt-BR.md) primeiro.

Não rode um comando de workflow até ponytail, i-have-adhd e humanizer aparecerem como **Installed** em
Customize → Plugins.

## Antes de começar

- Cursor **3.25.6** ou posterior no editor. Confira em **Help → About** no editor.
- O CLI do Cursor, comando **`agent`**, **2026.10.14** ou posterior, se você usa o agente na linha de
  comando. Confira com `agent --version`.
- Git, com a memória ligada, porque o vault é um repositório git.

Não instale o tightship no Claude Code e no Cursor na mesma máquina. O Cursor importa plugins do Claude Code
como **Imported**, e as mesmas peças carregariam duas vezes.

## Adicione o catálogo do tightship

Se você seguiu [os companheiros](companions.pt-BR.md), os quatro catálogos já estão adicionados. Caso
contrário:

```sh
agent plugin marketplace add DietrichGebert/ponytail
agent plugin marketplace add ayghri/i-have-adhd
agent plugin marketplace add blader/humanizer
agent plugin marketplace add ATNexusLab/tightship
```

## Instale o tightship

Com os três companheiros já como **Installed**:

```sh
agent plugin install tightship@tightship
```

Você pode instalar em Customize → Plugins no editor em vez do comando acima.

## Confira

Em Customize → Plugins, ponytail, i-have-adhd, humanizer e tightship aparecem como **Installed**, e as skills
e os comandos do tightship aparecem ali.
