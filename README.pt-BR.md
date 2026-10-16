# [![tightship: ship it tied down](docs/design/brand/banner.svg)](https://github.com/ATNexusLab/tightship)

[English](README.md) · Português

[![License: MIT](https://img.shields.io/badge/license-MIT-5C6670?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/github/actions/workflow/status/ATNexusLab/tightship/rust.yml?style=flat-square&label=rust)](https://github.com/ATNexusLab/tightship/actions/workflows/rust.yml)
[![Scan](https://img.shields.io/github/actions/workflow/status/ATNexusLab/tightship/scan.yml?style=flat-square&label=scan)](https://github.com/ATNexusLab/tightship/actions/workflows/scan.yml)
[![Claude Code 2.1.295+](https://img.shields.io/badge/Claude%20Code-2.1.295%2B-FF5A1F?style=flat-square)](docs/install/claude-code/README.pt-BR.md)
[![Cursor 3.25.6+](https://img.shields.io/badge/Cursor-3.25.6%2B-121212?style=flat-square)](docs/install/cursor/README.pt-BR.md)
[![Antigravity CLI 1.4.0+](https://img.shields.io/badge/Antigravity%20CLI-1.4.0%2B-4285F4?style=flat-square)](docs/install/antigravity-cli/README.pt-BR.md)
[![Release](https://img.shields.io/github/v/release/ATNexusLab/tightship?include_prereleases&style=flat-square&label=release&color=121212)](https://github.com/ATNexusLab/tightship/releases)

**Um workflow para agentes de código com IA, do requisito ao release.**

O tightship carrega um processo de engenharia completo no seu agente de código. Toda mudança ganha uma issue
e uma spec. Os gates precisam voltar verdes, e um verificador precisa falhar ao tentar refutar a alegação de
pronto. Só então a mudança é entregue. Ele se instala como plugin nativo e guarda memória entre sessões e
máquinas.

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
| Status line | Opcional. Gravada nas suas configurações só enquanto você a deixa ligada |

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

Execute o instalador para o Claude Code (2.1.295+), Antigravity CLI (1.4.0+) ou ambos:

```sh
curl -fsSL https://github.com/ATNexusLab/tightship/releases/latest/download/install.sh | sh
```

No Windows usando o PowerShell:

```powershell
irm https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1 | iex
```

O instalador detecta seus CLIs e pergunta quais opções ligar. Os guias de instalação
([Claude Code](docs/install/claude-code/README.pt-BR.md),
[Antigravity CLI](docs/install/antigravity-cli/README.pt-BR.md)) cobrem opções, atualizações, desinstalação
e a migração da instalação anterior. Para o Cursor (3.25.6+, `agent` 2026.10.14+), siga o
[guia de instalação do Cursor](docs/install/cursor/README.pt-BR.md); ele ainda não entra no instalador
unificado.

## Harnesses

| Harness | Status | Marco |
| --- | --- | --- |
| Claude Code | Disponível | M1 |
| Antigravity CLI | Disponível | M3 |
| Cursor | Planejado | M2 |
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
agent --plugin-dir dist/cursor
```

O build precisa de um arquivo `.identity-terms` na raiz. O [CONTRIBUTING.pt-BR.md](CONTRIBUTING.pt-BR.md)
explica esse arquivo e lista todos os gates.

## Docs

O guia de instalação tem versão em português. O resto da documentação está só em inglês.

| Onde | O quê |
| --- | --- |
| [Guia de instalação (Claude Code)](docs/install/claude-code/README.pt-BR.md) | Instalação, opções, atualizações, desinstalação, migração |
| [Guia de instalação (Cursor)](docs/install/cursor/README.pt-BR.md) | Companheiros, instalação pelo marketplace, opções, desinstalação |
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
