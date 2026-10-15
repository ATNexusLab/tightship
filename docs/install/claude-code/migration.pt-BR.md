# Migração da instalação anterior ao plugin

[English](migration.md) · Português

A instalação anterior ao plugin mesclava os arquivos do workflow em `~/.claude`. Com esses arquivos e o plugin
no lugar ao mesmo tempo, cada skill e cada comando carregam duas vezes. Estes passos removem o que a
instalação anterior mesclou, preservam as suas próprias configurações, skills e comandos, e deixam o seu vault
como está.

No Windows, `~/.claude` é `%USERPROFILE%\.claude`. Rode os blocos `sh` no macOS e no Linux, e os blocos
PowerShell no Windows.

## 1. Faça backup de `~/.claude`

```sh
cp -R ~/.claude ~/.claude.before-tightship
```

```powershell
Copy-Item -Recurse "$HOME\.claude" "$HOME\.claude.before-tightship"
```

## 2. Anote a pasta do seu vault

A instalação anterior o guarda em `~/ai-memory`, a menos que você o tenha movido. O passo 6 precisa do
caminho absoluto dele.

## 3. Apague o que a instalação anterior mesclou

Relativo a `~/.claude`:

| O quê | Caminhos |
| --- | --- |
| As 13 skills que o plugin entrega | `skills/api-design`, `skills/architecture-reading`, `skills/database-design`, `skills/frontend-architecture`, `skills/grilling`, `skills/hexagonal-architecture`, `skills/performance-analysis`, `skills/project-docs`, `skills/requirements-engineering`, `skills/security-audit`, `skills/spec-writing`, `skills/technical-writing`, `skills/testing-contract` |
| Os 9 comandos | `commands/audit.md`, `commands/bootstrap.md`, `commands/closeout.md`, `commands/epic.md`, `commands/handoff.md`, `commands/pr-review.md`, `commands/requirements.md`, `commands/spec.md`, `commands/sprint.md` |
| O verificador | `agents/adversarial-verifier.md` |
| Os hooks de memória e o gerador de configurações | `hooks/vault.py`, `hooks/test_vault.py`, `hooks/generate-settings.py`, `hooks/test_generate_settings.py`, `hooks/__pycache__`, `.vault-checkpoint` |
| A status line | `statusline-command.ts`, `package.json`, `bun.lock`, `tsconfig.json`, `node_modules` |
| O setup e as docs dele | `setup.py`, `settings.base.json`, `export-workflow.py`, `SETUP.md`, `README.md`, `AGENTS.md`, `my-plugins.md`, `vault-template`, `themes`, `.gitattributes` |

```sh
cd ~/.claude && rm -rf \
  skills/api-design skills/architecture-reading skills/database-design skills/frontend-architecture \
  skills/grilling skills/hexagonal-architecture skills/performance-analysis skills/project-docs \
  skills/requirements-engineering skills/security-audit skills/spec-writing skills/technical-writing \
  skills/testing-contract \
  commands/audit.md commands/bootstrap.md commands/closeout.md commands/epic.md commands/handoff.md \
  commands/pr-review.md commands/requirements.md commands/spec.md commands/sprint.md \
  agents/adversarial-verifier.md \
  hooks/vault.py hooks/test_vault.py hooks/generate-settings.py hooks/test_generate_settings.py \
  hooks/__pycache__ .vault-checkpoint \
  statusline-command.ts package.json bun.lock tsconfig.json node_modules \
  setup.py settings.base.json export-workflow.py SETUP.md README.md AGENTS.md my-plugins.md \
  vault-template themes .gitattributes
```

