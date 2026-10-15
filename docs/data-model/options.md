# Options

**Milestone:** M1, epic #3 · **Spec:** [choose the options](../specs/options/choose-the-options.md)

The [Option](README.md#conceptual-model) entity as the harness stores it. The plugin declares the options and
never writes their state.

## Claude Code

One boolean per option, under the plugin's identifier in the adopter's user settings:

```json
{
  "pluginConfigs": {
    "tightship@tightship": { "options": { "contract": true, "memory": false } }
  }
}
```

| Attribute | Type | Domain | Written by |
| --- | --- | --- | --- |
| Name | key | `contract`, `memory`, `status_line`, from `content/options.toml` | The maintainer, in the manifest |
| State | boolean | `true` or `false`; the key is absent until the adopter sets it | Claude Code, when the adopter chooses |

Claude Code keeps the state across an update and deletes it when the plugin is uninstalled from its last
scope. It never reads a project's or a folder's settings for it.
