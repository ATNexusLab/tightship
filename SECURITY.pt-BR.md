# Política de segurança

[English](SECURITY.md) · Português

## Versões com suporte

O tightship está em pré-release. Só o release publicado mais recente recebe correções de segurança.

## Reporte uma vulnerabilidade

Reporte em privado pelo
[reporte privado de vulnerabilidades do GitHub](https://github.com/ATNexusLab/tightship/security/advisories/new).
Não abra issue, pull request nem discussão pública sobre ela.

Inclua:

- a versão, o harness e o sistema operacional
- o que um atacante consegue fazer e do que ele precisa para isso
- os passos para reproduzir, ou uma prova de conceito

## O que acontece depois

O mantenedor confirma o recebimento em até 7 dias. Vocês dois combinam a severidade e uma data para a
correção. A correção sai num novo release, e o advisory é publicado quando o release sai, com crédito a você,
a não ser que você peça para não ser citado.

## Escopo

O plugin roda hooks e um binário na sua máquina, lê o seu vault e grava nas configurações do seu harness.
Reportes sobre qualquer um destes estão no escopo:

- execução de código ou acesso a arquivos além do que o
  [guia de instalação](docs/install/claude-code/README.pt-BR.md) descreve
- a sua identidade, os seus segredos ou as notas do seu vault vazando para um build ou um release
- os binários ou o catálogo do release não batendo com o código deste repositório

Vulnerabilidades num plugin companheiro (ponytail, i-have-adhd, humanizer) ou num harness são dos
mantenedores de cada um.
