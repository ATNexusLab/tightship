# tightship documentation

The docs of the tightship plugin. Every folder opens on its own `README.md`, on GitHub, in
VS Code, or on the local site.

The folders group the docs by who reads them and how long each one lives
([ADR 0005](decisions/0005-docs-are-laid-out-by-audience-and-lifecycle.md)).

| Group | What it holds |
| --- | --- |
| [Product](product/README.md) | What the plugin must be: the [requirements](product/requirements/README.md) and the [design](product/design/README.md) |
| [System](system/README.md) | How it is built: the [architecture](system/architecture/README.md) and the [data model](system/data-model/README.md) |
| [Decisions](decisions/README.md) | The architecture decision records |
| [Work](work/README.md) | What the work planned and did: the [roadmap](work/roadmap/README.md) and its sprints, and the [specs](work/specs/README.md) |
| [Guides](guides/README.md) | Installing the plugin in each harness, and releasing it |

`assets/` holds the local site's stylesheets and scripts.

## Local site

```sh
uvx zensical==0.0.70 serve --open
```

It opens at `http://localhost:8000`, with a light and a dark theme. In VS Code and in Zed, the
`docs: serve` task runs the same command. Clicking any image or diagram opens it full screen to zoom and
pan.

Diagrams are `.drawio.svg` files in the `diagrams/` folder of the doc they illustrate. They open for
editing in draw.io desktop, or in VS Code with the draw.io extension the workspace recommends. Zed
previews them and cannot edit them.

## Editor

The repository carries its setup for VS Code in `.vscode/` and for Zed in `.zed/`: Rust formats on save,
clippy's diagnostics show as you type, and the `docs: serve` task is there. VS Code offers its extensions
when the folder opens. Zed installs extensions only from your own settings, so add these by ID:

| Extension | What it serves |
| --- | --- |
| `markdownlint` | The Markdown lint, with the rules of `.markdownlint-cli2.jsonc` |
| `toml` | `Cargo.toml`, `harness.toml`, and the other TOML files |
| `powershell` | `install.ps1` and the PowerShell launchers |

Rust support is built into Zed.
