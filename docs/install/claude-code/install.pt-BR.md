# Instalação

[English](install.md) · Português

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
