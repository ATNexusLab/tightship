# AGENTS.md

The workflow, packaged as a native plugin for Claude Code, Cursor, Antigravity CLI, and Codex, built from
one canonical source in this repository. What it must do: [the SRS](docs/requirements/README.md).

## Stack

Not decided. The M1 ADR settles the format of the canonical content, the build, and the shipped
scripts ([OQ-02](docs/requirements/open-questions.md#oq-02)). Until then the repository holds only docs:

- Markdown, linted by `markdownlint-cli2`, configured in `.markdownlint-cli2.jsonc`.
- Diagrams in draw.io, as `.drawio.svg`.
- The local docs site on Zensical, configured in `zensical.toml`, run through `uvx`.

## Programming languages & scripts

Not decided: the M1 ADR names the language of application code and of a new script
([OQ-02](docs/requirements/open-questions.md#oq-02)).

## Conventions

- Prose in `docs/` wraps at about 105 columns; the lint stops a line past 110, outside tables.
- `<a>` is the only HTML allowed in Markdown: the anchors of requirement IDs.

## Commands

| Gate | Command | Status |
| --- | --- | --- |
| Static analysis | `npx markdownlint-cli2@0.23.3` | Runs on every `.md` |
| Type checking | n/a | No code until the M1 ADR |
| Formatting | n/a | No formatter until the M1 ADR |
| Build | `uvx zensical==0.0.70 build` | Builds the docs site into `site/` |
| Tests | n/a | No runner until the M1 ADR |
| Run | `uvx zensical==0.0.70 serve --open` | Serves the docs at `http://localhost:8000` |

## Architecture

Not decided: the M1 ADR settles it with the stack, together with
[OQ-03](docs/requirements/open-questions.md#oq-03) and [OQ-09](docs/requirements/open-questions.md#oq-09).
The map so far: [docs/architecture/](docs/architecture/README.md).

## Branches

- `develop` is the integration branch; every PR targets it.
- `main` receives only releases from `develop`, as a merge commit.
- Neither branch has protection configured on GitHub: both rules are convention.

## Language

English throughout the repository: code, file names, docs, commits, branches, issues
([BR-03](docs/requirements/business-rules.md#br-03)).

## Gotchas

- Zensical parses Markdown with Python-Markdown: a list right under a paragraph line, with no blank line
  between, renders as plain text with literal hyphens, while GitHub shows a list. The lint rule MD032
  catches it.
