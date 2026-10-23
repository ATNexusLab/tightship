+++
name = "bootstrap"
description = "Set up this repo for the workflow — its AGENTS.md, GitHub Project, .vscode/, and global docs"
argument-hint = "[optional notes about the project]"
+++

Set up the repository for the workflow, layered on top of the global contract, for: {{capability:arguments}}

This command produces five things, each created on first run and realigned on every later run:

1. The repo's `AGENTS.md` files: one at the root, plus one per area (`apps/<app>/`, a monorepo
   workspace) that has its own stack or conventions — {{capability:area-instructions}}. The area file
   has the same sections as the root, scoped to the area, and never repeats or contradicts the root.
2. The repo's project skills, in {{capability:project-skills-folder}}: its gates and every procedure of
   one area or step.
3. The repo's GitHub Project, where the backlog lives.
4. `.vscode/`, the editor setup the project needs.
5. The docs skeleton from `{{skill:project-docs}}`: `docs/README.md`, `docs/architecture/`, `docs/data-model/`,
   their diagrams, and the local docs site.

It does not move code or open an ADR — an architectural decision that comes out of this belongs to
`{{command:spec}}`. The roadmap and sprints belong to `{{command:sprint}}`.

## 0. Which of the three jobs this is

Decide before reading anything else; the rest of the command changes with the answer.

- **An `AGENTS.md` or {{capability:harness-instructions-file}} already exists, at the root or in any
  area** → **evolution.** Find every one ({{capability:instruction-files-search}}, outside dependency
  folders) and every project skill, and read each. List each block of each file with where it goes:
  kept, moved to a project skill by name, or cut because the repository shows it. A project skill that
  changes is listed with its change. No file changes until I approve the list.
  {{capability:instruction-files-migration}}
- **The repo has code** → **detection.** The architecture is *read and recorded as it is*, not as it
  should be. A divergence you find is a note in the file, never a rewrite of the code.
- **The repo is empty** → **decision, made with me.** There is nothing to detect and nothing to
  default: the stack, the tooling, the repo shape, and the architecture are my choices. You never pick
  them yourself, never carry them over from another repository on this machine, and never start from a
  reference pattern. The `gates` skill only exists once something is there to run — what does not run yet is
  recorded as a declared pending, never as an invented command.

## 1. Detect (the repo-with-code path)

Read what this repo actually has: its dependency manifest and lockfile, its toolchain, formatter, and
type-checker config, its container and orchestration files, its CI workflows, its migration directory,
and the top-level source layout. Every ecosystem names these differently — find the names, never
assume them. Infer stack, package manager, and toolchain from what you found.

Two things usually asked that the environment answers: **branches** (`git branch -r`, plus protection
via `gh api`) and the **project's languages** — docs, code, commits (the README, the code, the commit
subjects).

For the architecture itself, read it with `{{skill:architecture-reading}}` — its passes, in its order. If the
codebase is large or unfamiliar, dispatch {{capability:explore-agent}} with that same lens.

## 2. Agree before writing

**Evolution and detection** — run a `{{skill:grilling}}` session: the code answers the facts, so decide what it
settles and state the assumptions this `AGENTS.md` rests on — the ones that change the file if they are
wrong. Only what I contest becomes a question. Typically shape-changing here: the intended architecture
when the code does not reveal it, a protected branch that is convention rather than configuration, and
the languages when the repo is mixed.

**Decision** — no assumptions list. List the decisions the file needs (stack, repo shape, architecture,
branches, language), then take them **one at a time**: the options that fit the requirements, each
with its trade-off, and I choose. A decision I have not made stays open and goes to `Open` in the
output — it is never filled in to complete the file.

**In every mode**, the same step also settles the setup in §4–§6: the branch model (an integration
branch plus `main`, or `main` alone), the sprint length, the label set, the `.vscode/` contents, and
the design tool the team documents screens with, when the project has an interface.
These are decisions or assumptions like any other — nothing in §4–§6 runs before they close.

**Nothing is written until this step closes**, in every mode.

## 3. Write it

