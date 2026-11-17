# What a doc covers

A doc that describes code lists that code in its front matter, as paths or globs from the repository's
root (`*` within one folder, `**` across folders):

```yaml
---
covers: [src/billing/**, src/main.rs]
---
```

Every doc under `system/` except a folder's `README.md` has one; a guide or a client doc that describes
code may. `tightship docs affected` reads it to name the docs a diff touches, and `tightship docs check`
reports what drifted:

- a link to a file or an anchor that does not exist, in every doc;
- in a doc with `covers`, a path, symbol, or subcommand in a code span that its covered code no longer
  names, and a pattern that matches no file;
- a doc under `system/` with no `covers`;
- a `TODO`, `FIXME`, or `TBD` outside code, anywhere but `work/`, whose docs are history and get only the
  link check.

It reads `docs/` and `client-docs/`, and exits 1 on any finding. A finding is fixed in the doc, or in the
code when the doc is the one that is right.
