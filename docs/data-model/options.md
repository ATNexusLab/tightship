# Options

**Milestone:** M1, epic #3; M3, epic #59 · **Specs:** [choose the options](../specs/options/choose-the-options.md),
[Antigravity CLI plugin](../specs/antigravity-cli-plugin/antigravity-cli-plugin.md)

The [Option](README.md#conceptual-model) entity as each harness stores it.

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
| `vault_folder` | string, a `directory` setting | An absolute or `~` folder; absent until the adopter sets it, with no default | Claude Code, when the adopter chooses (M1, #5) |

The `status_line` option's effect is the `statusLine` key of the user settings, written only after the adopter
consents ([status line](status-line.md), M1, #7).

Claude Code keeps the state across an update and deletes it when the plugin is uninstalled from its last
scope. It never reads a project's or a folder's settings for it.

## Antigravity CLI (M3, #59)

Antigravity CLI rejects `userConfig` in `plugin.json` under `agy plugin validate --strict`. The adopter's
option values live in `~/.gemini/config/plugin_data/tightship/options.toml` (`${PLUGIN_DATA}/options.toml`),
which Antigravity CLI preserves across plugin updates:

```toml
contract = true
memory = true
status_line = true
vault_folder = "~/ai-memory"
```

| Attribute | Type | Domain | Written by |
| --- | --- | --- | --- |
| `contract`, `memory`, `status_line` | boolean | `true` or `false`; an absent file or key means `true` | `install.sh` / `install.ps1` or the adopter |
| `vault_folder` | string | An absolute or `~` folder; absent or empty until set | `install.sh` / `install.ps1` or the adopter |
