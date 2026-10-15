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

Claude Code keeps the state across an update and deletes it when the plugin is uninstalled from its last
scope. It never reads a project's or a folder's settings for it.
