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

`--prune` também remove os companheiros que a instalação adicionou e a pasta de dados do plugin. Um
companheiro que você mesmo instalou fica. O Claude Code apaga as opções do tightship das suas configurações
de usuário.

## 3. Remova os catálogos

```sh
claude plugin marketplace remove tightship
claude plugin marketplace remove ponytail
claude plugin marketplace remove i-have-adhd
claude plugin marketplace remove humanizer
```

Mantenha o catálogo de um companheiro se você ainda usa esse companheiro.

O seu vault nunca é tocado.
