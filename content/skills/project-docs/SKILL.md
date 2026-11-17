+++
name = "project-docs"
description = "Use when creating or updating a project's docs under docs/ — the folder structure by audience and lifecycle, the global docs (requirements, design, architecture, data model, roadmap), their diagrams (.drawio.svg), and the local docs site (Zensical, themed light and dark with the product's identity). Templates and what each doc owns."
+++

# Project Docs

Everything under `docs/` is written for people. Big files do not get read, so every doc is a **folder of
small files**, one subject each, with a `README.md` as the folder's index. GitHub shows that `README.md`
when the folder opens, VS Code shows the tree, and the local docs site turns the same folders into a
sidebar with search. Nothing is written twice for any of the three.

Every project is documented as if a team depended on it, even with one person on it: working alone is not
a reason to skip what needs documenting. The repo keeps the global docs below, and every feature keeps its
spec; a feature's PR updates the global docs it changes, so any part reads alone or against the whole.

## Standards

Each doc type follows one standard, chosen in this order: a global standard the industry already uses,
then ISO, then the convention of the doc's ecosystem, and our own convention only where none of those
exists. A doc type missing from the table is chosen by the same order.

| Doc type | Standard | Rung |
| --- | --- | --- |
| SRS | ISO/IEC/IEEE 29148 and the ISO/IEC 25010 quality model, with [SemVer](https://semver.org/) on the document | ISO |
| SRS changelog | [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) | ecosystem |
| Architecture | [arc42](https://arc42.org/overview) sections, drawn in the [C4 model](https://c4model.com/) | global |
| Data model | Chen notation for the conceptual model, crow's foot for the physical | global |
| Use cases, sequences, states | [UML](https://www.omg.org/spec/UML/) | global |
| Business processes | [BPMN](https://www.omg.org/spec/BPMN/) | global |
| Design tokens | the names and types of the [Design Tokens Format 2025.10](https://www.designtokens.org/TR/2025.10/) (W3C DTCG), in a Markdown table | global |
| Brand, screens | ours; a screen cites [ISO 9241-161](https://www.iso.org/standard/85790.html) for its components | own |
| Sprints | the [Scrum Guide 2020](https://scrumguides.org/scrum-guide.html): Sprint Goal, Product Goal, Definition of Done | global |
| Roadmap | ours: milestones, then epics | own |
| Feature spec | ours, `{{skill:spec-writing}}`; its scenarios in [Gherkin](https://cucumber.io/docs/gherkin/reference/) | own |
| ADR | [Nygard's format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions), plus `Alternatives considered` | global |
| User-facing pages, in `guides/` | [Diátaxis](https://diataxis.fr/): each page is a tutorial, a how-to guide, a reference, or an explanation | global |
| README, runbook | ours, `{{skill:technical-writing}}` | own |

## Structure

The folders group the docs by who reads them and how long each one lives: what the product must be,
how the system is built, why it was decided, what the work did, and what a user follows.

```text
docs/
  README.md                      map of the documentation; how to open the local site
  product/                       what the product must be — living
    README.md
    requirements/                the SRS — `{{skill:requirements-engineering}}`
      diagrams/use-cases-<area>.drawio.svg, process-<name>.drawio.svg
    design/
      README.md                  the design tool the repo adopted, and the index
      brand/                     description, brand book, logo, banner and every brand asset
      tokens.md                  the design tokens, and where the app's source of them lives
      pages/<feature>/<page>.md  one screen: its exported image and its specification
  system/                        how the system is built — living
    README.md
    architecture/                arc42 sections, C4 diagrams — see `system/architecture/`
      README.md                  goals, context and scope, solution strategy, and the index
      building-blocks/           the system decomposed, one file per building block
      runtime/                   scenarios, end to end across every layer
      deployment.md              where every building block runs
      concepts/                  crosscutting concepts, one file each
      risks.md                   risks and technical debt
      diagrams/<topic>.drawio.svg
    data-model/
      README.md                  conceptual and physical models, invariants, and the entity index
      access-patterns.md
      storage.md                 engine, indexes, policies, migrations — or files on disk
      diagrams/<topic>.drawio.svg
  decisions/NNNN-<slug>.md       ADRs — never edited once accepted
  work/                          what the work planned and did — history
    README.md
    roadmap/
      README.md                  milestones → epics, and requirement coverage
      sprints/sprint-NN.md       one sprint: plan, baseline, result
    specs/<epic>/<feature>.md    one feature in depth
  guides/                        pages a user follows, each one Diátaxis type
    README.md
    <subject>/                   one folder per subject, such as install/
  assets/                        the local site's own files
    stylesheets/                 docs-theme.css, image-viewer.css
    javascripts/                 image-viewer.js
```

- **One file, one subject**, readable in one sitting. A file that grows a second subject splits.
- **Every folder has a `README.md`**: what the folder holds and a linked list or table of its files.
  `assets/` and its folders are the exception: they hold the site's files, never a page.
- **Links are relative** (`../../system/architecture/README.md`), so they work on GitHub, in VS Code,
  and on the site.
- A level the project does not have is one line in the parent `README.md` saying why, not an empty file.
  A group with no folder left in it is dropped, and `docs/README.md` says why.
- **Diagrams are `.drawio.svg`**, each in the `diagrams/` folder of the doc it illustrates: edited in
  VS Code, rendered by GitHub.

## Previous layout

Docs written before this structure kept every folder at the root of `docs/`. The bootstrap moves them by
this table and rewrites every link that pointed into a moved folder; `docs/decisions/` stays.

| Previous | Current |
| --- | --- |
| `docs/requirements/` | `docs/product/requirements/` |
| `docs/design/` | `docs/product/design/` |
| `docs/architecture/` | `docs/system/architecture/` |
| `docs/data-model/` | `docs/system/data-model/` |
| `docs/roadmap/` | `docs/work/roadmap/` |
| `docs/specs/` | `docs/work/specs/` |
| `docs/stylesheets/`, `docs/javascripts/` | `docs/assets/stylesheets/`, `docs/assets/javascripts/` |
| any other folder of pages a user follows, such as `docs/install/` | `docs/guides/<folder>/` |

## What each doc owns — one source per fact

| Doc | Owns | Never holds |
| --- | --- | --- |
| `product/requirements/` | The versioned SRS: requirement IDs, attributes, acceptance criteria | Design |
| `product/design/` | The brand, the tokens, and every screen grouped by feature | A resource the code ships — it lives in the app's own folder |
| `system/architecture/` | The map: what exists, how it talks, which milestone builds it | The checkable import rules — `AGENTS.md` → Architecture |
| `system/data-model/` | Entities, relations, invariants, storage | Feature behavior |
| `decisions/` | Why an architectural choice was made | The current map |
| `work/roadmap/` | Milestones → epics, requirement coverage, each sprint's plan and dated result | Item status — issues, the GitHub Project, and GitHub Milestones own it |
| `work/specs/` | One feature in depth | Anything global |
| `guides/` | What a user follows to install, use, or operate the product | How the system is built |

A doc that restates another is replaced by a link. `{{skill:technical-writing}}` still decides whether a doc should
exist at all.

## `system/architecture/`

**Standard:** the section structure of **arc42**, drawn with the **C4 model**. Both are independent of
any architecture style.

**The content is inferred, never prescribed.** The building blocks, scenarios, and concepts come from the
architecture decided in the repo — its `AGENTS.md` Architecture section, its ADRs, and in detection its
code — and are named with that architecture's own vocabulary. This skill names no style, no layer, and no
kind of component. **Coverage is the whole system,** from infrastructure to interface: every part the
system has is documented somewhere below, and a section it lacks is one line in `README.md` saying why.

| arc42 section | Where |
| --- | --- |
| 1. Introduction and goals | `README.md` — the goal in one paragraph; quality goals link to `../../product/requirements/` NFRs |
| 2. Constraints | a link to `../../product/requirements/overview.md` |
| 3. Context and scope | `README.md` — C4 context diagram, and every external actor and system with what flows |
| 4. Solution strategy | `README.md` — the decided style and key technology choices, each linking its ADR |
| 5. Building block view | `building-blocks/` |
| 6. Runtime view | `runtime/` |
| 7. Deployment view | `deployment.md` |
| 8. Crosscutting concepts | `concepts/` |
| 9. Architecture decisions | a link to `../../decisions/` |
| 10. Quality requirements | a link to `../../product/requirements/` NFRs |
| 11. Risks and technical debt | `risks.md` |
| 12. Glossary | a link to `../../product/requirements/glossary.md` |

- **`building-blocks/`** — `README.md` holds the C4 container diagram and a table: block · technology ·
  responsibility · talks to (protocol) · milestone · link. One file per block: its job, what it never
  does, its inner blocks (C4 component diagram and table), and what someone needs to change it safely.
  The sections of a block's file come from what that block is.
- **`runtime/<scenario>.md`** — one scenario the architecture must get right, end to end across every
  block it crosses: numbered steps, each naming the block that acts and whom it talks to.
- **`deployment.md`** — every environment and where each block runs in it, with the C4 deployment
  diagram; for software installed on the user's machine, how it is packaged and installed. Delivery —
  CI/CD, gates, release — lives here too, since it is how blocks reach their environments.
- **`concepts/<concept>.md`** — one per concept that cuts across blocks, named by the architecture.
  Security is always one of them.
- **`risks.md`** — each known risk or piece of debt: what it is, its consequence, and the issue that
  tracks it.

Everything not built yet is still drawn, marked with its milestone and dashed, so the map shows the whole
system from day one. A section that only describes something another doc owns is a link, never a copy.

## `system/data-model/`

The method — access patterns first, identity, invariants — is `{{skill:database-design}}`. This folder is its
record.

- `README.md` — the conceptual model (Chen diagram), the physical model diagram, the invariants, and a
  table: entity · one-line meaning · milestone.
- **The physical model is always documented, as a diagram, never text.** It exists in every project with a
  database, however wide it gets, and no per-entity file or attribute table in Markdown restates it.
  `diagrams/physical.drawio.svg` is the model as the engine holds it: every table with its columns, types,
  nullability, primary key, foreign keys and unique constraints, and the cardinality of every relation.
- **It is drawn whole and per module.** `physical-<module>.drawio.svg` sits next to the complete diagram,
  never instead of it. A table another module owns appears there as a header-only box. Whoever changes a
  table changes the complete diagram and its module's.
- **The physical diagram is read from the database, never from the docs or from memory.** Apply the
  migrations to a throwaway database, query its catalog for columns, types, nullability and constraints,
  and generate the draw.io XML from that. `README.md` states how many migrations were read and when.
- What a diagram cannot show — an invariant, a domain rule, what deleting a parent means — is one line
  under `Invariants` in `README.md`.
- `access-patterns.md` — operation · reads/writes · filter and order · expected volume.
- `storage.md` — a database: engine, indexes, row-level policies, migration path; the tables and keys
  are the physical diagram's. Files: path · format · written by · lifecycle.

## `product/design/`

The brand and every screen are documented here, and each one shows on the docs site itself. A project
with no interface says so in one line in `docs/README.md`, and has no `product/design/`.

- **The tool is the team's.** `design/README.md` names the tool the repo adopted, as its `AGENTS.md`
  records it, and the index of what follows. Before producing or changing a design, load the design skill
  the session has.
- **`brand/`** — the brand description, the brand book, and the logo, banner and every other brand asset.
  A copy the app ships lives in the app's own folder; this one is documentation.
- **`tokens.md`** — every token by layer (color, type, spacing, radius, shadow) with its value in light
  and dark, where it is applied, and the path of the app's own source of them. Each token is named by its
  DTCG path (`color.accent`, `font.display`) and its columns are DTCG types (`color`, `fontFamily`,
  `fontWeight`, `dimension`, `shadow`); colors are written as hex. The docs site's `docs-theme.css`
  carries the same values, and whoever changes a token changes both.
- **`pages/<feature>/<page>.md`** — one screen: the image exported from the design tool, embedded in the
  page, at phone width before desktop, then its specification — areas and what each is responsible for,
  states, and the tokens it uses. Its components follow ISO 9241-161, and the page names any it departs
  from and why. A page that exists only as an HTML file to open is not documented: the
  HTML, when the tool produces one, sits next to the page as its source, and the image is what the site
  shows. A feature's spec links its pages.

## `work/roadmap/`

Status lives on GitHub, never here: each issue's Status and `Sprint` in the Project, and each milestone
as a **GitHub Milestone** holding its epics and tasks. This folder holds the plan and its history.

`README.md`:

```markdown
# Roadmap

Live status: the GitHub Project (<url>) and the milestones (<repo url>/milestones).

**Product Goal:** <the future state of the product the milestones move toward>

## M<N> — <name>
<Goal in one sentence.> GitHub Milestone: [M<N>](<milestone url>).

| Epic | Issue | Covers |
| --- | --- | --- |
| <epic name> | #<n> | FR-<AREA>-01, NFR-<CHAR>-02 |

## Requirement coverage
Every approved requirement of a planned milestone maps to exactly one epic.

| Requirement | Epic | Milestone |
| --- | --- | --- |

## Sprints
- [Sprint <N>](sprints/sprint-NN.md) — <goal>
```

`sprints/sprint-NN.md`:

```markdown
# Sprint <N> — <start> → <end>

**Sprint Goal:** <one sentence a user could observe at the end>
**Definition of Done:** <the repo's gates, as its `AGENTS.md` or `gates` skill lists them>
**Requirements baseline:** v<version>
**Items:** #<n>, #<n>

## Result — <close date>
Done: #… · Carried over: #… → Sprint <N+1>, because <reason> · Back to backlog: #…
```

## Diagrams

- One diagram per file, in the `diagrams/` folder of the doc that owns it, embedded with a relative
  path; a diagram never sits in a folder shared by every doc. `system/architecture/diagrams/`:
  `context`, `containers`, `deployment`, `components-<block>`, `runtime-<scenario>`.
  `system/data-model/diagrams/`: `conceptual`, `physical`, and their `-<module>` versions, and
  `states-<entity>`. `product/requirements/diagrams/`: `use-cases-<area>`, `process-<name>`. Each is
  `<topic>.drawio.svg`, a valid SVG with the diagram inside: GitHub, the site, and Zed's preview render
  it, and draw.io desktop or the `hediet.vscode-drawio` extension in VS Code edits it. Zed has no editor
  for it: its extension API has no webview.
- **Producing one:** write the diagram as draw.io XML (`<mxfile>`) in the scratchpad, then export with
  the draw.io desktop CLI: `<drawio> --export --embed-diagram --svg-theme light --output
  docs/<group>/<doc>/diagrams/<topic>.drawio.svg <file>.drawio`, where `<drawio>` is `drawio` on Linux,
  `/Applications/draw.io.app/Contents/MacOS/draw.io` on macOS, and `"C:\Program
  Files\draw.io\draw.io.exe"` on Windows. Without `--embed-diagram` the SVG can no longer be edited.
  Without `--svg-theme light` the SVG follows the reader's browser theme and turns black on a light page:
  the root carries `color-scheme: light;`. Labels are plain text (`html=0`) and the model has
  `background="#ffffff"`. No draw.io desktop on the machine → ask to install it: `winget install
  JGraph.Draw` on Windows, `brew install --cask drawio` on macOS, and on Linux the distribution's `drawio`
  package or the AppImage from `jgraph/drawio-desktop` releases. Never hand-write the SVG half.
- **Right after every export:** draw.io 30 still writes `light-dark()` in every color and a transparent
  background, and Zed's preview, which renders SVG with resvg, resolves neither. This step replaces each
  `light-dark(<light>, <dark>)` with `<light>` and gives the root a white background; the embedded diagram
  is left as it is, so draw.io still opens the file. On macOS and Linux:

  ```sh
  perl -0pi -e 's/light-dark\(((?:[^(),]|\([^()]*\))+),\s*(?:[^()]|\([^()]*\))*\)/$1/g; s/background: transparent; background-color: transparent;/background: #ffffff; background-color: #ffffff;/' <file>
  ```

  On Windows, in PowerShell 7 (`pwsh`): Windows PowerShell 5.1 reads and writes the file in the system
  code page and breaks every non-ASCII character in it.

  ```powershell
  $svg = Get-Content -Raw <file>
  $svg = $svg -replace 'light-dark\(((?:[^(),]|\([^()]*\))+),\s*(?:[^()]|\([^()]*\))*\)', '$1'
  $svg = $svg -replace 'background: transparent; background-color: transparent;', 'background: #ffffff; background-color: #ffffff;'
  Set-Content -NoNewline -Path <file> -Value $svg
  ```

  The check: `grep -c 'light-dark(' <file>` prints `0`, and `grep -c mxfile <file>` does not.
- **Architecture diagrams:** boxes labeled `<name>` + `[<technology>]` + a one-line responsibility; arrows
  labeled with what flows and the protocol. Built elements solid, later milestones dashed.
- **Conceptual model in Chen notation** (the BR-Modelo look): entity = rectangle; weak entity = double
  rectangle; relationship = diamond; attribute = ellipse, key attribute underlined, multivalued = double
  ellipse; cardinality `(min,max)` on each entity–relationship edge.
- **Physical model in crow's foot notation:** one box per table, the table name in the header and one
  row per column as `<key> <column> : <type>`, where the key is `PK`, `FK`, `UK` or blank and a nullable
  column ends in `?`. A composite unique constraint is its own last row. Each foreign key is an edge
  with crow's foot ends carrying the cardinality. Tables not built yet are dashed.
- **Use cases in UML:** the system boundary as a rectangle, each actor outside it, one ellipse per use
  case labeled `<FR-…> <verb phrase>`, and `«include»`/`«extend»` as dashed arrows. One diagram per area
  of the SRS, linked from the area's requirements file; a spec links the diagram, never redraws it.
- **Runtime scenarios in UML sequence:** one lifeline per building block the scenario crosses, named as
  the container diagram names it, and each message labeled with what flows and the protocol.

Two more exist only when the case calls for them, never as a set to fill:

- **State machine in UML,** `states-<entity>`, for an entity whose lifecycle has rules — an order, a
  subscription, an invoice: one rounded box per state, each transition labeled `<event> [<guard>]`, the
  initial and final states as the UML dots. The `Invariants` of `system/data-model/README.md` links it.
- **Business process in BPMN,** `process-<name>`, for a process with people, hand-offs and decisions —
  an approval, a return: one lane per role, tasks as rounded boxes, gateways as diamonds, start and end
  events as circles. The requirements file of its area links it.

## Local docs site

Internal docs are read locally, never deployed: whoever can clone the repo runs the site. A public
documentation site, if the project has one, is a separate config and folder and never mixes with this
one.

- **Generator: Zensical** (MIT, the successor to Material for MkDocs). It builds the sidebar from the
  folders and uses each folder's `README.md` as the section page.
- `zensical.toml` at the repo root, with the light and dark schemes the reader toggles, both in the
  product's colors, and the brand's logo and favicon:

  ```toml
  [project]
  site_name = "<Project> — internal docs"
  docs_dir = "docs"
  extra_css = ["assets/stylesheets/docs-theme.css", "assets/stylesheets/image-viewer.css"]
  extra_javascript = [{ path = "assets/javascripts/image-viewer.js", type = "module" }]

  [project.theme]
  logo = "product/design/brand/<logo file>"
  favicon = "product/design/brand/<favicon file>"

  [[project.theme.palette]]
  scheme = "default"
  primary = "custom"
  accent = "custom"
  toggle.icon = "lucide/sun"
  toggle.name = "<Switch to dark mode>"

  [[project.theme.palette]]
  scheme = "slate"
  primary = "custom"
  accent = "custom"
  toggle.icon = "lucide/moon"
  toggle.name = "<Switch to light mode>"
  ```

- **The site uses the product's colors and logo.** Copy `docs-theme.css` from this skill to
  `docs/assets/stylesheets/` and fill each scheme with the values of `product/design/tokens.md`. The logo
  is the brand version that reads on both themes. A project with no interface keeps the Zensical defaults
  and drops the logo and favicon lines.
- **One command opens the site:** `uvx zensical==<version> serve --open`, on `http://localhost:8000`, with
  the latest stable version from PyPI pinned. `docs/README.md` says it, and the editor setup the bootstrap
  writes, `.vscode/tasks.json` or `.zed/tasks.json`, runs the same command as its `docs: serve` task.
- **Every image opens in a viewer.** Clicking an image or a diagram on any page opens it full screen;
  the scroll wheel zooms at the cursor, dragging pans, a double click fits it back and Esc closes. A
  diagram readable only at column width is not delivered. Copy `image-viewer.js` from this skill to
  `docs/assets/javascripts/` and `image-viewer.css` to `docs/assets/stylesheets/`, and put its three
  strings in the language of the docs.
- **Both themes are verified.** On the served site, in light and in dark: the brand colors, every page's
  text and links, the logo, each diagram on its white card, and the viewer — click, zoom, drag, fit,
  close. A palette with no `media` ignores the system theme, so a browser forced dark still shows light:
  switch with the site's own toggle and confirm `body[data-md-color-scheme]` is `slate`.
- **`docs/README.md` is the home page.** A `docs/index.html` takes `/` from it and leaves the reader on
  a page with no sidebar and no search: it never exists.
- **Every folder the home page links to has its own `README.md`.** A link to a folder without one is a
  404 on the site, even though GitHub lists the folder.
- **A hand-written HTML page under `docs/` references nothing outside `docs/`.** The site serves only
  `docs_dir`, and it does not follow symlinks: an image or a stylesheet that lives elsewhere is copied
  next to the page, or drawn inline.
- **Before handing docs over, crawl the built site:** every page, every link, anchor and image, in a
  browser. `zensical build` reporting no issues does not cover the hand-written HTML pages.
- The build output `site/` is in `.gitignore`. `zensical new` also generates `.github/workflows/docs.yml`,
  a deploy workflow: never keep it.
