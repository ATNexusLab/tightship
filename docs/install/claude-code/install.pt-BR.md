# Instalação

[English](install.md) · Português

Instale o tightship com o instalador de um comando, ou adicione o catálogo e instale manualmente.

## Instalador de um comando

O instalador detecta seus CLIs, pergunta pelas suas opções, adiciona o catálogo do tightship, instala o
plugin e aplica as suas opções. Nada é gravado antes de você escolher Install no resumo.

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

No `cmd`:

```bat
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) -Harness claude-code"
```

Sem `--harness`, a primeira pergunta lista todos os CLIs encontrados, então uma execução também pode instalar
o Cursor e o Antigravity CLI.

## Instalação manual

Adicione o catálogo do tightship, que é este repositório, e instale:

```sh
claude plugin marketplace add ATNexusLab/tightship
claude plugin install tightship@tightship
```

```text
✔ Successfully installed plugin: tightship@tightship (scope: user)
```

A instalação não pede nenhuma opção no shell. Defina as opções ao instalar com `--config`, ou no diálogo da
próxima sessão: veja [as opções](options.pt-BR.md).

## Confira

Rode `claude plugin list`. `tightship@tightship` aparece sem erro.
