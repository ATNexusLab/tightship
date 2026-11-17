# 0005. Docs are laid out by audience and lifecycle

- Status: accepted
- Date: 2026-11-16

## Context

`project-docs` added each doc type as a folder at the root of `docs/`, one at a time. The root ended up
holding four kinds of folder side by side. Some described the product and the system as they are now
(requirements, design, architecture, data model). Others kept the history of the work (roadmap and
sprints, the specs of each epic). Then came the pages a user follows (install, release) and the local
site's own stylesheets and scripts. A reader had to know that history to tell a doc that must match the
code from a record that never changes again.

Issue [#138](https://github.com/ATNexusLab/tightship/issues/138) also asks for docs that present a
delivered project to its client. They need a place that is not mixed with the docs for whoever builds
the project, since the internal site serves the whole of `docs/`.

## Decision

**`docs/` groups its folders by who reads them and how long each one lives:**

```text
docs/
  README.md
  product/     what the product must be, kept current: requirements/, design/
  system/      how the system is built, kept current: architecture/, data-model/
  decisions/   why, never edited once accepted
  work/        what the work planned and did, history: roadmap/, specs/<epic>/
  guides/      pages a user follows, one Diátaxis type each
  assets/      the local site's stylesheets/ and javascripts/
client-docs/   the docs for the client, with their own site in zensical.client.toml
```

- A doc under `product/` or `system/` is wrong when it disagrees with the code. A doc under `work/` or
  `decisions/` is a record of its date and is not rewritten to match the code.
- `decisions/` stays at the root of `docs/`: an ADR belongs to neither the product nor the system alone,
  and every doc links it.
- `client-docs/` sits outside `docs/`, so the internal site never serves it and it never links into
  `docs/`. It follows Diátaxis, the standard ADR 0004 picked for pages a user reads, and adds a row to
  that ADR's table: client docs, Diátaxis, global.
- `project-docs` holds the tree and a table from the previous layout to this one. The bootstrap moves a
  repository on the previous layout by that table once the adopter approves the list of moves.

## Consequences

- Every command and skill that names a docs path names the new one, and only the new one.
- An adopter's repository moves once, through the bootstrap, and every relative link in it is rewritten.
- This repository moves its own docs in the same change. Links from outside the repository to the old
  paths on `develop` break; the tags already published keep the old layout.
- A group a project does not have, such as `system/` in a project that delivers only client docs, is
  dropped and named in `docs/README.md`.

## Alternatives considered

- **Diátaxis at the root of `docs/`**, as tutorials, how-to guides, reference, and explanation.
  Rejected: an SRS, an arc42 map, and a sprint record are none of the four, and would all land under
  reference or explanation.
- **Keep every folder at the root and only add `client/`.** Rejected: it leaves the mix that made the
  structure hard to read, and the client folder would be served by the internal site.
- **`docs/client/` with a second site that reads only that folder.** Rejected: the internal site serves
  all of `docs_dir`, so the client pages would show up in it, and their links would be tempted into the
  internal docs.
