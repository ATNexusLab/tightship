# Options

**Milestone:** M1, epic #3

The contract, the memory layer, and the status line are options the adopter turns on or off
([FR-INS-03](../../requirements/functional/install.md#fr-ins-03)). `content/options.toml` lists them, and each
harness renders the list into its own user configuration. In Claude Code that is the manifest's `userConfig`:
Claude Code asks for each option, stores it in the adopter's user settings, keeps it across an update, and
deletes it on uninstall. The plugin writes no option state.

## Reading an option

An option is off only when its value is exactly `false`. An absent value, or any other, is on, because an
option the adopter never set reaches no piece
([FR-INS-03.2](../../requirements/functional/install.md#fr-ins-03)).

| Where | Reads | Never set |
| --- | --- | --- |
| A hook | The environment variable `CLAUDE_PLUGIN_OPTION_<NAME>`, `<NAME>` uppercased | Absent, so on |
| A skill or an agent | Nothing: it never reads an option | The literal `${user_config.<name>}` would reach it |

A change reaches the hooks from the next session start. Each option's epic reads it this way: the contract
in #4, whose [hooks](contract.md) are the first to read one, memory in #5 and #6, the status line in #7.

Every rule and error message: the [epic #3 spec](../../specs/options/choose-the-options.md).
