# Atualização

[English](update.md) · Português

O tightship se atualiza sozinho a partir dos lançamentos no GitHub, tanto no editor Cursor quanto no CLI
`agent`. As suas opções em `~/.cursor/tightship/options.json` mantêm seus valores durante uma atualização.

## Automática

Com a opção `auto_update` ligada, que é o padrão, cada início de sessão procura uma release nova, no máximo
uma vez por dia, sem fazer a sessão esperar. Uma release mais nova é baixada em segundo plano, tem o sha256
conferido com o que a release publicou e é colocada no lugar, na pasta do plugin. A sessão seguinte avisa,
uma única vez: "tightship was updated from 1.1.0 to 1.2.0."

Um download cujo sha256 não confere não é instalado, e a sessão seguinte avisa. Uma falha de rede não muda
nada e é tentada de novo no dia seguinte.

Para procurar agora, rode o atualizador você mesmo, com `PLUGIN` definido como em
[as opções](options.pt-BR.md#o-programa-tightship):

```sh
"$PLUGIN/bin/tightship" update
```

Para desligar:

```sh
"$PLUGIN/bin/tightship" options set auto_update false
```

## Pelo instalador

Rodar o instalador de novo troca o tightship e os companheiros pelas versões mais recentes, e mantém as suas
opções:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness cursor --yes
```

No Windows, no PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness cursor -Yes
```

Os companheiros só se atualizam por esse caminho.

## À mão

Baixe o `tightship-cursor.zip` do release mais recente, extraia e troque o conteúdo de
`~/.cursor/plugins/local/tightship/` por ele. Recarregue a janela do editor.
