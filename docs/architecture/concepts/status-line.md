# Status line

**Milestone:** M1, epic #7

Claude Code lets a plugin set no `statusLine`: a plugin's settings take only `agent` and `subagentStatusLine`.
So the workflow's status line lives in the adopter's user settings, and the `tightship` binary writes it there
only after the adopter says yes ([FR-STL-02](../../requirements/functional/status-line.md#fr-stl-02)).

## Consent

At each `startup`, `tightship hook status-line` checks the `status_line` [option](options.md). When it is on
and the status line is neither ours nor declined, the hook prints a question for the agent to put to the
adopter word for word, with the two commands to run on the answer: `tightship status-line enable` or
`tightship status-line decline`. A no is recorded and not asked again until the option is turned off and on.

## The record

`status-line.json`, in the plugin's data folder, holds `on` or `declined`, and with `on` the adopter's previous
`statusLine` when they had one ([data model](../../data-model/status-line.md)). `enable` writes it before it
touches the settings, so the settings never hold our status line without the previous value saved.

## Turning it off

With the option off, the next session start puts the previous `statusLine` back exactly, or removes the key
when there was none, and deletes the record
([FR-STL-03](../../requirements/functional/status-line.md#fr-stl-03)). Claude Code runs no plugin code on
uninstall, so the adopter turns the option off and opens one session before uninstalling.

## Following updates

Each plugin version installs in its own folder. While the option is on, every session start points our
`statusLine` command at the current folder and keeps the adopter's other keys of `statusLine`.

Every rule and message: the [epic #7 spec](../../specs/status-line/show-the-status-line.md).
