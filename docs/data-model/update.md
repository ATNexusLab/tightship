# Update

**Specs:** [keep the plugin updated](../specs/auto-update/update-the-plugin.md)

What the `auto_update` [option](options.md) reads and writes. Cursor and Antigravity CLI keep two files in
the plugin's data folder (`TIGHTSHIP_DATA`), which survives an update. Claude Code keeps one key in its user
settings.

## `update.json`

Written by `tightship update`, read and cleared by `tightship hook update`:

```json
{
  "checked_at": "2026-10-23T12:00:00Z",
  "notice": { "kind": "updated", "from": "1.1.0", "to": "1.2.0" },
  "error": "cannot download https://github.com/<owner>/tightship/releases/latest/download/latest.json: ..."
}
```

| Field | Type | Domain | Written by |
| --- | --- | --- | --- |
| `checked_at` | string | RFC 3339 UTC; absent until the first check. A check starts only when it is absent or more than 86,400 seconds old | `update`, before it downloads |
| `notice` | object | `kind` is `updated` or `refused`; `from` and `to` are `X.Y.Z`. Absent when there is nothing to say | `update`; removed by `hook update` once printed |
| `error` | string | The message of the last failed check; absent after one that succeeds | `update` |

A file that is not valid JSON, or breaks the table, reads as empty and is overwritten.

## `update.lock`

An empty file that `update` creates with `create_new` and removes when it ends, so two sessions never update
at once. A lock older than 3,600 seconds was left by a process that died, and the next `update` takes it.

## The plugin's folder during a swap

`update` extracts the release into `<plugin folder>/.update/`, moves each current file aside as
`<file>.tightship-old`, and moves the new ones in. A failure moves every file back. After a swap, the
`.update/` folder and every `.tightship-old` file are removed, except a binary still running on Windows,
which the next update removes. Files under a top-level folder whose name starts with `.` stay unless the
release carries them.

## Claude Code's marketplace entry

`tightship hook marketplace-auto-update` writes `autoUpdate` on the plugin's entry in the user settings,
following the option, and creates the entry from the manifest's `repository` when it is missing:

```json
{
  "extraKnownMarketplaces": {
    "tightship": { "source": { "source": "github", "repo": "<owner>/tightship" }, "autoUpdate": true }
  }
}
```

Every other key of the file keeps its value and its order, and the file is written only when the value
changes.
