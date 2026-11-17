# Onde o tightship roda

[English](README.md) · Português

Onde o tightship roda: uma linha por superfície de cada harness na sua máquina, com como o plugin chega lá e
quais das suas peças carregam. Uma peça que não carrega diz por quê.

| Harness | Superfície | Como chega lá | Contrato | Skills | Comandos | Hooks | Status line | Vault |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Claude Code | CLI¹ | O instalador, ou `claude plugin install` | Sim | Sim | Sim | Sim | Sim | Sim |
| Claude Code | App desktop | A mesma instalação da CLI: as sessões locais do app leem `~/.claude` | Sim | Sim | Sim | Sim | Sim | Sim |
| Claude Code | Terminal do Zed¹ | A mesma instalação da CLI | Sim | Sim | Sim | Sim | Sim | Sim |
| Claude Code | Agent panel do Zed | A entrada `claude-acp` do registro | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) |
| Cursor | Editor | O instalador, ou uma cópia em `~/.cursor/plugins/local/` | Sim | Sim | Sim | Sim | Não⁴ | Sim |
| Cursor | CLI `agent`² | A mesma instalação do editor | Sim | Sim | Sim | Sim | Sim | Sim |
| Cursor | Terminal do Zed² | A mesma instalação do editor | Sim | Sim | Sim | Sim | Sim | Sim |
| Cursor | Agent panel do Zed | A entrada `cursor` do registro | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) |
| Antigravity | CLI³ | O instalador, ou `agy plugin install` | Sim | Sim | Sim | Sim | Sim | Sim |
| Antigravity | IDE | A mesma instalação da CLI: a IDE lê `~/.gemini/config/plugins/` | Sim | Sim | Sim | Sim | Não⁵ | Sim |
| Antigravity | App desktop | A mesma instalação da CLI: o app lê `~/.gemini/config/plugins/` | Sim | Sim | Sim | Sim | Não⁵ | Sim |
| Antigravity | Terminal do Zed³ | A mesma instalação da CLI | Sim | Sim | Sim | Sim | Sim | Sim |
| Antigravity | Agent panel do Zed | A entrada `antigravity-acp` do registro | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) | [Ainda não checado](https://github.com/ATNexusLab/tightship/issues/163) |

Numa linha de CLI, um `Sim` vem dos testes unitários e de integração, que rodam a CLI. O terminal do Zed roda
a mesma CLI, então a linha dele repete a da CLI. Nenhum teste roda um app desktop, uma IDE ou o agent panel
do Zed: ali, um `Sim` vem de uma sessão em uso, e uma célula ainda não checada aponta para a issue que a
acompanha.

1. Claude Code: `xtask/tests/contract.rs` gera o contrato e seus hooks e os roda;
   `xtask/tests/build/references.rs` gera as skills e os comandos; `xtask/tests/install/claude_code.rs`
   instala o plugin, aplica a status line e cria o vault.
2. Cursor: `xtask/tests/cursor_contract.rs` confere os hooks e roda o contrato;
   `xtask/tests/build/cursor.rs` gera as skills, os comandos e o hook da status line;
   `xtask/tests/install/cursor.rs` instala em `~/.cursor/plugins/local/`;
   `tightship/tests/hook/start.rs` lê o vault. Nenhum teste roda os hooks do vault a partir do plugin gerado.
3. Antigravity: `xtask/tests/antigravity_cli_contract.rs` roda os hooks do contrato;
   `xtask/tests/build/manifest.rs` gera as skills e os comandos; `xtask/tests/install/antigravity_cli.rs`
   instala e grava a status line; `tightship/tests/hook/contract_part.rs` injeta as notas do vault. Nenhum
   teste desenha a status line a partir da pasta de configurações do Antigravity.
4. O editor do Cursor não tem status line: só a CLI `agent` lê `statusLine` em `~/.cursor/cli-config.json`.
5. O Antigravity documenta `statusLine` [só para a CLI](https://antigravity.google/docs/settings); a IDE e o
   app não desenham uma.

Superfícies web não têm suporte: claude.ai, claude.ai/code, os Cloud Agents do Cursor e os Remote Agents do
Antigravity. O tightship trabalha num checkout na sua máquina.

Guias de instalação: [Claude Code](claude-code/README.pt-BR.md) · [Cursor](cursor/README.pt-BR.md) ·
[Antigravity CLI](antigravity-cli/README.pt-BR.md).