```powershell
Set-Location "$HOME\.claude"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue -Path `
  skills\api-design, skills\architecture-reading, skills\database-design, skills\frontend-architecture, `
  skills\grilling, skills\hexagonal-architecture, skills\performance-analysis, skills\project-docs, `
  skills\requirements-engineering, skills\security-audit, skills\spec-writing, skills\technical-writing, `
  skills\testing-contract, `
  commands\audit.md, commands\bootstrap.md, commands\closeout.md, commands\epic.md, commands\handoff.md, `
  commands\pr-review.md, commands\requirements.md, commands\spec.md, commands\sprint.md, `
  agents\adversarial-verifier.md, `
  hooks\vault.py, hooks\test_vault.py, hooks\generate-settings.py, hooks\test_generate_settings.py, `
  hooks\__pycache__, .vault-checkpoint, `
  statusline-command.ts, package.json, bun.lock, tsconfig.json, node_modules, `
  setup.py, settings.base.json, export-workflow.py, SETUP.md, README.md, AGENTS.md, my-plugins.md, `
  vault-template, themes, .gitattributes
```

Depois, à mão:

- Apague o `.gitignore`, a menos que você mantenha `~/.claude` num repositório git seu. Nesse caso, apague só
  as linhas que a instalação anterior adicionou.
- O setup anterior guardou um arquivo que você já tinha, como o seu próprio `README.md`, como
  `<name>.bak`. Devolva o nome original.
- Mantenha o `.i-have-adhd-always`, o interruptor do companheiro i-have-adhd.

Qualquer outra skill, comando ou hook em `~/.claude` é seu, e fica.

## 4. Limpe o `CLAUDE.md`

A instalação anterior escreveu o contrato inteiro em `~/.claude/CLAUDE.md`. Agora o plugin o carrega. Apague
desse arquivo cada linha que a instalação anterior escreveu e mantenha as linhas que você mesmo adicionou. Se
nada sobrar, apague o arquivo.

## 5. Limpe o `settings.json`

Apague de `~/.claude/settings.json` os três hooks que rodam `hooks/vault.py` e o `statusLine` que roda
`statusline-command.ts`. Todas as outras chaves ficam como estão. A instalação anterior já exigia Python,
então este script faz isso por você:

```sh
cd ~/.claude && python3 - <<'PY'
import json
from pathlib import Path

path = Path("settings.json")
settings = json.loads(path.read_text(encoding="utf-8"))
hooks = settings.get("hooks", {})
for event in list(hooks):
    hooks[event] = [
        group for group in hooks[event]
        if not any("vault.py" in hook.get("command", "") for hook in group.get("hooks", []))
    ]
    if not hooks[event]:
        del hooks[event]
if "hooks" in settings and not hooks:
    del settings["hooks"]
if "statusline-command.ts" in settings.get("statusLine", {}).get("command", ""):
    del settings["statusLine"]
path.write_text(json.dumps(settings, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
PY
```

```powershell
Set-Location "$HOME\.claude"
@'
import json
from pathlib import Path

path = Path("settings.json")
settings = json.loads(path.read_text(encoding="utf-8"))
hooks = settings.get("hooks", {})
for event in list(hooks):
    hooks[event] = [
        group for group in hooks[event]
        if not any("vault.py" in hook.get("command", "") for hook in group.get("hooks", []))
    ]
    if not hooks[event]:
        del hooks[event]
if "hooks" in settings and not hooks:
    del settings["hooks"]
if "statusline-command.ts" in settings.get("statusLine", {}).get("command", ""):
    del settings["statusLine"]
path.write_text(json.dumps(settings, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
'@ | python -
```

## 6. Instale o plugin

Siga a [instalação](install.pt-BR.md) e passe a pasta do seu vault, do passo 2, para a instalação:

```sh
claude plugin install tightship@tightship --config vault_folder=/absolute/path/to/your/vault
```

A primeira sessão pergunta se deve mostrar a status line do tightship. Responda sim para tê-la de volta.

## 7. Confira

- `claude plugin list` mostra `tightship@tightship` sem erro.
- Numa sessão, `/hooks` não mostra nenhum hook que rode `vault.py`.
- O início da sessão lista as notas fixadas do seu vault uma vez.
- Cada skill e cada comando aparecem uma vez na sessão.

Quando tudo funcionar, apague `~/.claude.before-tightship`.
