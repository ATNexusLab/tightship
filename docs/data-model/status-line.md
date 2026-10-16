# Status line

**Milestone:** M1, epic #7; M3, epic #59 · **Specs:**
[show the status line](../specs/status-line/show-the-status-line.md),
[Antigravity CLI plugin](../specs/antigravity-cli-plugin/antigravity-cli-plugin.md)

What the `status_line` [option](options.md) changes, and the record that lets it be undone.

## The user settings' `statusLine`

Written by `tightship hook status-line`, while the option is on, into the harness's user settings
(`~/.claude/settings.json` in Claude Code; `~/.gemini/antigravity-cli/settings.json` in Antigravity CLI,
M3, #59), never a project's or a folder's:

```json
{
  "statusLine": { "type": "command", "command": "\"<plugin folder>/bin/tightship\" status-line render" }
}
```

It is ours while its `command` runs the plugin's `bin/tightship`, `bin/tightship.cmd`, or
`bin/<platform>/tightship[.exe]` with `status-line render`, with `/` or `\` between folders (M3, #59; #55).
Every other key of the file keeps its value and its order.

## `status-line.json`

In the plugin's data folder (in Antigravity CLI, `~/.gemini/config/plugin_data/tightship/status-line.json`,
M3, #59), which the harness keeps across an update:

| Attribute | Type | Domain | Written by |
| --- | --- | --- | --- |
| `state` | string | `on`; a `declined` from an earlier version reads as no record | The session start |
| `previous` | any JSON value | The adopter's `statusLine` before ours; absent when they had none | The session start |

## Invariants

- While the user settings hold our status line, the record holds `on`, and `previous` when the adopter had a
  `statusLine`.
- Turning the option off deletes the record.

## Cursor CLI

**Milestone:** M2, epic #58 · **Spec:** [ship for Cursor](../specs/cursor-plugin/ship-for-cursor.md)

The Cursor editor has no status line. The CLI alone reads `statusLine` from `~/.cursor/cli-config.json`:

```json
{
  "statusLine": { "type": "command", "command": "\"${CURSOR_PLUGIN_ROOT}/bin/tightship\" status-line render" }
}
```

`tightship hook status-line` writes that entry while the option is on and saves the adopter's previous value
in `$HOME/.cursor/tightship/status-line.json`, with the same `state` and `previous` shape as
`status-line.json` above. FR-STL-01 to FR-STL-03 apply in the CLI only; the editor is out of scope for those
criteria ([epic #58 spec](../specs/cursor-plugin/ship-for-cursor.md)).
