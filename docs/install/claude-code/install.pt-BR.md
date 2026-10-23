# Instalação

[English](install.md) · Português

Instale o tightship com o instalador de um comando, ou adicione os catálogos e instale manualmente.

## Instalador de um comando

O instalador detecta seus CLIs, pergunta pelas suas opções, adiciona os catálogos dos companheiros e o do
tightship, instala o plugin e aplica as suas opções. Nada é gravado antes de você escolher Install no resumo.

No Linux e macOS:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness claude-code
```

No Windows usando o PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness claude-code
```

Sem `--harness`, a primeira pergunta lista todos os CLIs encontrados, então uma execução também pode instalar
o Cursor e o Antigravity CLI.

## Instalação manual

O tightship depende de três plugins companheiros, cada um no seu próprio catálogo. Adicione esses três
catálogos primeiro, depois o do tightship, que é este repositório, e instale uma vez:

```sh
claude plugin marketplace add DietrichGebert/ponytail
claude plugin marketplace add ayghri/i-have-adhd
claude plugin marketplace add blader/humanizer
claude plugin marketplace add ATNexusLab/tightship
claude plugin install tightship@tightship
```

A instalação adiciona os companheiros junto com o plugin:

```text
✔ Successfully installed plugin: tightship@tightship (scope: user) (+ 3 dependencies: ponytail, i-have-adhd, humanizer)
```

Se faltar o catálogo de um companheiro, o Claude Code avisa que a dependência não foi instalada, e o
tightship não carrega. Adicione o catálogo que falta e rode a instalação de novo.

A instalação não pede nenhuma opção no shell. Defina as opções ao instalar com `--config`, ou no diálogo da
próxima sessão: veja [as opções](options.pt-BR.md).

## Confira

Rode `claude plugin list`. `tightship@tightship` aparece sem erro, assim como `ponytail`, `i-have-adhd` e
`humanizer`.
