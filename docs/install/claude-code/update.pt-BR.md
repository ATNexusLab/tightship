# Atualização

[English](update.md) · Português

O Claude Code encontra uma nova versão pelo catálogo que você adicionou a partir da URL do release. As suas
opções mantêm os valores numa atualização.

## Automática

Com a opção `auto_update` ligada, que é o padrão, o tightship liga a atualização automática do Claude Code
para o seu catálogo a cada início de sessão. O Claude Code passa a instalar cada release nova sozinho, e a
sessão seguinte a uma atualização a carrega.

Desligue a opção para parar; o próximo início de sessão desliga de novo a atualização automática do catálogo:

```sh
echo '{"auto_update":"false"}' | claude plugin configure tightship@tightship --values-stdin
```

## Pelo instalador

Rodar o instalador de novo atualiza o tightship e os três companheiros, e mantém as opções que você já tinha
escolhido:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | \
  sh -s -- --harness claude-code --yes
```

No Windows, no PowerShell:

```powershell
& ([scriptblock]::Create((irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1))) `
  -Harness claude-code -Yes
```

## À mão

```sh
claude plugin marketplace update tightship
claude plugin update tightship@tightship
```

A nova versão carrega na próxima sessão.
