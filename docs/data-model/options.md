# Options

**Milestone:** M1, epic #3 · **Spec:** [choose the options](../specs/options/choose-the-options.md)

The [Option](README.md#conceptual-model) entity as the harness stores it. The plugin declares the options and
never writes their state.

## Claude Code

One value per option, and the memory layer's vault folder (M1, #5), under the plugin's identifier in the
adopter's user settings:

```json
{
  "pluginConfigs": {
    "tightship@tightship": { "options": { "contract": true, "memory": true, "vault_folder": "/home/adopter/vault" } }
  }
}
```

| Attribute | Type | Domain | Written by |
| --- | --- | --- | --- |
| Name | key | `contract`, `memory`, `status_line`, from `content/options.toml` | The maintainer, in the manifest |
| State | boolean | `true` or `false`; the key is absent until the adopter sets it | Claude Code, when the adopter chooses |
| `vault_folder` | string, a `directory` setting | An absolute folder; absent until the adopter sets it, with no default | Claude Code, when the adopter chooses (M1, #5) |

The `status_line` option's effect is the `statusLine` key of the user settings, written only after the adopter
consents ([status line](status-line.md), M1, #7).

Claude Code keeps the state across an update and deletes it when the plugin is uninstalled from its last
scope. It never reads a project's or a folder's settings for it.

## Cursor

**Milestone:** M2, epic #58 · **Spec:** [ship for Cursor](../specs/cursor-plugin/ship-for-cursor.md)

One file the plugin owns, because Configure values do not reach hooks:

`$HOME/.cursor/tightship/options.json`

```json
{
  "contract": true,
  "memory": true,
  "status_line": true,
  "vault_folder": "/home/adopter/vault"
}
```

| Attribute | Type | Domain | Written by |
| --- | --- | --- | --- |
| Name | key | `contract`, `memory`, `status_line`, from `content/options.toml` | The maintainer, in the manifest `variables` and in this file's schema |
| State | boolean | `true` or `false`; absent is on for hooks, as in epic #3 | `tightship options set`, or the install flow in the guide |
| `vault_folder` | string | An absolute folder, or `~` with a path as in FR-MEM-09.2 | `tightship options set vault_folder <path>` |

The manifest's `variables` mirror the options for Customize → Configure only. Hooks read this file through
the binary.

Uninstall removes `$HOME/.cursor/tightship/` whole, including `options.json` and status-line backup state.