Additive to the global contract — **never repeat** what is already global. `AGENTS.md` holds what every
task in the repo needs, in these sections and in this order, and is at most 4,000 characters:

- **Stack** — languages, frameworks, package manager, as confirmed. Never a version the manifest states.
- **Programming languages & scripts** — required, never inferred from another repo: the programming
  language application code is written in, and the one a new script is written in here. The global
  contract mandates neither, so a repo that does not state them has no rule at all. Detection reads what
  the repo already uses; decision asks me.
- **Architecture** — the current shape: the modules, where each kind of code lives today, the data flow,
  and the ADRs behind it. It records what the code does or what I chose in step 2; no architectural
  pattern is a default. It describes, never commands: no "conform strictly", nothing that makes the shape
  a law.
- **Conventions** — what every task here needs and the repo does not show: how the entry point is wired,
  how the dev stack is composed. Only what this repo does or what I decided, never one carried over from
  another project, and never turned into a lint rule: lint keeps the framework's rules and generic
  quality rules. A procedure of one area or step is a project skill, not a convention.
- **Branches** — the integration branch every PR targets, and what reaches `main`.
- **Language** — the repo's language and every part that differs from it: code and file names, comments,
  docs, commits, branches, UI copy. The chat is not the repo's: it follows the harness setting.
- **Gotchas** — only what was **observed** in this repo and holds in every task. A predicted problem is
  not a gotcha; a gotcha of one area goes to that area's project skill.
- **Project skills** — one line per project skill: its name and when it applies, never its text.

## 3b. Project skills

Each one is `<name>/SKILL.md` in {{capability:project-skills-folder}}, `<name>` in lowercase words joined
by `-`, and opens with `name` and `description` front matter, the description stating when to use it.

- **`gates`**, for any repo with something to run: every gate as its exact invocation, discovered from
  the repo's own config and task runner, cheapest first — static analysis, type checking, formatting,
  build, tests, and how to run it. A gate the language lacks is marked `n/a` with its reason, because a
  gate missing silently reads as a gate that passes.
- **One skill per procedure of one area or step** found in detection or decided in step 2: a release,
  running the app, a subsystem's gotchas, the design tool the team documents screens with and where its
  sources live.
- {{capability:project-skill-pointer}}

## 4. GitHub Project

Look before creating: `gh project list --owner <owner>` and the repo's linked projects. Reuse a project
that already serves this repo; never create a second one.

- **Project** — titled after the repo, linked to it (`gh project create`, `gh project link`).
- **Status** — exactly Backlog (GRAY) · Todo (GREEN) · In Progress (YELLOW) · In Review (BLUE) ·
  Staging (ORANGE) · Done (PURPLE). Staging holds what is merged into the integration branch and waits
  for the release. Read the current options first, with GraphQL (`options { id name color description }`).
  All six there → leave the field alone. Otherwise set them with the `updateProjectV2Field` mutation,
  passing every option that already exists with its `id`: `singleSelectOptions` replaces the whole list,
  and an option sent without its `id` is created anew, so every item that held the old one loses its
  Status.
- **Sprint** — an Iteration field, created with the `createProjectV2Field` GraphQL mutation,
  `dataType: ITERATION`, and the sprint length agreed in step 2.
- **Labels** — `epic`, one per change type the repo's commits use (`feat`, `fix`, `chore`, `docs`, …),
  `security-debt`, `tech-debt`, `requirement-change`, and one `area:<module>` per module of the
  Architecture section. Reuse what exists; create only what step 2 agreed.
- **Branches** — create the integration branch from `main` if it is missing, and push it.

Workflows and views are mine to configure — the API cannot edit them — so the output names the workflow
settings the flow needs: "Item closed" → Done, and "Auto-archive items" with the filter `is:closed`.
"Pull request linked to issue" → In Review is named only when `main` is the sole branch: GitHub reads a
closing keyword reliably only in a PR into the default branch, so a PR into the integration branch
usually links no issue and the workflow does not fire. There, an issue's In Review and Staging are set with
`gh project item-edit`.

