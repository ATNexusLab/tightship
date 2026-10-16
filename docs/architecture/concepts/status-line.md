# Status line

**Milestone:** M1, epic #7; M3, epic #59

Neither Claude Code nor Antigravity CLI lets a plugin manifest set `statusLine`: the status line command lives
in the adopter's user settings (`~/.claude/settings.json` in Claude Code;
`~/.gemini/antigravity-cli/settings.json` in Antigravity CLI, M3, #59), and the `tightship` binary writes it
there only after the adopter says yes ([FR-STL-02](../../requirements/functional/status-line.md#fr-stl-02)).

## Consent

At each `startup`, `tightship hook status-line` checks the `status_line` [option](options.md). When it is on
and the status line is neither ours nor declined, the hook prints a question for the agent to put to the
adopter word for word, with the two commands to run on the answer: `tightship status-line enable` or
`tightship status-line decline` (or `install.sh` / `install.ps1` asks for consent during installation and
enables it, M3, #59). A no is recorded and not asked again until the option is turned off and on.

## The record

`status-line.json`, in the plugin's data folder, holds `on` or `declined`, and with `on` the adopter's previous
`statusLine` when they had one ([data model](../../data-model/status-line.md)). `enable` writes it before it
touches the settings, so the settings never hold our status line without the previous value saved.

## Turning it off

With the option off, the next session start puts the previous `statusLine` back exactly, or removes the key
when there was none, and deletes the record
([FR-STL-03](../../requirements/functional/status-line.md#fr-stl-03)). Neither harness runs plugin code on
uninstall, so the adopter turns the option off and opens one session (or runs `tightship hook status-line`)
before uninstalling.

## Following updates

Each plugin version installs in its own folder. While the option is on, every session start points our
`statusLine` command at the current folder and keeps the adopter's other keys of `statusLine`.

Every rule and message: the [epic #7 spec](../../specs/status-line/show-the-status-line.md) and the
[epic #59 spec](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md).
