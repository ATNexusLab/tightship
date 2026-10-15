# [![tightship: ship it tied down](docs/design/brand/banner.svg)](https://github.com/ATNexusLab/tightship)

[English](README.md) · Português

[![License: MIT](https://img.shields.io/badge/license-MIT-5C6670?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/github/actions/workflow/status/ATNexusLab/tightship/rust.yml?style=flat-square&label=rust)](https://github.com/ATNexusLab/tightship/actions/workflows/rust.yml)
[![Scan](https://img.shields.io/github/actions/workflow/status/ATNexusLab/tightship/scan.yml?style=flat-square&label=scan)](https://github.com/ATNexusLab/tightship/actions/workflows/scan.yml)
[![Claude Code 2.1.295+](https://img.shields.io/badge/Claude%20Code-2.1.295%2B-FF5A1F?style=flat-square)](docs/install/claude-code/README.pt-BR.md)
[![Status: pre-release](https://img.shields.io/badge/status-pre--release-121212?style=flat-square)](docs/roadmap/README.md)

**Um workflow para agentes de código com IA, do requisito ao release.**

O tightship carrega um processo de engenharia completo no seu agente de código. Toda mudança ganha uma issue
e uma spec. Os gates precisam voltar verdes, e um verificador precisa falhar ao tentar refutar a alegação de
pronto. Só então a mudança é entregue. Ele se instala como plugin nativo e guarda memória entre sessões e
máquinas.

> [!NOTE]
> Pré-release. O marco M1, o plugin para Claude Code, está sendo finalizado. Ainda não há release estável.
> Veja o [roadmap](docs/roadmap/README.md).

## Por quê

No padrão, um agente de código escreve rápido e esquece tudo entre uma sessão e outra. Ele pula a spec e
dá o trabalho por pronto antes que alguém confira.

Times resolvem isso com processo: requisitos com ID, uma issue para cada mudança, gates que precisam passar,
revisão antes do commit. O tightship escreve esse processo uma vez e o carrega no agente. Quem trabalha
sozinho ganha o mesmo rigor que um time exigiria.

## O que vem junto

| Peça | O que faz |
| --- | --- |
| Contrato | Regras em toda sessão: como o agente planeja, escreve código, testa, lida com git e segredos, e o que conta como pronto |
| 10 comandos | Um por etapa: `/requirements`, `/bootstrap`, `/sprint`, `/spec`, `/epic`, `/audit`, `/closeout`, `/pr-review`, `/handoff`, `/sync` |
| 13 skills | Carregadas sob demanda: arquitetura de API, de banco de dados, de frontend e hexagonal, engenharia de requisitos, escrita de spec, auditoria de segurança, testes, performance, escrita técnica e outras |
| Verificador | Um subagente somente leitura que tenta refutar uma correção, um achado ou um "pronto" antes que você confie nele |
| Memória | Opcional. Notas curadas num repositório git seu, compartilhadas entre sessões, máquinas e harnesses |
| Status line | Opcional. Gravada nas suas configurações só depois que você concorda |

Três plugins companheiros são instalados junto. O [ponytail](https://github.com/DietrichGebert/ponytail)
mantém o código no mais simples que funciona. O [i-have-adhd](https://github.com/ayghri/i-have-adhd)
molda as respostas para você agir sobre elas. O [humanizer](https://github.com/blader/humanizer) evita que
a prosa pareça gerada.

## Como uma mudança anda

```text
/requirements → /bootstrap → /sprint plan → grilling → /spec → /epic
      → plan → tests → code → gates green → /audit → /closeout → commit → PR
```

Todo requisito tem um ID. A spec cita os IDs que cobre. O épico e as sub-issues saem da spec. Todo commit e
todo pull request apontam para a sua issue. Nada é commitado antes de os gates passarem com zero erros e
zero avisos e de você ter lido a mudança.

## Início rápido

Você precisa do Claude Code 2.1.295 ou posterior. No Windows, também do Git for Windows: os hooks rodam no
`sh` dele.

```sh
claude plugin marketplace add DietrichGebert/ponytail
claude plugin marketplace add ayghri/i-have-adhd
claude plugin marketplace add blader/humanizer
claude plugin marketplace add https://github.com/ATNexusLab/tightship/releases/latest/download/marketplace.json
claude plugin install tightship@tightship
```

A próxima sessão pergunta quais opções ligar. O [guia de instalação](docs/install/claude-code/README.pt-BR.md)
cobre opções, atualizações, desinstalação e a migração da instalação anterior ao plugin.

## Harnesses

| Harness | Status | Marco |
| --- | --- | --- |
| Claude Code | Em andamento | M1 |
| Cursor | Planejado | M2 |
| Antigravity CLI | Planejado | M3 |
| Codex | Planejado | M4 |

Cada peça é escrita uma vez em `content/`, que não cita nenhum harness. Um build em Rust a transforma no
formato de plugin nativo de cada harness
([ADR 0001](docs/decisions/0001-rust-build-and-tokenized-canonical-content.md)). Roda em macOS, Linux e
Windows.

## Build a partir do código

Você precisa do `rustup`, que instala o toolchain fixado na primeira chamada ao `cargo`, e do gitleaks
8.30.1 no `PATH`.

```sh
git clone https://github.com/ATNexusLab/tightship.git
cd tightship
cargo xtask build
claude --plugin-dir dist/claude-code
```

O build precisa de um arquivo `.identity-terms` na raiz. O [CONTRIBUTING.pt-BR.md](CONTRIBUTING.pt-BR.md)
explica esse arquivo e lista todos os gates.

## Docs

O guia de instalação tem versão em português. O resto da documentação está só em inglês.

| Onde | O quê |
| --- | --- |
| [Guia de instalação](docs/install/claude-code/README.pt-BR.md) | Instalação, opções, atualizações, desinstalação, migração |
| [Requisitos](docs/requirements/README.md) | A SRS versionada: cada requisito e seus critérios de aceite |
| [Arquitetura](docs/architecture/README.md) | O mapa do sistema, nas seções do arc42 com diagramas C4 |
| [Decisões](docs/decisions/README.md) | Os registros de decisão de arquitetura |
| [Roadmap](docs/roadmap/README.md) | Marcos, épicos e cobertura dos requisitos |
| [Design](docs/design/README.md) | A marca, a voz e os tokens |

## Contribuir

Issues e pull requests são bem-vindos. Leia o [CONTRIBUTING.pt-BR.md](CONTRIBUTING.pt-BR.md) antes.
Reporte vulnerabilidades como o [SECURITY.pt-BR.md](SECURITY.pt-BR.md) indica, nunca numa issue pública.
Todos seguem o [Código de Conduta](CODE_OF_CONDUCT.pt-BR.md).

## Licença

[MIT](LICENSE) © ATNexusLab
