# Workflow documentation

The internal docs of the workflow plugin. Every folder opens on its own `README.md`, on GitHub, in
VS Code, or on the local site.

| Doc | What it holds |
| --- | --- |
| [Requirements](requirements/README.md) | The versioned SRS: every requirement with its ID, attributes, and acceptance criteria, and a use case diagram per area |
| [Architecture](architecture/README.md) | The map of the system in arc42 sections, with C4 diagrams |
| [Data model](data-model/README.md) | The vault and the options: entities, invariants, access patterns |

The roadmap (`roadmap/`), feature specs (`specs/`), and ADRs (`decisions/`) are added by `/sprint` and
`/spec` as the work reaches them. The product has no interface, so there is no design doc.

## Local site

```sh
uvx zensical==0.0.70 serve --open
```

It opens at `http://localhost:8000`, with a light and a dark theme. In VS Code, the `docs: serve` task
runs the same command. Clicking any image or diagram opens it full screen to zoom and pan.

Diagrams are `.drawio.svg` files in the `diagrams/` folder of the doc they illustrate. They open for
editing in VS Code with the draw.io extension the workspace recommends.
