# Atualização

[English](update.md) · Português

O Claude Code encontra uma nova versão pelo catálogo que você adicionou a partir da URL do release. As suas
opções mantêm os valores numa atualização.

## Automática

A atualização automática vem desligada para um catálogo fora do catálogo oficial do Claude Code. Para
ligá-la:

1. Rode `/plugin` numa sessão.
2. Abra **Marketplaces**, depois `tightship`.
3. Escolha **Enable auto-update**.

A sessão seguinte a uma atualização carrega a nova versão.

## À mão

```sh
claude plugin marketplace update tightship
claude plugin update tightship@tightship
```

A nova versão carrega na próxima sessão.
