# tightship documentation

The docs of the tightship plugin. Every folder opens on its own `README.md`, on GitHub, in
VS Code, or on the local site.

| Doc | What it holds |
| --- | --- |
| [Install guide, Claude Code](install/claude-code/README.md) | Installing, configuring, updating, and removing the plugin, and moving to it from the pre-plugin install |
| [Requirements](requirements/README.md) | The versioned SRS: every requirement with its ID, attributes, and acceptance criteria, and a use case diagram per area |
| [Architecture](architecture/README.md) | The map of the system in arc42 sections, with C4 diagrams |
| [Data model](data-model/README.md) | The vault and the options: entities, invariants, access patterns |
| [Roadmap](roadmap/README.md) | The milestones, their epics, requirement coverage, and each sprint's plan |
| [Release](release/README.md) | How a version reaches adopters, and the checklist it passes on macOS, Linux, and Windows |
| [Specs](specs/README.md) | One closed spec per feature, grouped by epic |
| [Decisions](decisions/README.md) | The architecture decision records |
| [Design](design/README.md) | The brand, its voice and assets, and the tokens |

## Local site

```sh
uvx zensical==0.0.70 serve --open
```

It opens at `http://localhost:8000`, with a light and a dark theme. In VS Code, the `docs: serve` task
runs the same command. Clicking any image or diagram opens it full screen to zoom and pan.

Diagrams are `.drawio.svg` files in the `diagrams/` folder of the doc they illustrate. They open for
editing in VS Code with the draw.io extension the workspace recommends.
