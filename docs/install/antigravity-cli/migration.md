# Migration

English · [Português](migration.pt-BR.md)

How to move to the tightship Antigravity CLI plugin from loose skills or rules, and how to connect to an
existing vault created with Claude Code or another harness.

## 1. Back up `~/.gemini`

Before cleaning up loose files, make a backup of your configuration directory:

```sh
cp -R ~/.gemini ~/.gemini.before-tightship
```

On Windows in PowerShell:

```powershell
Copy-Item -Recurse "$HOME\.gemini" "$HOME\.gemini.before-tightship"
```

## 2. Remove loose duplicate skills

If you previously copied tightship's skills or commands into `~/.gemini/config/skills/`, they will load twice
alongside the plugin. Remove the loose copies, keeping your personal skills and `humanizer`:

```sh
cd ~/.gemini/config/skills && rm -rf \
  api-design architecture-reading database-design frontend-architecture \
  grilling hexagonal-architecture performance-analysis project-docs \
  requirements-engineering security-audit spec-writing technical-writing \
  testing-contract \
  audit bootstrap closeout epic handoff pr-review requirements spec sprint
```

On Windows in PowerShell:

```powershell
Set-Location "$HOME\.gemini\config\skills"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue -Path `
  api-design, architecture-reading, database-design, frontend-architecture, `
  grilling, hexagonal-architecture, performance-analysis, project-docs, `
  requirements-engineering, security-audit, spec-writing, technical-writing, `
  testing-contract, `
  audit, bootstrap, closeout, epic, handoff, pr-review, requirements, spec, sprint
```

## 3. Migrate workspace rules into `AGENTS.md`

In Antigravity CLI, rules placed in `.agent/rules/*.md` that apply to an entire repository or sub-area
belong in `AGENTS.md`. Move those instructions into the corresponding `AGENTS.md` file and remove the loose
rules. Personal uncommitted rules can remain untouched.

## 4. Connect to an existing memory vault

If you already have a memory vault created with Claude Code or another harness (for example at `~/ai-memory`),
tightship uses the exact same markdown format across all harnesses. No conversion is needed.

Set `vault_folder` in your `options.toml`:

```toml
contract = true
memory = true
status_line = true
vault_folder = "~/ai-memory"
```

In your next session, tightship will find your existing notes, pinned summaries, and handoffs immediately.
