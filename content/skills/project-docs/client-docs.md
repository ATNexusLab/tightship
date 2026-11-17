# Client docs

The docs that present a delivered project to its client: what the system is, how its users start, how
they do each task, and what it offers. They follow [Diátaxis](https://diataxis.fr/), and every page is
one of its four types. The client and the client's users read them, so they are written in the client's
language, the one `{{command:bootstrap}}` agreed, whatever language the repository uses.

## Structure

```text
client-docs/
  README.md                the explanation: what the system is, for whom, what was delivered
  tutorial.md              getting started: one path from the first access to the first task done
  guides/
    README.md              the index of the tasks
    <task>.md              one how-to guide per task a user of the client does
  reference/
    README.md              the index of the reference
    features.md            every feature, what it does, and who can use it
    environments.md        each environment's address and what it is for
  assets/
    images/                the screens, copied from the project's design pages
    stylesheets/           docs-theme.css, image-viewer.css
    javascripts/           image-viewer.js
zensical.client.toml       the client site's config, at the repository root
```

A page the project has no content for is one line in its folder's `README.md` saying why, as in the
project docs.

## Rules

- **Nothing links outside `client-docs/`.** The client may have no access to the repository, and the
  client site serves only its own folder. A fact the project docs hold is written again here, in the
  client's words, because this reader cannot follow the link; whoever changes the fact changes both.
- **No internal vocabulary.** No requirement ID, epic, milestone, sprint, ADR, module name, or path of the
  code. A feature is named the way its users name it.
- **Screens are copies.** An image of a screen is copied from `docs/product/design/pages/<feature>/` to
  `assets/images/`, and the page embeds the copy. A project with no interface has no images.
- **Kept in step with the code.** A change to what a page describes updates the page in the same pull
  request, and `{{command:closeout}}` names it.

## Templates

`README.md`, the explanation:

```markdown
# <System name>

<What the system does, in one paragraph, for the people who use it.>

## Who uses it
<Each kind of user, and what each one does with the system.>

## What was delivered
<The features delivered, grouped as the users see them, each linking its reference entry.>

## Out of scope
<What the system does not do, each with its reason or the phase that will.>

## Where to start
[Getting started](tutorial.md) · [Guides](guides/README.md) · [Reference](reference/README.md)
```

`tutorial.md`, the tutorial:

```markdown
# Getting started

<What the reader will have done at the end, in one sentence.>

## Before you start
<The access, account, or device the reader needs, and where to get it.>

## 1. <First step, as an action>
<What to do, and what the reader sees when it worked. A screen image where it helps.>

## <N>. <Last step>

## Next
<The guides a new user reads next.>
```

`guides/<task>.md`, a how-to guide:

```markdown
# <Task, as an action: "Issue an invoice">

<When a user does this, in one sentence.>

1. <One action per step, with the screen and the control named as the system shows them.>
2. <…>

**Result:** <what the user sees when the task is done.>

**If something goes wrong:** <each likely problem, with what to do.>
```

`reference/features.md`, the reference:

```markdown
# Features

| Feature | What it does | Who can use it |
| --- | --- | --- |
| <name users know it by> | <one line> | <roles> |
```

`reference/environments.md`:

```markdown
# Environments

| Environment | Address | What it is for |
| --- | --- | --- |
| <Production> | <URL> | <who uses it, for what> |
```

## The client site

The client site is separate from the internal site: its own config, its own folder, its own build output.
It is never deployed by the repository; how the client receives it is the adopter's call.

`zensical.client.toml`, at the repository root:

```toml
[project]
site_name = "<Product>"
docs_dir = "client-docs"
site_dir = "client-site"
extra_css = ["assets/stylesheets/docs-theme.css", "assets/stylesheets/image-viewer.css"]
extra_javascript = [{ path = "assets/javascripts/image-viewer.js", type = "module" }]

[project.theme]
logo = "assets/images/<logo file>"
favicon = "assets/images/<favicon file>"
language = "<client's language code>"

[[project.theme.palette]]
scheme = "default"
primary = "custom"
accent = "custom"
toggle.icon = "lucide/sun"
toggle.name = "<Switch to dark mode, in the client's language>"

[[project.theme.palette]]
scheme = "slate"
primary = "custom"
accent = "custom"
toggle.icon = "lucide/moon"
toggle.name = "<Switch to light mode, in the client's language>"
```

- The theme, the logo, and the image viewer are the internal site's, copied into `client-docs/assets/`:
  `docs-theme.css` with the product's tokens, the logo and favicon from the brand, and the viewer's three
  strings in the client's language. A project with no interface keeps the Zensical defaults and drops
  the logo and favicon lines.
- `client-site/` is in `.gitignore`.
- **One command opens it:** `uvx zensical==<version> serve -f zensical.client.toml -a localhost:8001
  --open`, on its own port so both sites run at once, with the same pinned version as the internal site.
  The editor setup runs it as the `docs: serve client` task.
- **Before handing it over:** `uvx zensical==<version> build -f zensical.client.toml` exits 0, a crawl of
  the built site finds no broken link, anchor, or image, no relative link of any page resolves outside
  `client-docs/`, and both themes pass the check the internal site passes.
