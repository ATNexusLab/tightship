# Migration from the pre-plugin install

English · [Português](migration.pt-BR.md)

The pre-plugin install merged the workflow's files into `~/.claude`. With those files and the plugin both in
place, every skill and command loads twice. These steps remove what the pre-plugin install merged, keep your
own settings, skills, and commands, and keep your vault as it is.

On Windows, `~/.claude` is `%USERPROFILE%\.claude`. Run the `sh` blocks on macOS and Linux, and the PowerShell
blocks on Windows.

## 1. Back up `~/.claude`

```sh
cp -R ~/.claude ~/.claude.before-tightship
```

```powershell
Copy-Item -Recurse "$HOME\.claude" "$HOME\.claude.before-tightship"
```

## 2. Note your vault's folder

The pre-plugin install keeps it at `~/ai-memory`, unless you moved it. Step 6 needs its absolute path.

## 3. Delete what the pre-plugin install merged

Relative to `~/.claude`:

| What | Paths |
| --- | --- |
| The 13 skills the plugin ships | `skills/api-design`, `skills/architecture-reading`, `skills/database-design`, `skills/frontend-architecture`, `skills/grilling`, `skills/hexagonal-architecture`, `skills/performance-analysis`, `skills/project-docs`, `skills/requirements-engineering`, `skills/security-audit`, `skills/spec-writing`, `skills/technical-writing`, `skills/testing-contract` |
| The 9 commands | `commands/audit.md`, `commands/bootstrap.md`, `commands/closeout.md`, `commands/epic.md`, `commands/handoff.md`, `commands/pr-review.md`, `commands/requirements.md`, `commands/spec.md`, `commands/sprint.md` |
| The verifier | `agents/adversarial-verifier.md` |
| The memory hooks and the settings generator | `hooks/vault.py`, `hooks/test_vault.py`, `hooks/generate-settings.py`, `hooks/test_generate_settings.py`, `hooks/__pycache__`, `.vault-checkpoint` |
| The status line | `statusline-command.ts`, `package.json`, `bun.lock`, `tsconfig.json`, `node_modules` |
| The setup and its docs | `setup.py`, `settings.base.json`, `export-workflow.py`, `SETUP.md`, `README.md`, `AGENTS.md`, `my-plugins.md`, `vault-template`, `themes`, `.gitattributes` |

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

Then, by hand:

- Delete `.gitignore`, unless you keep `~/.claude` in a git repository of your own. Then delete only
  the lines the pre-plugin install added.
- The pre-plugin setup kept a file you already had, such as your own `README.md`, as
  `<name>.bak`. Rename it back.
- Keep `.i-have-adhd-always`, the i-have-adhd companion's switch.

Any other skill, command, or hook in `~/.claude` is yours, and stays.

## 4. Clean `CLAUDE.md`

The pre-plugin install wrote the whole contract into `~/.claude/CLAUDE.md`. The plugin now loads it. Delete
from that file every line the pre-plugin install wrote, and keep the lines you added yourself. If nothing is
left, delete the file.

## 5. Clean `settings.json`

Delete from `~/.claude/settings.json` the three hooks that run `hooks/vault.py`, and the `statusLine` that
runs `statusline-command.ts`. Every other key stays as it is. The pre-plugin install already required Python,
so this does it for you:

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

## 6. Install the plugin

Follow [installation](install.md), and pass your vault's folder from step 2 to the install:

```sh
claude plugin install tightship@tightship --config vault_folder=/absolute/path/to/your/vault
```

The first session asks whether to show tightship's status line. Say yes to get it back.

## 7. Check it

- `claude plugin list` shows `tightship@tightship` with no error.
- In a session, `/hooks` shows no hook that runs `vault.py`.
- The session start lists your vault's pinned notes once.
- Each skill and command appears once in the session.

Once everything works, delete `~/.claude.before-tightship`.
