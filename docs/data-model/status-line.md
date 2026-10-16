# Status line

**Milestone:** M1, epic #7; M3, epic #59 · **Specs:**
[show the status line](../specs/status-line/show-the-status-line.md),
[Antigravity CLI plugin](../specs/antigravity-cli-plugin/antigravity-cli-plugin.md)

What the `status_line` [option](options.md) changes, and the record that lets it be undone.

## The user settings' `statusLine`

Written by `tightship status-line enable` into the harness's user settings (`~/.claude/settings.json` in Claude
Code; `~/.gemini/antigravity-cli/settings.json` in Antigravity CLI, M3, #59), never a project's or a folder's:

```json
{
  "statusLine": { "type": "command", "command": "\"<plugin folder>/bin/tightship\" status-line render" }
}
```

It is ours while its `command` ends in `/bin/tightship" status-line render` or `\bin\tightship.cmd"
status-line render` (M3, #59). Every other key of the file keeps its value and its order.

## `status-line.json`

In the plugin's data folder (in Antigravity CLI, `~/.gemini/config/plugin_data/tightship/status-line.json`,
M3, #59), which the harness keeps across an update:

| Attribute | Type | Domain | Written by |
| --- | --- | --- | --- |
| `state` | string | `on` or `declined` | `enable`, `decline`, or the session start |
| `previous` | any JSON value | The adopter's `statusLine` before `enable`; absent when they had none | `enable` |

## Invariants

- While the user settings hold our status line, the record holds `on`, and `previous` when the adopter had a
  `statusLine`.
- Turning the option off deletes the record.
