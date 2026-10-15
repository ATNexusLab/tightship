# Vault

**Milestone:** M1, epic #5 · **Spec:** [keep the vault](../specs/memory/keep-the-vault.md)

The [Vault](README.md#conceptual-model), [Note](README.md#conceptual-model), and
[Scope](README.md#conceptual-model) entities as files. The vault is the adopter's git repository, in the
folder they set; the plugin never copies it elsewhere.

## Layout

```text
<vault folder>/
  AGENTS.md             the vault's rules, from the template
  notes/<scope>/        one note per file; <scope> is a project's name or all
```

A note is a `.md` file under `notes/` that opens with a `---` line and closes its front matter with another.
Any other file there, such as an `index.md` an older tool generated, is not a note: the session start, the
search, and the lint leave it out.

## A note

File name: `<id>-<slug>.md`, the slug being the title in lowercase ASCII words joined by `-`, at most 60
characters. Front matter, one `key: value` per line, a list as `[a, b]`:

| Field | Type | Required | Domain | Written by |
| --- | --- | --- | --- | --- |
| `id` | string | Yes | `mem-` and four or more digits, unique in the vault | `tightship vault new`, the highest number plus one |
| `type` | string | Yes | `preference`, `decision`, `gotcha`, `project`, or `handoff` | The agent; `handoff` only through epic #6 |
| `project` | string | Yes | The scope; equal to the note's folder | The agent |
| `why` | string | Yes | At least 5 words: what not knowing the note would cost | The agent |
| `tags` | list of strings | No | Free words, searched like the body | The agent |
| `pinned` | string | No | `true`, or absent | The agent, with `--pin` |
| `source` | string | Yes | `user`, `session`, or `repo` | The agent |
| `harness` | string | No | Free; read and kept, never written by the plugin | Older tools |
| `created` | date | Yes | `YYYY-MM-DD`, when the fact was established | `tightship vault new`, today by default |
| `updated` | date | Yes | `YYYY-MM-DD`, when the fact was last revised | `tightship vault new`, today |

The body opens with `# <title>`. A `[[<stem>]]` in it links another note by its file name without `.md`.

## Outside the vault

| What | Where | Lifetime |
| --- | --- | --- |
| The prompt count of the checkpoint | `checkpoints/<session>` in the plugin's data folder: one integer | Deleted after 7 days without a prompt, and with the plugin on uninstall |
| The vault's folder for the agent's shell | One `export TIGHTSHIP_VAULT='<folder>'` line in the session's environment file | The session's |
