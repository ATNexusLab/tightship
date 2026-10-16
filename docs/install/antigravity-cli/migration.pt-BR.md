# Migração

[English](migration.md) · Português

Como migrar para o plugin tightship no Antigravity CLI a partir de regras e skills avulsas, e como conectar a
um vault existente criado no Claude Code ou em outro ambiente.

## 1. Faça backup de `~/.gemini`

Antes de limpar arquivos avulsos, faça uma cópia de segurança da sua pasta de configurações:

```sh
cp -R ~/.gemini ~/.gemini.before-tightship
```

No Windows usando o PowerShell:

```powershell
Copy-Item -Recurse "$HOME\.gemini" "$HOME\.gemini.before-tightship"
```

## 2. Remova skills duplicadas avulsas

Se você havia copiado skills ou comandos do tightship diretamente para `~/.gemini/config/skills/`, eles serão
carregados duas vezes ao lado do plugin. Remova as cópias avulsas, mantendo suas skills pessoais e o
`humanizer`:

```sh
cd ~/.gemini/config/skills && rm -rf \
  api-design architecture-reading database-design frontend-architecture \
  grilling hexagonal-architecture performance-analysis project-docs \
  requirements-engineering security-audit spec-writing technical-writing \
  testing-contract \
  audit bootstrap closeout epic handoff pr-review requirements spec sprint
```

No Windows usando o PowerShell:

```powershell
Set-Location "$HOME\.gemini\config\skills"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue -Path `
  api-design, architecture-reading, database-design, frontend-architecture, `
  grilling, hexagonal-architecture, performance-analysis, project-docs, `
  requirements-engineering, security-audit, spec-writing, technical-writing, `
  testing-contract, `
  audit, bootstrap, closeout, epic, handoff, pr-review, requirements, spec, sprint
```

## 3. Migre regras de workspace para `AGENTS.md`

No Antigravity CLI, regras colocadas em `.agent/rules/*.md` que pertencem ao repositório ou a uma subpasta
devem ficar no `AGENTS.md`. Mova essas instruções para o arquivo `AGENTS.md` correspondente e remova as
regras avulsas. Regras pessoais não comitadas podem permanecer sem alteração.

## 4. Conecte a um vault de memória existente

Se você já possui um vault de memória criado com o Claude Code ou outro harness (por exemplo em `~/ai-memory`),
o tightship utiliza exatamente o mesmo formato markdown em todos os ambientes. Nenhuma conversão é necessária.

Configure `vault_folder` no seu `options.toml`:

```toml
contract = true
memory = true
status_line = true
vault_folder = "~/ai-memory"
```

Na sua próxima sessão, o tightship encontrará suas notas, resumos fixados e handoffs imediatamente.
