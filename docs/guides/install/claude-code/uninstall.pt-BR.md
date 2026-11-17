# Desinstalação

[English](uninstall.md) · Português

## 1. Devolva a sua status line

Pule este passo se você nunca ligou a status line do tightship.

O Claude Code não roda código do plugin na desinstalação, e a desinstalação apaga a pasta onde o tightship
guarda a sua status line anterior. Então desligue a opção primeiro e abra uma sessão:

```sh
echo '{"status_line":"false"}' | claude plugin configure tightship@tightship --values-stdin
```

O início dessa sessão devolve o seu `statusLine` anterior, ou remove a chave se você não tinha uma.

## 2. Remova o plugin

```sh
claude plugin uninstall tightship@tightship --prune -y
```

`--prune` também remove a pasta de dados do plugin. O Claude Code apaga as opções do tightship das suas
configurações de usuário.

## 3. Remova o catálogo

```sh
claude plugin marketplace remove tightship
```

O seu vault nunca é tocado.
