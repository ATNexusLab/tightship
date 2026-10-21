# #59 — Build, install, and run the Antigravity CLI plugin

> **Status:** published
> **Profile:** API
> **Module:** `harnesses/antigravity-cli/`, `xtask/`, `tightship/`, `installer/`,
> `.github/workflows/`, `docs/install/antigravity-cli/`
> **Epic:** #59
> **Requirements:** NFR-FLEX-03, NFR-FLEX-07, NFR-COMP-01, FR-INS-01 to FR-INS-08, FR-CTR-01 to FR-CTR-03,
> FR-CTR-05, FR-SKL-01 to FR-SKL-03, FR-VER-01, FR-VER-02, FR-MEM-01 to FR-MEM-11, FR-STL-01 to FR-STL-03,
> BR-01, BR-03, BR-04
> **Sprint:** 2

The workflow builds and ships as a native Antigravity CLI (`agy`) plugin from the same canonical source
in `content/`, without changing a single file in `content/`. A unified one-command installer
(`install.sh` on macOS, Linux, and Git Bash; `install.ps1` on Windows PowerShell) guides the adopter
through an interactive terminal dialog to pick the harness(es) and options, installs the three companion
plugins and `tightship`, and writes the options and status line configuration.

Terms from the SRS, as used here:

- **Harness**: a coding-agent host that loads plugins; this epic adds Antigravity CLI (`agy`) as the
  milestone M3 harness ([glossary](../../requirements/glossary.md),
  [NFR-FLEX-03](../../requirements/non-functional/flexibility.md#nfr-flex-03)).
- **Canonical content**: the one harness-neutral copy of the contract, skills, commands, subagent,
  options, and vault template in `content/` ([NFR-MNT-01](../../requirements/non-functional/maintainability.md#nfr-mnt-01),
  [NFR-FLEX-06](../../requirements/non-functional/flexibility.md#nfr-flex-06)).
- **Per-harness place**: `harnesses/<harness>/` and the harness's renderer in `xtask/src/build/`, the
  only files that may name a harness ([ADR 0001](../../decisions/0001-rust-build-and-tokenized-canonical-content.md),
  [NFR-FLEX-07](../../requirements/non-functional/flexibility.md#nfr-flex-07)).
- **Companion plugins**: ponytail, i-have-adhd, and humanizer, required alongside the plugin
  ([BR-04](../../requirements/business-rules.md#br-04),
  [FR-INS-02](../../requirements/functional/install.md#fr-ins-02)).
- **Option**: a part of the plugin the adopter turns on or off (`contract`, `memory`, `status_line`) or
  points at a directory (`vault_folder`) ([FR-INS-03](../../requirements/functional/install.md#fr-ins-03),
  [FR-INS-04](../../requirements/functional/install.md#fr-ins-04),
  [FR-INS-05](../../requirements/functional/install.md#fr-ins-05)).
- **Contract**: the always-on rules loaded into every session while `contract` is enabled
  ([FR-CTR-01](../../requirements/functional/contract.md#fr-ctr-01),
  [FR-CTR-02](../../requirements/functional/contract.md#fr-ctr-02)).
- **Verifier**: the read-only subagent `adversarial-verifier` that tries to refute a claim and may edit
  files only in a private copy through its shell
  ([FR-VER-01](../../requirements/functional/verifier.md#fr-ver-01),
  [FR-VER-02](../../requirements/functional/verifier.md#fr-ver-02)).
- **Vault**: the adopter's git repository of curated notes and handoffs, shared across harnesses on the
  same machine with zero conversion steps ([FR-MEM-01](../../requirements/functional/memory.md#fr-mem-01),
  [NFR-COMP-01](../../requirements/non-functional/compatibility.md#nfr-comp-01)).
- **Status line**: the two-row status display under the prompt, written into the harness's user settings
  only with the adopter's consent and restored when turned off
  ([FR-STL-01](../../requirements/functional/status-line.md#fr-stl-01),
  [FR-STL-02](../../requirements/functional/status-line.md#fr-stl-02),
  [FR-STL-03](../../requirements/functional/status-line.md#fr-stl-03)).
- **Pre-plugin install**: the workflow merged into the adopter's user customization directory before the
  plugin existed ([FR-INS-07](../../requirements/functional/install.md#fr-ins-07)).

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-16, verified against Antigravity CLI (`agy`) 1.4.0:

1. **Minimum version and build/release artifacts (`OQ-04`, `NFR-FLEX-03`).** The minimum Antigravity CLI
   version is `1.4.0`. `cargo xtask build` renders `dist/antigravity-cli/` alongside
   `dist/claude-code/`, and `cargo xtask package <tag>` packs `dist/tightship-antigravity-cli.zip`
   alongside `dist/tightship-claude-code.zip` and `dist/marketplace.json`. Bilingual install guides live
   in `docs/install/antigravity-cli/` (`.md` and `.pt-BR.md`), and the release is never published
   before M2 (`BR-01`).
2. **Native plugin layout and commands as skills (`FR-SKL-01`, `FR-SKL-02`, `FR-SKL-03`).**
   `dist/antigravity-cli/plugin.json` and `dist/antigravity-cli/hooks.json` sit at the plugin root, which
   passes `agy plugin validate <dir> --strict`. Native Antigravity plugins have no `commands/` directory;
   every skill in `skills/<name>/SKILL.md` is natively a `/<name>` slash command in `agy`. Both
   `content/skills/<name>/SKILL.md` and `content/commands/<name>.md` render to
   `dist/antigravity-cli/skills/<name>/SKILL.md`. Piece references are unprefixed (`{name}` for skills
   and subagents, `/{name}` for commands).
3. **Contract delivery via `SessionStart` hooks and on-demand `/contract` skill (`OQ-06`, `FR-CTR-01`,
   `FR-CTR-02`, `FR-CTR-03`).** Antigravity CLI truncates each `rules/*.md` file at 24,000 bytes (while
   the rendered contract is ~24,900 bytes) and loads `rules/` unconditionally. Instead, `SessionStart`
   hooks in `hooks.json` inject the contract parts in Antigravity's JSON hook output format
   (`{"injectSteps": [{"systemMessage": {"systemMessage": "..."}}]}`), gated by `contract` and `memory`.
   Because `SessionStart` in `agy` fires once when a conversation starts and not after `/compact`, the
   build also writes `dist/antigravity-cli/skills/contract/SKILL.md` and appends a reload line to the
   injected contract header so the agent reloads `skills/contract/SKILL.md` after compaction.
4. **Options stored in the plugin's data folder, `options.toml` (`FR-INS-03`, `FR-INS-04`, `FR-INS-05`).**
   `agy plugin validate --strict` rejects `userConfig` in `plugin.json` and `agy` sets no
   `CLAUDE_PLUGIN_OPTION_*` variables. It preserves the plugin's data folder,
   `~/.gemini/config/plugin_data/tightship`, across updates, but gives a hook neither `PLUGIN_DATA` nor
   `PLUGIN_ROOT` (probed in 1.4.2, #89): the launchers fall back to that folder and to their own parent.
   The adopter's options (`contract`, `memory`, `status_line`, `vault_folder`) live in
   `options.toml` there. The `tightship` binary reads
   `${TIGHTSHIP_DATA}/options.toml` whenever the corresponding `TIGHTSHIP_*` environment variable is not
   set, and the Antigravity launchers (`bin/tightship` and `bin/tightship.cmd`) set `TIGHTSHIP_ROOT`,
   `TIGHTSHIP_DATA`, and `TIGHTSHIP_SETTINGS` when invoked by hooks or by the agent's `run_command` tool.
5. **Status line in `~/.gemini/antigravity-cli/settings.json` (`OQ-08`, `FR-STL-01`, `FR-STL-02`,
   `FR-STL-03`).** Antigravity CLI reads `"statusLine": {"type": "command", "command": "..."}` from
   `~/.gemini/antigravity-cli/settings.json` and pipes the same JSON schema (`workspace`, `model`,
   `context_window`, etc.) to `tightship status-line render` as Claude Code. `tightship hook status-line`
   writes or restores `statusLine` in `TIGHTSHIP_SETTINGS` and records the previous value in
   `${TIGHTSHIP_DATA}/status-line.json`.
6. **Unified interactive one-command installer (`install.sh` and `install.ps1`) (`OQ-05`, `FR-INS-01`,
   `FR-INS-02`, `FR-INS-03`, `FR-STL-02`, amending `ADR 0001`).** Versioned at the repository root and
   attached to every release, `install.sh` (`curl -fsSL .../install.sh | sh`) and `install.ps1`
   (`irm .../install.ps1 | iex`) open an interactive terminal dialog (or accept CLI flags) to choose
   which harness(es) to install into (`claude-code`, `antigravity-cli`, or both) and which options to
   enable (`contract`, `memory`, `status_line`, `vault_folder`), then install the three companions, the
   plugin, and the chosen options in one command.

## Acceptance Criteria

### Contract

| Command or trigger | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| `cargo xtask build [--binaries <folder>]` | Maintainer or CI | Yes | Renders `dist/claude-code/` and `dist/antigravity-cli/` from `content/` and `harnesses/` |
| `cargo xtask package <tag>` | Release pipeline | Yes | Packs `dist/tightship-claude-code.zip`, `dist/tightship-antigravity-cli.zip`, and `dist/marketplace.json` |
| `sh install.sh [flags]` or `pwsh -File install.ps1 [flags]` | Adopter | Yes | Prompts for (or reads from flags) harness(es) and options, installs the 3 companions and `tightship`, writes options, and applies the status line |
| Conversation start in `agy` (`SessionStart`) | Adopter, through `agy` | No: may update `settings.json` and `status-line.json` | Injects the contract (`hook contract`) and memory context (`hook start`) as `injectSteps` JSON, and syncs the status line (`hook status-line`) |
| Each prompt in `agy` (`PreInvocation`) | Adopter, through `agy` | No: increments the session prompt counter | Increments `${TIGHTSHIP_DATA}/checkpoints/<conversationId>` and injects the checkpoint on every 10th prompt |
| `/contract` (or loading the `contract` skill after `/compact`) | Adopter or agent | Yes | Loads `skills/contract/SKILL.md` with the full contract text into the session |
| `tightship vault ...` | Agent, through `run_command` | Per subcommand | Resolves `vault_folder` from `${TIGHTSHIP_DATA}/options.toml` when `TIGHTSHIP_VAULT` is unset |

### Request

#### `harnesses/antigravity-cli/harness.toml`

New file. Same schema as `harnesses/claude-code/harness.toml` (`xtask/src/build/harness.rs`), with one
new optional boolean key `commands_as_skills` on `[references]` (or rendered directly by
`xtask/src/build/antigravity_cli.rs`):

| Section / Key | Type | Required | Value |
| --- | --- | --- | --- |
| `plugin.name` | string | Yes | `"tightship"` |
| `plugin.description` | string | Yes | `"Skills, a command per step of the development loop, and a read-only verifier"` |
| `plugin.author` | string | Yes | `"ATNexusLab"` |
| `plugin.repository` | string | Yes | `"https://github.com/ATNexusLab/tightship"` |
| `plugin.license` | string | Yes | `"MIT"` |
| `references.skill` | string | Yes | `"{name}"` |
| `references.command` | string | Yes | `"/{name}"` |
| `references.agent` | string | Yes | `"{name}"` |
| `references.option` | string | Yes | `"{name}"` |
| `access.read-only` | string | Yes | `"view_file, list_dir, find_by_name, grep_search, run_command"` |
| `contract.limit` | integer | Yes | `100000` |
| `capabilities.arguments` | string | Yes | `"$ARGUMENTS"` |
| `capabilities.plan-mode` | string | Yes | `"plan mode"` |
| `capabilities.plan-approval` | string | Yes | `"an artifact with \`RequestFeedback: true\`"` |
| `capabilities.explore-agent` | string | Yes | `"the \`research\` subagent"` |
| `capabilities.general-agent` | string | Yes | `"the \`self\` subagent"` |
| `capabilities.review-command` | string | Yes | `"\`/pr-review\`"` |
| `capabilities.security-review-command` | string | Yes | `"the \`security-audit\` skill"` |
| `capabilities.harness-instructions-file` | string | Yes | `"\`AGENTS.md\`"` |
| `capabilities.area-instructions` | string | Yes | `"Antigravity CLI loads a directory's \`AGENTS.md\` when it works on files under it"` |
| `capabilities.skill-tool` | string | Yes | `"\`view_file\` on the skill's \`SKILL.md\`"` |
| `capabilities.instruction-files-precedence` | string | Yes | `"Antigravity CLI reads \`AGENTS.md\` and \`.agent/rules/*.md\` in the workspace"` |
| `capabilities.instruction-files-search` | string | Yes | `"\`find . -name AGENTS.md -o -path '*/.agent/rules/*.md'\`"` |
| `capabilities.instruction-files-migration` | string | Yes | `"Rules in \`.agent/rules/*.md\` that belong to the whole project or an area move into that directory's \`AGENTS.md\`, and the \`.agent/rules/*.md\` file is removed. A personal uncommitted rule stays untouched until I ask whether it belongs in the repo."` |
| `neutrality.names` | array of strings | Yes | `["Antigravity CLI", "Antigravity", "Google DeepMind"]` |
| `neutrality.paths` | array of strings | Yes | `[".gemini/", "~/.gemini", ".agent/", ".agents/", ".antigravity-plugin", ".gemini-plugin", "PLUGIN_ROOT", "PLUGIN_DATA"]` |
| `neutrality.tools` | array of strings | Yes | `["invoke_subagent", "define_subagent", "manage_subagents", "ask_question", "view_file", "replace_file_content", "multi_replace_file_content", "write_to_file", "run_command", "manage_task"]` |
| `companions` | array of tables | Yes | `ponytail` (`https://github.com/DietrichGebert/ponytail`), `i-have-adhd` (`https://github.com/ayghri/i-have-adhd`), `humanizer` (`https://github.com/blader/humanizer`) |

#### `${PLUGIN_DATA}/options.toml`

Optional TOML file at `~/.gemini/config/plugin_data/tightship/options.toml` (where `agy` sets
`PLUGIN_DATA`). Every key is optional; an absent file or an absent boolean key means `true`:

| Key | Type | Required | Domain / Validation |
| --- | --- | --- | --- |
| `contract` | boolean | No | `true` or `false`; defaults to `true` when absent |
| `memory` | boolean | No | `true` or `false`; defaults to `true` when absent |
| `status_line` | boolean | No | `true` or `false`; defaults to `true` when absent |
| `vault_folder` | string | No | Absolute path or `~`/`~/...` path to the vault folder; empty or absent means not set |

```toml
contract = true
memory = true
status_line = true
vault_folder = "~/ai-memory"
```

#### `tightship hook contract [--gate <option>] <file>`

New subcommand on `tightship hook`:

| Argument / Variable | Type | Required | Validation |
| --- | --- | --- | --- |
| `--gate <option>` | string | No | Name of an additional boolean option in `${TIGHTSHIP_DATA}/options.toml` that must not be `false` |
| `<file>` | path | Yes | Path to the rendered contract part file |
| `TIGHTSHIP_DATA` | path (env) | No | Folder holding `options.toml` |
| `TIGHTSHIP_HOOK_OUTPUT` | path (env) | No | A JSON file whose `"{text}"` slot non-empty stdout fills, such as `{"injectSteps":[{"systemMessage":{"systemMessage":"{text}"}}]}`; absent or empty prints plain text |

#### `install.sh` and `install.ps1` arguments and interactive prompts

Both scripts accept the same optional flags; any flag omitted is asked interactively when a terminal is
available (`/dev/tty` on POSIX, `Host.UI` on PowerShell), or uses its default when `--yes` is passed:

| Flag | Type | Required | Domain / Default |
| --- | --- | --- | --- |
| `--harness <list>` | string | No | Comma-separated subset of `claude-code`, `antigravity-cli`, or `all`. Interactive prompt when omitted |
| `--contract <bool>` | string | No | `true` or `false`; default `true` |
| `--memory <bool>` | string | No | `true` or `false`; default `true` |
| `--status-line <bool>` | string | No | `true` or `false`; default `true` |
| `--vault-folder <path>` | path | No | Absolute path or `~/...` path; default empty |
| `--init-vault` | flag | No | Creates `<vault-folder>` with `tightship vault init` when it does not exist yet |
| `--from-dir <folder>` | path | No | Installs from a local `dist/` directory instead of downloading the installer's own release |
| `--yes` / `-y` | flag | No | Accepts defaults for any omitted flag without prompting |

### Response

#### `dist/antigravity-cli/` layout

```text
dist/antigravity-cli/
  plugin.json
  hooks.json
  agents/
    adversarial-verifier.md
  bin/
    tightship
    tightship.cmd
    <platform>/tightship[.exe]
  contract/
    part-1.md
    memory.md
  skills/
    contract/SKILL.md
    <13 canonical skills>/SKILL.md (+ supporting files)
    <10 canonical commands>/SKILL.md
```

- `plugin.json`:

```json
{
  "name": "tightship",
  "version": "1.0.2",
  "description": "Skills, a command per step of the development loop, and a read-only verifier",
  "author": {
    "name": "ATNexusLab"
  },
  "repository": "https://github.com/ATNexusLab/tightship",
  "license": "MIT"
}
```

- `agents/adversarial-verifier.md` front matter:

```yaml
---
name: "adversarial-verifier"
description: "..."
tools:
  - view_file
  - list_dir
  - find_by_name
  - grep_search
  - run_command
subagent: true
mainAgent: false
model: inherit
---
```

- `hooks.json`, with `bin\tightship` in `hooks.windows.json` (#89):

```json
{
  "tightship": {
    "SessionStart": [
      { "type": "command", "command": "bin/tightship hook contract-part part-1.md" },
      { "type": "command", "command": "bin/tightship hook contract-part memory.md" },
      { "type": "command", "command": "bin/tightship hook start" },
      { "type": "command", "command": "bin/tightship hook status-line" }
    ],
    "PreInvocation": [
      { "type": "command", "command": "bin/tightship hook prompt" }
    ]
  }
}
```

- `cargo xtask build` stdout:

```text
Built dist/claude-code/: 24 pieces (13 skills, 10 commands, 1 agent), 32 files.
Try it: claude --plugin-dir dist/claude-code
Built dist/antigravity-cli/: 24 pieces (13 skills, 10 commands, 1 agent), 33 files.
Try it: agy plugin validate dist/antigravity-cli --strict
```

- `cargo xtask package <tag>` stdout:

```text
Packed dist/tightship-claude-code.zip (34 files), dist/tightship-antigravity-cli.zip (36 files), and dist/marketplace.json for v1.0.2.
sha256 (claude-code): <64 lowercase hex digits>
sha256 (antigravity-cli): <64 lowercase hex digits>
```

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | Runs `cargo xtask build`, pushes release tags, promotes releases after the checklist |
| Release pipeline | `GITHUB_TOKEN` with `contents: write` | Uploads `tightship-claude-code.zip`, `tightship-antigravity-cli.zip`, `marketplace.json`, `install.sh`, and `install.ps1` |
| Adopter | Local user account on macOS, Linux, or Windows | Runs `install.sh` / `install.ps1` or `agy plugin install`; owns `~/.gemini/` and `~/.claude/` |
| Verifier subagent (`adversarial-verifier`) | `view_file`, `list_dir`, `find_by_name`, `grep_search`, `run_command` | Read-only harness tools; no `replace_file_content`, `multi_replace_file_content`, or `write_to_file` |

---

## Business Rules

### 1. Authorization and tool restriction (`BR-01`, `FR-VER-01`, `FR-VER-02`)

- Only a maintainer with repository write access pushes a `v<version>` release tag or promotes a
  pre-release (`BR-01`).
- In Antigravity CLI, `agents/adversarial-verifier.md` declares `subagent: true`, `mainAgent: false`,
  `model: inherit`, and the five tools `view_file`, `list_dir`, `find_by_name`, `grep_search`, and
  `run_command` (`FR-VER-01`, `FR-VER-02`). It receives no file-editing tool of the harness.

### 2. Multi-harness build and packaging (`NFR-FLEX-03`, `NFR-FLEX-07`)

- **When:** `cargo xtask build` or `cargo xtask build --binaries <folder>` runs.
- It reads `content/` and every `harnesses/<name>/harness.toml`, checks references, harness neutrality,
  options, and contract limits across **every** harness in `harnesses/` before compiling or copying
  binaries, then renders `dist/claude-code/` and `dist/antigravity-cli/`, scans both staging directories
  for `.identity-terms`, and renames each into `dist/<harness>/` only if every check passed.
- If any check fails for either harness, neither `dist/claude-code/` nor `dist/antigravity-cli/` is
  published.
- **When:** `cargo xtask package <tag>` runs.
- It checks both `dist/claude-code/` and `dist/antigravity-cli/` for the four platform binaries and a
  `plugin.json` whose `version` matches `<tag>` (`v<version>`), writes `dist/tightship-claude-code.zip`,
  `dist/tightship-antigravity-cli.zip` (with `0755` on `bin/tightship`, `bin/tightship.cmd`, and the
  four binaries, `0644` on all other files), and `dist/marketplace.json`.

### 3. Antigravity CLI pieces, contract skill, and launchers (`FR-SKL-01`..`FR-SKL-03`, `FR-CTR-01`, `FR-CTR-05`)

- **When:** `xtask` renders `dist/antigravity-cli/`.
- Every skill in `content/skills/<name>/SKILL.md` and every command in `content/commands/<name>.md`
  renders to `dist/antigravity-cli/skills/<name>/SKILL.md` with YAML front matter (`name` and
  `description`; no `argument-hint`, because `agy plugin validate --strict` rejects unknown front matter
  keys if strict schema checks apply, or keeps standard skill front matter). Supporting files copy to
  `dist/antigravity-cli/skills/<skill>/<relative>`.
- When `content/contract.md` exists, `xtask` also writes `dist/antigravity-cli/skills/contract/SKILL.md`
  with `name: "contract"` and `description: "Reload the workflow's global working contract"`, whose body
  is the rendered `content/contract.md` (without its `+++` front matter), and prepends the line:
  `After any context compaction (/compact), load the contract skill to restore this contract in full.`
  immediately under the level-1 heading of `contract/part-1.md`.
- `dist/antigravity-cli/bin/tightship` (POSIX `sh`) and `dist/antigravity-cli/bin/tightship.cmd`
  (Windows `cmd`) set `TIGHTSHIP_ROOT` (to `${PLUGIN_ROOT}` or the parent of `bin/`), `TIGHTSHIP_DATA`
  (to `${PLUGIN_DATA}` or `<config_dir>/plugin_data/tightship`), `TIGHTSHIP_SETTINGS` (to
  `${AGY_CONFIG_DIR:-$HOME/.gemini/antigravity-cli}/settings.json` on POSIX and
  `%USERPROFILE%\.gemini\antigravity-cli\settings.json` on Windows), and
  `TIGHTSHIP_HOOK_OUTPUT` (the plugin's `hook-output.json`) before executing the platform binary.

### 4. Options in `${TIGHTSHIP_DATA}/options.toml` and JSON hook output (`FR-INS-03`..`05`)

- **When:** `tightship` runs `vault ...`, `hook contract`, `hook start`, `hook export`, `hook checkpoint`,
  `hook prompt`, or `hook status-line`.
- If `${TIGHTSHIP_DATA}/options.toml` exists, `tightship` parses it as a TOML table. A malformed
  `options.toml` fails with:
  > `"error: <path> is not valid TOML"`
- An option's effective value is the `TIGHTSHIP_*` environment variable when set and non-empty, or the
  key in `${TIGHTSHIP_DATA}/options.toml` otherwise:
  - `contract`: off only when `false` (`TIGHTSHIP_CONTRACT=false` or `contract = false`).
  - `memory`: off only when `false` (`TIGHTSHIP_MEMORY=false` or `memory = false`).
  - `status_line`: off only when `false` (`TIGHTSHIP_STATUS_LINE=false` or `status_line = false`).
  - `vault_folder`: `TIGHTSHIP_VAULT` when set and non-empty, else `vault_folder` in `options.toml`. A
    value of `~` or `~/...` (or `~\...` on Windows) expands to the user's home directory (`#60`).
- When `memory` is `false`, `hook start`, `hook export`, `hook checkpoint`, and `hook prompt` exit `0`
  and print nothing.
- When `TIGHTSHIP_HOOK_OUTPUT` names `hook-output.json`, `hook contract`, `hook start`, `hook checkpoint`,
  and `hook prompt` put any non-empty output string `<text>` in its `"{text}"` slot:
  `{"injectSteps":[{"systemMessage":{"systemMessage":"<text>"}}]}`
  followed by a newline, and print nothing (0 bytes) when the output is empty.

### 5. Prompt checkpoint counter on `conversationId` (`FR-MEM-02`)

- **When:** `tightship hook prompt` reads JSON from `stdin`.
- It extracts `"session_id"` if present and non-empty, or `"conversationId"` if present and non-empty
  (the key Antigravity CLI passes to `PreInvocation` hooks), sanitizing non-`[A-Za-z0-9_-]` characters to
  `-`, or `"unknown"` when neither is present.

### 6. Status line command on Windows and POSIX (`FR-STL-01`..`FR-STL-03`)

- **When:** `tightship hook status-line` or `tightship status-line enable` writes `statusLine` into
  `TIGHTSHIP_SETTINGS`.
- On Windows (`cfg!(windows)`), `settings::command_for(root)` formats
  `<root>\bin\tightship.cmd status-line render`, unquoted, when `<root>\bin\tightship.cmd` exists: Go
  escapes every `"` as `\"` under `cmd /c`, which `cmd` cannot read (#89). A path with a space or one of
  `& ^ ( ) % ! ; = ,` becomes its 8.3 short name, read from `cmd /u`; when none avoids them the hook
  fails and names the path. Otherwise it formats
  `sh "<root>/bin/tightship" status-line render`. `settings::is_ours` matches the launcher or the binary,
  quoted or not, after `sh` or not, with `/` or `\` between folders.

### 7. Unified interactive installer (`FR-INS-01`..`03`, `FR-STL-02`, `BR-04`)

- **When:** the adopter runs `install.sh` or `install.ps1`.
- If `--harness` is omitted, the script checks `PATH` for `claude` and `agy` and prompts:
  > `"Install tightship for: [1] Claude Code [2] Antigravity CLI [3] Both (default: <detected>): "`
- For each selected harness, it checks the harness CLI is on `PATH` and meets the minimum version
  (`2.1.295` for `claude`, `1.4.0` for `agy`). If missing or too old, it exits `1` with:
  - > `"error: claude 2.1.295 or later is required; found <version or 'not installed'>"`
  - > `"error: agy 1.4.0 or later is required; found <version or 'not installed'>"`
- For any option flag omitted (unless `--yes` is passed), it prompts:
  - > `"Enable the working contract in every session? [Y/n]: "`
  - > `"Enable the memory layer (curated vault notes and handoffs)? [Y/n]: "`
  - When `memory` is `true`:
    > `"Vault folder path (for example ~/ai-memory, or leave empty to set later): "`
  - When `vault_folder` is non-empty and the folder does not exist yet:
    > `"Vault folder <path> does not exist. Create it now with tightship vault init? [Y/n]: "`
  - > `"Enable tightship's status line (replaces your current statusLine and restores it when off)? [Y/n]: "`
- For **Claude Code**, the script runs:
  1. `claude plugin marketplace add DietrichGebert/ponytail`
  2. `claude plugin marketplace add ayghri/i-have-adhd`
  3. `claude plugin marketplace add blader/humanizer`
  4. `claude plugin marketplace add ATNexusLab/tightship`
  5. `claude plugin install tightship@tightship` with `--config contract=<bool>`,
     `--config memory=<bool>`, `--config status_line=<bool>`, and (when non-empty)
     `--config vault_folder=<expanded_path>`.
- For **Antigravity CLI**, the script runs:
  1. `agy plugin install https://github.com/DietrichGebert/ponytail`
  2. `agy plugin install https://github.com/ayghri/i-have-adhd`
  3. Downloads `https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md` to
     `~/.gemini/config/skills/humanizer/SKILL.md` (creating the directory if needed).
  4. Downloads `tightship-antigravity-cli.zip` from the release the installer belongs to (or copies
     `--from-dir`), checks it against the `sha256` that release wrote into the installer, extracts it to a
     temporary directory, and runs `agy plugin install <temp_dir>` (which installs into
     `~/.gemini/config/plugins/tightship`). The installer at the repository root belongs to no release:
     without `--from-dir` it stops before installing anything.
  5. Writes `~/.gemini/config/plugin_data/tightship/options.toml` with `contract`, `memory`,
     `status_line`, and (when non-empty) `vault_folder`.
  6. Runs `~/.gemini/config/plugins/tightship/bin/tightship hook status-line` so the status line setting
     in `~/.gemini/antigravity-cli/settings.json` reflects `status_line` immediately.
- If the adopter chose to initialize a missing `vault_folder`, the script runs
  `tightship vault init <vault_folder>`.
- On completion, it prints:
  > `"Installed tightship for <harnesses>."`

### 8. Persistence and audit (`FR-INS-01`, `FR-INS-04`, `FR-STL-03`)

- **Files changed on the adopter's machine (Antigravity CLI):**
  - `~/.gemini/config/plugins/tightship/` (the installed plugin)
  - `~/.gemini/config/plugins/ponytail/` and `~/.gemini/config/plugins/i-have-adhd/` (companions)
  - `~/.gemini/config/skills/humanizer/SKILL.md` (the `humanizer` companion skill)
  - `~/.gemini/config/plugin_data/tightship/options.toml` (the adopter's option values)
  - `~/.gemini/config/plugin_data/tightship/status-line.json` (saved previous `statusLine`)
  - `~/.gemini/config/plugin_data/tightship/checkpoints/<conversationId>` (prompt counter)
  - `~/.gemini/antigravity-cli/settings.json` (`statusLine` entry when enabled)
- **Audit:** N/A — local CLI tool and plugin; no server or audit log (`NFR-SEC-01`).
- **Events/integrations fired:** GitHub Actions `.github/workflows/release.yml` uploads
  `tightship-antigravity-cli.zip`, `install.sh`, and `install.ps1` alongside `tightship-claude-code.zip`
  and `marketplace.json`.

---

## Errors

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `PACKAGE_PLUGIN_MISSING` | `1` | `cargo xtask package <tag>` runs when `dist/claude-code` or `dist/antigravity-cli` is missing | `"error: dist/<harness>/<manifest> is missing; run cargo xtask build --binaries <folder> first"` |
| `PACKAGE_BINARY_MISSING` | `1` | `cargo xtask package <tag>` runs when any of the 4 platform binaries is missing in `dist/<harness>/bin/` | `"error: dist/<harness>/bin/<platform>/<binary> is missing; run cargo xtask build --binaries <folder> first"` |
| `OPTIONS_TOML_INVALID` | `1` | `${TIGHTSHIP_DATA}/options.toml` exists and is not valid TOML or has a non-boolean `contract`, `memory`, or `status_line` | `"error: <path> is not valid TOML"` |
| `OPTIONS_TOML_UNREADABLE` | `1` | `${TIGHTSHIP_DATA}/options.toml` exists and cannot be read | `"error: cannot read <path>: <io error>"` |
| `CONTRACT_FILE_UNREADABLE` | `1` | `tightship hook contract <file>` cannot read `<file>` | `"error: cannot read <file>: <io error>"` |
| `INSTALL_HARNESS_UNKNOWN` | `1` | `install.sh` or `install.ps1` receives an unknown `--harness` value | `"error: unknown harness <value>; expected antigravity-cli, claude-code, or all"` |
| `INSTALL_CLAUDE_MISSING` | `1` | `claude-code` selected and `claude` is missing or `< 2.1.295` | `"error: claude 2.1.295 or later is required; found <version or 'not installed'>"` |
| `INSTALL_AGY_MISSING` | `1` | `antigravity-cli` selected and `agy` is missing or `< 1.4.0` | `"error: agy 1.4.0 or later is required; found <version or 'not installed'>"` |
| `INSTALL_NOT_A_RELEASE` | `1` | `antigravity-cli` selected, no `--from-dir`, and the installer is the repository's, not a release's | `"error: this <installer> belongs to no release; run https://github.com/ATNexusLab/tightship/releases/latest/download/<installer>, or pass --from-dir <folder>"` |
| `INSTALL_ZIP_SHA256_MISMATCH` | `1` | The downloaded `tightship-antigravity-cli.zip` differs from the `sha256` its release wrote into the installer | `"error: tightship-antigravity-cli.zip has sha256 <actual>; <tag> published <expected>"` |

## Side Effects

- **Persistence:**
  - `cargo xtask build` writes `dist/claude-code/` and `dist/antigravity-cli/` via temporary staging
    folders `dist/.claude-code.tmp` and `dist/.antigravity-cli.tmp`.
  - `cargo xtask package <tag>` writes `dist/tightship-claude-code.zip`,
    `dist/tightship-antigravity-cli.zip`, `dist/marketplace.json`, and `dist/install.sh` and
    `dist/install.ps1` with the tag and the Antigravity CLI zip's `sha256`, via `dist/.package.tmp`.
  - `install.sh` and `install.ps1` write `~/.gemini/config/plugin_data/tightship/options.toml` and invoke
    each selected harness's plugin installer.
- **Concurrency:** Each staging directory is isolated per harness; `settings.json` updates in
  `tightship` write to `settings.json.tightship.tmp` and rename atomically.
- **Transaction:** If any build check or identity scan fails for either harness, both staging directories
  are discarded and `dist/` holds no partial output.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — Build both harnesses from canonical content (NFR-FLEX-03.1, NFR-FLEX-07.1)

```gherkin
Given the canonical content in content/ and harnesses/claude-code/ and harnesses/antigravity-cli/
And .identity-terms at the repository root
When the maintainer runs `cargo xtask build`
Then the command exits 0
And dist/claude-code/ and dist/antigravity-cli/ are both written
And `git diff content/` is empty
And `agy plugin validate dist/antigravity-cli --strict` exits 0 with 0 errors and 0 warnings
And dist/antigravity-cli/skills/ holds all 13 baseline skills, all 10 commands as skills, and contract/SKILL.md
And dist/antigravity-cli/agents/adversarial-verifier.md lists only view_file, list_dir, find_by_name, grep_search, and run_command
```

### Scenario 2 — Pack both release archives and the catalog (happy path, NFR-FLEX-03.1)

```gherkin
Given `cargo xtask build --binaries <folder>` succeeded with the four platform binaries
And Cargo.toml has version "1.0.2"
When the release pipeline runs `cargo xtask package v1.0.2`
Then the command exits 0
And dist/tightship-claude-code.zip, dist/tightship-antigravity-cli.zip, and dist/marketplace.json exist
And dist/tightship-antigravity-cli.zip contains plugin.json, hooks.json, bin/tightship, bin/tightship.cmd, and all four platform binaries with mode 0755
```

### Scenario 3 — Packaging fails when Antigravity CLI lacks the four binaries (`PACKAGE_BINARY_MISSING`)

```gherkin
Given dist/antigravity-cli/ was built for the host platform only
When the maintainer runs `cargo xtask package v1.0.2`
Then the command exits 1 with "error: dist/antigravity-cli/bin/macos/tightship is missing; run cargo xtask build --binaries <folder> first"
And dist/tightship-antigravity-cli.zip does not exist
```

### Scenario 4 — Session start injects contract and memory as `injectSteps` JSON (FR-CTR-01.1, FR-MEM-01.1..01.3)

```gherkin
Given dist/antigravity-cli/ is installed and PLUGIN_DATA/options.toml has contract = true, memory = true, and vault_folder pointing at a vault with a pinned note and a handoff
When Antigravity CLI runs the SessionStart hooks of hooks.json
Then `hook contract` for part-1.md and memory.md each output valid JSON `{"injectSteps":[{"systemMessage":{"systemMessage":"..."}}]}` holding the whole contract from its first rule to its last
And `hook start` outputs valid JSON `{"injectSteps":[{"systemMessage":{"systemMessage":"..."}}]}` listing the project's pinned note and handoff body
```

### Scenario 5 — Turning `contract` or `memory` off suppresses their hooks (FR-CTR-02.1, FR-CTR-03.1, FR-MEM-10.1)

```gherkin
Given PLUGIN_DATA/options.toml has contract = true and memory = false
When Antigravity CLI runs the SessionStart and PreInvocation hooks
Then `hook contract` outputs part-1.md's JSON and outputs 0 bytes for memory.md
And `hook start` and `hook prompt` output 0 bytes and exit 0
And when PLUGIN_DATA/options.toml has contract = false, both `hook contract` invocations output 0 bytes and exit 0
```

### Scenario 6 — Malformed `options.toml` fails without blocking the session (`OPTIONS_TOML_INVALID`)

```gherkin
Given PLUGIN_DATA/options.toml contains `contract = "not-a-bool"`
When `tightship hook start` runs
Then it prints `error: <path>/options.toml is not valid TOML` to stderr and exits 1
And Antigravity CLI logs the hook warning and continues the session
```

### Scenario 7 — Prompt checkpoint fires on every 10th `PreInvocation` using `conversationId` (FR-MEM-02.1)

```gherkin
Given PLUGIN_DATA/options.toml has memory = true and vault_folder set
When `tightship hook prompt` runs 10 times with stdin `{"conversationId":"conv-42"}` and TIGHTSHIP_HOOK_OUTPUT naming hook-output.json
Then calls 1 through 9 output 0 bytes and exit 0
And call 10 outputs `{"injectSteps":[{"systemMessage":{"systemMessage":"MEMORY CHECKPOINT..."}}]}` and exits 0
```

### Scenario 8 — `tightship vault` reads `vault_folder` from `options.toml` (FR-MEM-03..11, NFR-COMP-01.1)

```gherkin
Given a note and a handoff written in the vault from Claude Code
And PLUGIN_DATA/options.toml in Antigravity CLI has vault_folder pointing at that same vault (with ~ or an absolute path) and TIGHTSHIP_VAULT is unset
When the agent in Antigravity CLI runs `tightship vault path`, `tightship vault search`, and `tightship vault handoff claim`
Then `tightship vault path` prints the vault's expanded path
And `tightship vault search` and `tightship hook start` find the note and handoff written from Claude Code with 0 conversion steps
```

### Scenario 9 — Status line enables, renders, and restores (FR-STL-01.1, FR-STL-02.1, FR-STL-03.1..03.2)

```gherkin
Given `~/.gemini/antigravity-cli/settings.json` holds a previous `statusLine`
And PLUGIN_DATA/options.toml has status_line = true
When `tightship hook status-line` runs
Then `~/.gemini/antigravity-cli/settings.json` holds tightship's `statusLine` command and `PLUGIN_DATA/status-line.json` holds the previous one
And when Antigravity CLI pipes its status line JSON to that command, it prints the two-row status line
And when PLUGIN_DATA/options.toml is changed to status_line = false and `tightship hook status-line` runs again, `~/.gemini/antigravity-cli/settings.json` holds the previous `statusLine` exactly
```

### Scenario 10 — Interactive and flag-driven install via `install.sh` and `install.ps1` (FR-INS-01.1..05.1, BR-04)

```gherkin
Given a machine with Antigravity CLI 1.4.0 or later
When the adopter runs `install.sh` (or `install.ps1` on Windows) and selects Antigravity CLI, contract = on, memory = on, status_line = on, and vault_folder = ~/ai-memory
Then ponytail and i-have-adhd are installed in ~/.gemini/config/plugins/
And humanizer is installed at ~/.gemini/config/skills/humanizer/SKILL.md
And tightship is installed in ~/.gemini/config/plugins/tightship/
And ~/.gemini/config/plugin_data/tightship/options.toml holds contract = true, memory = true, status_line = true, and vault_folder = "~/ai-memory"
And ~/.gemini/antigravity-cli/settings.json holds tightship's statusLine
```

### Scenario 11 — Installer rejects unknown harness or outdated CLI (FR-INS-01.1)

```gherkin
Given a machine where `agy --version` reports 1.3.0 (or `agy` is not on PATH)
When the adopter runs `sh install.sh --harness antigravity-cli --yes`
Then it exits 1 with "error: agy 1.4.0 or later is required; found 1.3.0"
And no files in ~/.gemini/config/plugins/tightship or ~/.gemini/antigravity-cli/settings.json are modified
```

### Scenario 12 — Bilingual Antigravity CLI install guide covers all subjects (FR-INS-06.1..08.1, BR-03)

```gherkin
Given docs/install/antigravity-cli/
When the adopter opens README.md and README.pt-BR.md
Then both link pages for installation (one-command installer and manual commands), the companions, the options, update, uninstall, migration, and without-the-contract in English and Brazilian Portuguese
And both state "Antigravity CLI 1.4.0 or later"
```

### Scenario 13 — Unauthorized release tag push is rejected (BR-01)

```gherkin
Given a user without write permission to ATNexusLab/tightship
When they run `git push origin v1.1.0`
Then GitHub refuses the push and `.github/workflows/release.yml` does not run
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md) and
[ADR 0001](../../decisions/0001-rust-build-and-tokenized-canonical-content.md), marked M3, #59:

- **ADR 0001 amended:** allows the cross-platform bootstrap installers (`install.sh` and `install.ps1`,
  assembled from `installer/` and each harness's fragments) and the Windows batch launcher
  (`tightship/launcher.cmd` with `harnesses/antigravity-cli/launcher-env.cmd`, built into
  `bin/tightship.cmd`), because the installer runs before any binary is downloaded and Antigravity CLI on
  Windows invokes hooks via `cmd /c`.
- **Building blocks:**
  - New block: [Antigravity CLI plugin](../../architecture/building-blocks/antigravity-cli-plugin.md)
    (`dist/antigravity-cli/`): `plugin.json`, `hooks.json`, `agents/adversarial-verifier.md`,
    `skills/<name>/SKILL.md` (skills, commands, and `contract`), `contract/`, and `bin/`.
  - [Harness definitions](../../architecture/building-blocks/harness-definitions.md): adds
    `harnesses/antigravity-cli/harness.toml`.
  - [Build](../../architecture/building-blocks/build.md): builds every harness in `harnesses/` and packs
    both `dist/tightship-claude-code.zip` and `dist/tightship-antigravity-cli.zip`.
  - [`tightship` binary](../../architecture/building-blocks/tightship-binary.md): gains
    `tightship hook contract`, reads `${TIGHTSHIP_DATA}/options.toml` when option environment variables
    are unset, fills the `"{text}"` slot of the JSON file `TIGHTSHIP_HOOK_OUTPUT` names, and
    reads `conversationId` in `hook prompt`.
  - [Release pipeline](../../architecture/building-blocks/release-pipeline.md): attaches
    `tightship-antigravity-cli.zip`, `install.sh`, and `install.ps1` to each release.
- **Crosscutting concepts:** [options.md](../../architecture/concepts/options.md),
  [contract.md](../../architecture/concepts/contract.md), [memory.md](../../architecture/concepts/memory.md),
  and [status-line.md](../../architecture/concepts/status-line.md) document the Antigravity CLI paths and
  mechanisms.

## Data model impact

Applied to [docs/data-model/](../../data-model/README.md), marked M3, #59:

- **[Option](../../data-model/options.md):** in Antigravity CLI, option values persist in
  `~/.gemini/config/plugin_data/tightship/options.toml` (`contract`, `memory`, `status_line`,
  `vault_folder`), read by `tightship` via `TIGHTSHIP_DATA`.
- **[Status line record](../../data-model/status-line.md):** in Antigravity CLI, `status-line.json` lives
  at `~/.gemini/config/plugin_data/tightship/status-line.json` and backs up the `statusLine` key of
  `~/.gemini/antigravity-cli/settings.json`.

## Out of Scope

- **Cursor (`M2`, #58) and Codex (`M4`) plugins:** handled in their own epics. Adding Cursor and Codex to
  `install.sh` and `install.ps1` is a new harness folder with its `[install]` table and its two fragments.
- **Publishing to a hosted Antigravity plugin marketplace:** Antigravity CLI installs from a local
  directory, zip, or git URL; no public registry is required.
- **Automatic reload after `/compact` without calling the `contract` skill:** Antigravity CLI 1.4.0 has
  no post-compaction hook event (`SessionStart` fires only once at conversation start); the injected
  contract header instructs the agent to load the `contract` skill after compaction.
- **Cutting the M3 release tag:** by `BR-01`, the M3 release tag is never cut before M2 (Cursor) is
  released, and both may share one tag.

## Task Breakdown

| # | Issue | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- | --- |
| 1 | #62 | Render `dist/antigravity-cli/` and pack `tightship-antigravity-cli.zip` in `xtask` | `harnesses/antigravity-cli/harness.toml` (new), `harnesses/antigravity-cli/launcher.cmd` (new), `xtask/src/build.rs`, `xtask/src/build/antigravity_cli.rs` (new), `xtask/src/package.rs`, `xtask/tests/build.rs`, `xtask/tests/release.rs`, `.github/workflows/release.yml`, 8 files | Scenarios 1, 2, 3, and 13 | — |
| 2 | #63 | Support `options.toml`, `hook contract`, `injectSteps` JSON output, and `conversationId` in `tightship` | `tightship/src/main.rs`, `tightship/src/failure.rs`, `tightship/src/options.rs` (new), `tightship/src/vault.rs`, `tightship/src/hook.rs`, `tightship/src/hook/contract.rs` (new), `tightship/src/hook/start.rs`, `tightship/src/hook/checkpoint.rs`, `tightship/src/hook/prompt.rs`, `tightship/src/status/settings.rs`, `tightship/tests/hook.rs`, `tightship/tests/status.rs`, `tightship/tests/vault.rs`, 13 files | Scenarios 4, 5, 6, 7, 8, and 9 | 1 |
| 3 | #64 | Add the unified interactive `install.sh` and `install.ps1` installers and the bilingual Antigravity CLI install guide | `install.sh` (new), `install.ps1` (new), `xtask/tests/install.rs` (new), `docs/install/antigravity-cli/` (14 new `.md` and `.pt-BR.md` files), `README.md`, `README.pt-BR.md`, `docs/README.md`, `zensical.toml`, `AGENTS.md`, 22 files | Scenarios 10, 11, and 12 | 2 |
