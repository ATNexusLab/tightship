# Support Zed and keep production artifacts lean

> **Status:** published
> **Profile:** API
> **Module:** `content/commands/`, `content/skills/`, `.zed/`, `docs/`
> **Epic:** [#158](https://github.com/ATNexusLab/tightship/issues/158)
> **Requirements:** NFR-FLEX-01, NFR-FLEX-02, NFR-FLEX-03, FR-SKL-01; new: FR-SKL-07, FR-SKL-08, NFR-FLEX-08
> **Sprint:** 3

An adopter works in Zed as well as in VS Code. The bootstrap writes the editor setup for the editors the
project uses, the diagrams the workflow exports render in Zed's preview, and every harness tightship
ships for runs in Zed's terminal and Zed's agent panel, each piece checked in a live session. A new skill
keeps every production artifact the workflow helps build down to what runs in production, and the audit
reports what breaks that rule.

Terms, as used here:

- **Harness**: a coding agent tightship ships a plugin for: Claude Code, Cursor, and Antigravity CLI
  ([NFR-FLEX-01](../../requirements/non-functional/flexibility.md#nfr-flex-01) to
  [NFR-FLEX-03](../../requirements/non-functional/flexibility.md#nfr-flex-03)). Codex has no plugin yet.
- **Agent panel**: Zed's panel that runs an external agent over the Agent Client Protocol (ACP), from the
  agent's entry in the ACP registry: `claude-acp`, `cursor` (`cursor-agent acp`), and `antigravity-acp`.
- **Piece**: what the plugin delivers to a session: the contract, the skills, the commands, the hooks,
  the status line, and the vault ([FR-SKL-01](../../requirements/functional/skills.md#fr-skl-01)).
- **Production artifact**: what a deploy runs: a container image, a serverless bundle, a release binary,
  a package.

## Decisions this spec rests on

Confirmed by the maintainer on 2026-11-11, in the grilling of
[#158](https://github.com/ATNexusLab/tightship/issues/158):

1. **One spec for the epic**, profile API: neither feature has a screen, and the contract is the text of
   the pieces in `content/`.
2. **SRS 4.1.0 adds three requirements**: FR-SKL-07 (the bootstrap sets up the editors the project
   uses), FR-SKL-08 (a skill keeps production artifacts lean), NFR-FLEX-08 (every harness runs in Zed).
3. **Zed covers every harness**, in Zed's terminal and in Zed's agent panel. This epic writes no support
   matrix: [#134](https://github.com/ATNexusLab/tightship/issues/134) owns it and folds these sections in.
4. **The diagram fix is a step of `project-docs`**, a text replacement after the draw.io export, with the
   command for macOS and Linux and the one for Windows. No Rust code.
5. **The skill is `production-artifact`**. Its description loads it; `delivery` and `/audit` name it. The
   contract does not change.
6. **The bootstrap settles the editors in its step 2**, as one more setup assumption.
7. **A piece that does not load in a harness's agent panel is recorded, not fixed**: the harness's install
   guide names it. A fix is a separate issue.

## Acceptance Criteria

### Contract

| Piece | Change | Read by |
| --- | --- | --- |
| `content/commands/bootstrap.md` | §2 settles the editors; §5 becomes `Editor setup`, with `.vscode/` and `.zed/`; front matter and Output name both | The agent running `/bootstrap` |
| `content/skills/project-docs/SKILL.md` | The export step makes the SVG light-only with a white background; VS Code is no longer the only editor named | The agent writing a diagram |
| `content/skills/production-artifact/SKILL.md` | New skill | The agent writing or changing what builds or ships an artifact |
| `content/skills/delivery/SKILL.md` | One line under `Building it` names the new skill | The agent delivering a plan |
| `content/commands/audit.md` | A fourth conditional lens, `Production artifact` | The agent running `/audit` |
| `docs/install/<harness>/install.md` and `.pt-BR.md` | A section `In Zed` (`No Zed` in pt-BR) | The adopter |

### Request

#### What the bootstrap reads to settle the editors

| Input | Type | Required | Source | Rule |
| --- | --- | --- | --- | --- |
| Existing `.vscode/` | folder | No | The repository | Present → VS Code is in the set |
| Existing `.zed/` | folder | No | The repository | Present → Zed is in the set |
| The adopter's correction | free text | No | The step 2 reply | Replaces the set; the only domain is VS Code, Zed, or both |

Neither folder present → the assumption proposes both. The assumption line, word for word, with
`\<editors\>` one of `VS Code`, `Zed`, `VS Code and Zed`, and `\<folders\>` the matching `.vscode/`, `.zed/`,
`.vscode/ and .zed/`:

> "Editors: \<editors\> — writes \<folders\>."

#### What the diagram export reads

| Input | Type | Required | Source | Rule |
| --- | --- | --- | --- | --- |
| `<topic>.drawio` | draw.io XML | Yes | The scratchpad | Written by the agent, as today |
| `<topic>.drawio.svg` | SVG | Yes | The draw.io CLI export | Rewritten in place by the step below |

### Response

#### `.zed/` written by the bootstrap

| File | Content | Rule |
| --- | --- | --- |
| `.zed/settings.json` | `lsp.<server>.initialization_options` or `.settings` with the command and arguments the gates pass; `languages.<Language>.formatter` and `format_on_save: "on"` for the formatter the gates check; `file_types` the stack needs | Only what serves the project, never a theme, font, or keymap; an extension that finds the gate's config alone gets no entry |
| `.zed/tasks.json` | A JSON array with the same tasks as `.vscode/tasks.json`, each as `{ "label", "command" }` | `docs: serve` included; a task that wraps a task-runner script does not go in |
| `docs/README.md` | A section `Editor`, a table of the Zed extensions the project needs, by extension ID | Zed reads `auto_install_extensions` only from the global settings, so the list lives in the docs |
| `.gitignore` | Does not ignore `.zed/` | — |

#### This repository's `.zed/`

`.zed/settings.json`:

```json
{
  "lsp": {
    "rust-analyzer": {
      "initialization_options": {
        "check": { "command": "clippy", "extraArgs": ["--", "-D", "warnings"] }
      }
    }
  },
  "languages": {
    "Rust": { "formatter": "language_server", "format_on_save": "on" },
    "Markdown": { "wrap_guides": [105, 110] }
  }
}
```

`.zed/tasks.json`:

```json
[{ "label": "docs: serve", "command": "uvx zensical==0.0.70 serve --open" }]
```

`docs/README.md`, section `Editor`: the Zed extensions `markdownlint`, `toml`, and `powershell`; Rust
is built in.

#### A diagram after the export step

- No `light-dark(` in the file: each `light-dark(<light>, <dark>)` is replaced by `<light>`, where
  `<light>` is `rgb(…)`, `#hex`, or a keyword, and `<dark>` may hold one nested `var(…)`.
- The root `style` reads `background: #ffffff; background-color: #ffffff;`.
- The `content` attribute, the embedded diagram, is unchanged, so draw.io still opens the file.

The step, word for word in the skill, on macOS and Linux:

```sh
perl -0pi -e 's/light-dark\(((?:[^(),]|\([^()]*\))+),\s*(?:[^()]|\([^()]*\))*\)/$1/g; s/background: transparent; background-color: transparent;/background: #ffffff; background-color: #ffffff;/' <file>
```

On Windows:

```powershell
$svg = Get-Content -Raw <file>
$svg = $svg -replace 'light-dark\(((?:[^(),]|\([^()]*\))+),\s*(?:[^()]|\([^()]*\))*\)', '$1'
$svg = $svg -replace 'background: transparent; background-color: transparent;', 'background: #ffffff; background-color: #ffffff;'
Set-Content -NoNewline -Path <file> -Value $svg
```

#### The `production-artifact` skill

Front matter `description`, word for word:

> "Use when writing or changing anything that builds or ships a production artifact — a container image,
> a serverless bundle, a release binary, a package — so it carries only what runs in production."

Its rules, naming no stack, packaging tool, or platform:

1. The artifact carries the build output, the runtime dependencies, and the files the program reads at
   run time, and nothing else.
2. Development dependencies, build tools, tests, and docs stay out. Building and running are separate
   steps, and the running step receives only the build's output.
3. Whatever selects the files that enter the build, an ignore file or an include list, starts from
   nothing and admits what the build needs.
4. The agent applies each rule with the commands of the stack it finds, and checks the result by listing
   what the built artifact holds.

#### The `In Zed` section of each install guide

Two surfaces, one paragraph each. Zed's terminal: the harness's CLI runs as in any terminal, and every
piece works. Zed's agent panel: the ACP registry entry the adopter installs, then the pieces that load
and the ones that do not, from the live check, each missing piece with its reason. Claude Code's guide
says the status line does not render in the agent panel: ACP has no status line surface.

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Adopter | N/A | The pieces run in the adopter's own session, with the harness's permissions; nothing here adds one |

---

## Business Rules

### 1. Who runs what

`/bootstrap` writes the editor setup, the agent writing a diagram runs the export step, and the agent
loads `production-artifact` when its task builds or ships an artifact. `/audit` loads it as a lens.

### 2. The editors

- **When:** step 2 of `/bootstrap`, in every mode.
- **What:** the editor set is one assumption, the line in
  [Request](#what-the-bootstrap-reads-to-settle-the-editors). §5 writes `.vscode/` for VS Code and `.zed/`
  for Zed, and only those.
- **Evolution:** a folder of an editor the adopter removed from the set is listed for removal and deleted
  only once the adopter approves the list, as every other change in evolution.

### 3. The Zed setup

`.zed/settings.json` and `.zed/tasks.json` follow [Response](#zed-written-by-the-bootstrap). The extension
reads the gate's config file and never gets a copy of its rules, as in `.vscode/`.

### 4. The diagrams

- **When:** after every `--export` of a `.drawio.svg`.
- **What:** the step in [Response](#a-diagram-after-the-export-step). The check: `grep -c 'light-dark('
  <file>` prints `0` and `grep -c mxfile <file>` does not.
- **Where they are edited:** in draw.io desktop or in VS Code with `hediet.vscode-drawio`; Zed previews
  them and has no editor for them, since its extension API has no webview.
- **This repository:** the 10 `.drawio.svg` files go through the step.

### 5. The harnesses in Zed

- Every harness is checked in a live session in Zed's terminal and in Zed's agent panel, piece by piece:
  the contract, a skill, a command, a hook, the vault, and the status line.
- What the check finds is what the install guide says; a missing piece is recorded with its reason.
- The terminal needs no setup beyond the harness's own install.

### 6. Lean production artifacts

The skill's four rules, in [Response](#the-production-artifact-skill). `delivery` says, under
`Building it`:

> "- **What builds or ships a production artifact** loads `{{skill:production-artifact}}` before it is
> written."

### 7. The audit lens

`/audit` adds, under **Conditional lenses**:

> "- **Production artifact** (a container file, a build or release workflow, a deploy config, a packaging
> manifest) → **sec** and **maint** also load `{{skill:production-artifact}}` and report a development
> dependency in the artifact, a missing or open ignore file, and a single step that builds and runs."

Each finding's title, word for word:

| Case | Title |
| --- | --- |
| A development dependency reaches the artifact | "Development dependency in the production artifact: \<dependency\> (\<file\>)" |
| No ignore file or include list selects the build's files | "No ignore file: the build receives the whole repository (\<file\>)" |
| The ignore file excludes paths instead of admitting them | "Open ignore file: \<file\> excludes paths instead of admitting them" |
| One step both builds and runs | "One step builds and runs the artifact (\<file\>)" |

### 8. Persistence and audit

- **Files changed:** the pieces in [Contract](#contract), `.zed/settings.json`, `.zed/tasks.json`,
  `docs/README.md`, the 10 diagrams.
- **Audit:** N/A, the pieces are text the agent reads; git records every change.
- **Events:** N/A.

---

## Errors

| Code | When | Message |
| --- | --- | --- |
| N/A | The bootstrap and the export step run inside the agent's session, which reports a failed command with its output, as the contract's faithful-reports line says | N/A |

The audit's findings are not errors: Rule 7 gives their titles.

## Side Effects

- **Persistence:** the files in Rule 8, in the adopter's repository or in this one.
- **Concurrency:** N/A, one agent session writes them.
- **Transaction:** the export and its rewrite run back to back on one file; a file the rewrite did not
  reach still carries `light-dark(`, which the check in Rule 4 catches.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — Bootstrap for Zed (happy path)

```gherkin
Given a repository with code and neither .vscode/ nor .zed/
When the adopter runs /bootstrap and answers the assumption "Editors: VS Code and Zed — writes .vscode/ and .zed/." with "Zed only"
Then .zed/settings.json and .zed/tasks.json are written and .vscode/ is not
And .zed/tasks.json holds the task "docs: serve"
And docs/README.md lists the Zed extensions the project needs
And the project opened in Zed formats on save and shows the linter's diagnostics
```

### Scenario 2 — Both editors (alternative)

```gherkin
Given a repository with .vscode/ and no .zed/
When the adopter runs /bootstrap and answers the assumption "Editors: VS Code — writes .vscode/." with "both"
Then .vscode/ is realigned and .zed/ is written
And both carry the same tasks
```

### Scenario 3 — An editor removed in evolution (exception)

```gherkin
Given a repository with .vscode/ and .zed/
When the adopter runs /bootstrap and answers the editors assumption with "VS Code only"
Then the change list proposes deleting .zed/
And .zed/ stays until the adopter approves the list
```

### Scenario 4 — A diagram renders in Zed (happy path)

```gherkin
Given a diagram exported with the draw.io CLI
When the agent runs the export step of project-docs
Then the file holds no "light-dark(" and its root background is #ffffff
And draw.io desktop opens it with the diagram intact
And Zed's preview shows it as GitHub does
```

### Scenario 5 — This repository's diagrams

```gherkin
Given the 10 .drawio.svg files of this repository
When each goes through the export step
Then each renders in Zed's preview and on the local docs site in both themes as on GitHub
```

### Scenario 6 — Every harness in Zed's terminal

```gherkin
Given Claude Code, Cursor, and Antigravity CLI, each with tightship installed
When each runs in Zed's terminal
Then every piece works as in any other terminal
```

### Scenario 7 — Every harness in Zed's agent panel

```gherkin
Given each harness's ACP registry entry installed in Zed
When a session starts in the agent panel
Then the contract, a skill, a command, a hook, and the vault are checked one by one
And the harness's install guide says which load and why each one that does not
```

### Scenario 8 — No status line in the agent panel (exception)

```gherkin
Given Claude Code in Zed's agent panel
When the session runs
Then no status line shows
And Claude Code's install guide says ACP has no status line surface
```

### Scenario 9 — The skill loads for a deploy (happy path)

```gherkin
Given the built plugin in a supported harness
When the agent writes the file that builds a container image
Then it loads production-artifact before writing it
And the image it describes holds the build output, the runtime dependencies, and the files read at run time only
```

### Scenario 10 — The audit reports a development dependency

```gherkin
Given a project whose production image installs the development dependencies
When the adopter runs /audit on it
Then a finding titled "Development dependency in the production artifact: <dependency> (<file>)" is reported
```

### Scenario 11 — The audit reports the ignore file

```gherkin
Given a project with a container build and no ignore file
When the adopter runs /audit on it
Then a finding titled "No ignore file: the build receives the whole repository (<file>)" is reported
And given an ignore file that only excludes paths, the finding is "Open ignore file: <file> excludes paths instead of admitting them"
```

### Scenario 12 — The audit reports a single step

```gherkin
Given a container file whose one stage builds and runs
When the adopter runs /audit on it
Then a finding titled "One step builds and runs the artifact (<file>)" is reported
```

### Scenario 13 — Nothing to audit (alternative)

```gherkin
Given a target that builds or ships no artifact
When the adopter runs /audit on it
Then production-artifact is not loaded and no finding of Rule 7 is reported
```

---

## Architecture impact

- **Canonical content** (`building-blocks/canonical-content.md`): `skills/production-artifact/` holds the
  rules for production artifacts; `/bootstrap` writes `.zed/` next to `.vscode/`.
- **Deployment** (`deployment.md`): each harness also runs in Zed's terminal and Zed's agent panel; the
  agent panel runs the harness's ACP adapter, which the adopter installs from Zed's ACP registry.
- **Context** (`diagrams/context.drawio.svg`): Zed joins the editors the adopter runs a harness in,
  marked with this epic.

## Data model impact

N/A — no entity, file, or attribute of `docs/data-model/` changes: `.zed/` belongs to the adopter's
repository, as `.vscode/` does.

## Out of Scope

- **The support matrix**: [#134](https://github.com/ATNexusLab/tightship/issues/134) writes it and takes
  the `In Zed` sections in.
- **Fixing a piece that does not load in an agent panel**: each gets its own issue once the check finds
  it.
- **Codex in Zed**: Codex has no plugin yet; its epic,
  [#142](https://github.com/ATNexusLab/tightship/issues/142), covers its surfaces.
- **Editing diagrams in Zed**: Zed's extension API has no webview, so no draw.io editor runs inside it.
- **Installing Zed extensions from the project**: Zed reads `auto_install_extensions` only from the
  global settings.
- **Editors other than VS Code and Zed.**
- **A `tightship` subcommand for the diagram step**: the step is two commands in the skill.

## Task Breakdown

| # | Issue | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- | --- |
| 1 | [#157](https://github.com/ATNexusLab/tightship/issues/157) | Ship only what production runs | `content/skills/production-artifact/` (new), `content/skills/delivery/SKILL.md`, `content/commands/audit.md`, `docs/architecture/building-blocks/canonical-content.md` | Scenarios 9 to 13 | — |
| 2 | [#159](https://github.com/ATNexusLab/tightship/issues/159) | Export diagrams Zed renders | `content/skills/project-docs/SKILL.md`, the 10 `.drawio.svg` of `docs/` | Scenarios 4 and 5 | — |
| 3 | [#156](https://github.com/ATNexusLab/tightship/issues/156) | Bootstrap and run every harness in Zed | `content/commands/bootstrap.md`, `.zed/settings.json`, `.zed/tasks.json`, `docs/README.md`, `docs/install/{claude-code,cursor,antigravity-cli}/install.md` and their `.pt-BR.md`, `docs/architecture/deployment.md`, `docs/architecture/diagrams/context.drawio.svg` | Scenarios 1 to 3, 6 to 8 | 2 |