## 5. `.vscode/`

Only what serves this project, in its stack, for anyone who clones it — never a personal preference
(theme, font, keymap).

- `extensions.json` — `recommendations` for the stack's language server, formatter, and linter, plus
  `hediet.vscode-drawio` for the diagrams.
- `settings.json` — only what the repo's own gates need in the editor: format on save with the repo's
  formatter, the linter the gates run, file associations the stack needs.
- `tasks.json` — only commands with no script behind them in the repo's task runner, like `docs: serve`
  for the local docs site, which runs the command `{{skill:project-docs}}` gives. A task that only wraps an
  existing script does not go in.
- The repo's `.gitignore` must not ignore `.vscode/`.

## 6. Docs

The structure and templates are `{{skill:project-docs}}`. Written from the requirements, the ADRs, and — in
detection — the code:

- `docs/README.md` — the map of the documentation and how to open the local site.
- `docs/architecture/` — arc42 with C4 diagrams, covering the whole system from infrastructure to
  interface; its content inferred from the Architecture section and the ADRs, each element marked with
  the milestone that builds it.
- `docs/requirements/diagrams/` — the use cases, one diagram per area of the SRS.
- `docs/data-model/` — conceptual, logical, and physical. Its method is `{{skill:database-design}}`.
- `docs/design/` — the brand, the tokens, and every screen by feature, in the tool §2 settled. A project
  with no interface says so in `docs/README.md` instead.
- Diagrams as `.drawio.svg` in the `diagrams/` folder of the doc each one illustrates.
- The local docs site: `zensical.toml` with both themes in the product's identity, `docs-theme.css`, the
  image viewer, `site/` in `.gitignore`, and no deploy workflow.

An existing single-file doc (`docs/requirements.md`, `docs/architecture.md`, …) is split into its folder,
its content kept, and every link to it updated.

In evolution, realign them with what changed; a doc that contradicts the code or an ADR is fixed here.

## 7. What never goes in

- **An aspirational rule.** A rule the repo does not follow today is a lie the next session enforces.
  Record the reality, and put the intent in the requirements or an ADR instead.
- Anything already in the global contract, or already in a skill — reference it, do not restate it.
- In `AGENTS.md`, a gate command or a procedure of one area or step: it is a project skill.
- A command that was not run.
- A catalogue of what was considered and rejected. The file says what to use and which behavior is
  forbidden; a discarded library belongs in its ADR's `Alternatives considered`.
- Another tool's instruction file. `.github/copilot-instructions.md` and its mirrors are not written,
  and in evolution they are removed: agent instructions live only in `AGENTS.md`. A framework that
  writes its own block into `AGENTS.md` or {{capability:harness-instructions-file}} is switched off in its config.

## 8. Validate

Run **every** gate of the `gates` skill and report its exit code. One that fails is removed or kept
with the failure stated in the skill — never left looking green; one marked `n/a` says why. Then check:
each `AGENTS.md` is 4,000 characters or fewer and states no gate command, nothing duplicates the global
contract, no placeholder or TODO survives, every project skill opens with its front matter and is listed
under Project skills, and the frontmatter and structure are clean.

Then check the setup: `gh project field-list` shows Status with its six options and the `Sprint`
iteration, `gh label list` holds the agreed labels, the integration branch exists on the remote, and
each `.drawio.svg` carries its diagram (`grep -c mxfile <file>` is not 0), `zensical build` exits 0, and
the served site passes the check in both themes that `{{skill:project-docs}}` sets.

## Output

```text
Bootstrap:
- Written:      [paths]
- Mode:         [new / evolution — what changed]
- Stack:        [languages, package manager, toolchain]
- Architecture: [one line]
- Skills:       [project skills written]
- Gates:        [each, with exit code]
- Project:      [url · Status ok · Sprint <length> · labels created/reused · workflows for you to set]
- Editor:       [.vscode files written]
- Docs:         [docs/README.md · requirements/diagrams/ · architecture/ · data-model/ · design/ · zensical.toml]
- Open:         [what is declared pending / none]
```
