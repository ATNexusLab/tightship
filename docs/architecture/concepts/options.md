# Options

**Milestone:** M1, epic #3; M3, epic #59

The contract, the memory layer, and the status line are options the adopter turns on or off
([FR-INS-03](../../requirements/functional/install.md#fr-ins-03)). `content/options.toml` lists them, and each
harness stores the adopter's choices:

- **Claude Code** (M1, #3): the manifest's `userConfig`. Claude Code asks for each option, stores it in the
  adopter's user settings, keeps it across an update, and deletes it on uninstall.
- **Antigravity CLI** (M3, #59): `${PLUGIN_DATA}/options.toml`
  (`~/.gemini/config/plugin_data/tightship/options.toml`), written by `install.sh` / `install.ps1` or by the
  adopter and preserved across plugin updates by Antigravity CLI.

## Reading an option

An option is off only when its value is exactly `false`. An absent value, or any other, is on, because an
option the adopter never set reaches no piece
([FR-INS-03.2](../../requirements/functional/install.md#fr-ins-03)).

| Where | Reads | Never set |
| --- | --- | --- |
| A hook in Claude Code | `CLAUDE_PLUGIN_OPTION_<NAME>`, `<NAME>` uppercased | Absent, so on |
| A hook or `tightship` in Antigravity CLI (M3, #59) | `${TIGHTSHIP_DATA}/options.toml` when `TIGHTSHIP_<NAME>` is unset | Absent file or key, so on |
| A skill, a command, or an agent | The token `{{option:<name>}}`, rendered `${user_config.<name>}` in Claude Code (M1, #6) and `<name>` in Antigravity CLI (M3, #59) | Not `false`, so on |

## The vault folder

Next to the three options, `content/options.toml` declares `vault_folder`, a setting of type `directory` that
belongs to the memory layer (M1, #5). Each entry names its `type`, `boolean` or `directory`. A `boolean`
renders with the default `true`; a `directory` renders with no default, so a folder the adopter never set
reaches `tightship` as an absent variable or key, and memory then has no vault.

A change reaches the hooks from the next session start. Each option's epic reads it this way: the contract
in #4, whose [hooks](contract.md) are the first to read one, memory in #5 and #6, the status line in #7. The
handoff command (#6) is the first piece to read one, to say memory is off.

Every rule and error message: the [epic #3 spec](../../specs/options/choose-the-options.md) and the
[epic #59 spec](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md).
