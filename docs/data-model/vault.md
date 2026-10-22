# Vault

**Milestone:** M1, epics #5 and #6 · **Spec:** [keep the vault](../specs/memory/keep-the-vault.md),
[hand off the session](../specs/handoff/hand-off-the-session.md)

The [Vault](README.md#conceptual-model), [Note](README.md#conceptual-model), and
[Scope](README.md#conceptual-model) entities as files. The vault is the adopter's git repository, in the
folder they set; the plugin never copies it elsewhere.

## Layout

```text
<vault folder>/
  AGENTS.md             the plugin's rules block, then the adopter's own text (M4, #118)
  notes/<scope>/        one note per file; <scope> is a project's name or all
```

A note is a `.md` file under `notes/` that opens with a `---` line and closes its front matter with another.
Any other file there, such as an `index.md` an older tool generated, is not a note: the session start, the
search, and the lint leave it out. The lint reports a file that opens with `---` and never closes its front
matter, and its file name still holds its `id`.

## A note

File name: `<id>-<slug>.md`, the slug being the title in lowercase ASCII words joined by `-`, at most 60
characters. Front matter, one `key: value` per line, a list as `[a, b]`:

| Field | Type | Required | Domain | Written by |
| --- | --- | --- | --- | --- |
| `id` | string | Yes | `mem-` and four or more digits, unique in the vault | `tightship vault new`, the highest number of the ids and file names plus one, under a lock |
| `type` | string | Yes | `preference`, `decision`, `gotcha`, `project`, or `handoff` | The agent; `handoff` only through `tightship vault handoff release` (#6) |
| `project` | string | Yes | The scope; equal to the note's folder | The agent |
| `why` | string | Yes | At least 5 words: what not knowing the note would cost | The agent |
| `tags` | list of strings | No | Free words, searched like the body | The agent |
| `pinned` | string | No | `true`, or absent | The agent, with `--pin` |
| `source` | string | Yes | `user`, `session`, or `repo` | The agent |
| `harness` | string | No | Free; read and kept, never written by the plugin | Older tools |
| `created` | date | Yes | `YYYY-MM-DD`, when the fact was established | `tightship vault new`, today by default |
| `updated` | date | Yes | `YYYY-MM-DD`, when the fact was last revised | `tightship vault new`, today |

The body opens with `# <title>`. A `[[<stem>]]` in it links another note by its file name without `.md`.

## A note's lifetime

Since M4 (#118). A note of `type` `gotcha` or `project`, whose `source` is not `user` and that is not pinned,
is removed by `tightship vault sync` once more than 30 days separate its `updated` from the machine's date. A
note with no valid `updated` never expires. In every note that stays, a `[[link]]` to it becomes plain text,
and that note's `updated` does not change.

## The rules block

Since M4 (#118). The vault's `AGENTS.md` holds the template the installed binary embeds between the lines
`<!-- tightship:rules:start -->` and `<!-- tightship:rules:end -->`. `vault init` writes the block alone, and
each sync rewrites it when it is missing or differs, putting it at the top of a file without markers. The text
outside the markers is the adopter's, and never changes.

## A handoff

A note of type `handoff`, written only by `tightship vault handoff release` (M1, #6). Its front matter is `id`,
`type: handoff`, `project`, `source: session`, `created`, and `updated`, and its `why` is always "without it the
next session, on another machine or harness, starts blind on work already in flight". Its body opens with
`# <title>`, then holds five parts, each label at the start of a line:

| Label | Holds |
| --- | --- |
| `**Goal:**` | What the work is for, with the issue or plan it follows |
| `**Done:**` | What is finished and verified |
| `**Next step:**` | The one action the next session starts with |
| `**Open decisions:**` | What waits for the adopter, or `none` |
| `**Repo state:**` | The branch, clean or the dirty files, and each unpushed commit as `<short hash> <subject>` |

A release removes the scope's other handoffs. Two machines that release before syncing leave two until the
next claim, which removes both and prints them. A claim prints only the handoffs it removed itself, so of
two sessions that claim at once, only one gets each handoff.

## Outside the vault

| What | Where | Lifetime |
| --- | --- | --- |
| The vault's folder for the agent's shell | One `export TIGHTSHIP_VAULT='<folder>'` line in the session's environment file | The session's |
