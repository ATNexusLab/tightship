# Audit: ponytail

Version audited: 5.3.3, from `DietrichGebert/ponytail`, read on 2026-11-10 from the copy Claude Code
installed. License: MIT, copyright 2026 DietrichGebert. Nothing of its text or code is copied into
tightship; the pieces that replace it are written anew, so no notice has to ship with them.

## What it does

| Function | How it runs | Files |
| --- | --- | --- |
| The rules: a "lazy senior developer" who takes the smallest change that fully works | `SessionStart` hook injects the body of `skills/ponytail/SKILL.md` | `hooks/ponytail-activate.js`, `ponytail-instructions.js`, `ponytail-runtime.js`, `ponytail-config.js` |
| Levels `lite`, `full`, `ultra`, `off`, and `review` | `UserPromptSubmit` hook reads `/ponytail <level>`, "stop ponytail", and "normal mode" | `hooks/ponytail-mode-tracker.js` |
| The same rules in every subagent | `SubagentStart` hook | `hooks/ponytail-subagent.js` |
| Codebase map: one line per folder with its top-level names, at most 2,000 characters | Called by the `SessionStart` hook, appended to the rules | `hooks/ponytail-map.js` |
| Status line badge `[PONYTAIL]` | A script it copies into the harness's config folder, which the agent is told to set as the status line | `hooks/ponytail-statusline.sh`, `.ps1` |
| Band: a panel above the prompt, a cost and token ledger per task, an optional webhook | A Claude Code mod module, after every turn | `hooks/band/*`, `hooks/ponytail-band-import.js` |
| `ponytail-audit` | Skill: a read-only audit of a whole repository, at most 20 findings | `skills/ponytail-audit/SKILL.md` |
| `ponytail-review` | Skill: the same lens on the current change | `skills/ponytail-review/SKILL.md` |
| `ponytail-debt` | Skill: lists every `shortcut:` marker as a ledger | `skills/ponytail-debt/SKILL.md` |
| `ponytail-gain` | Skill: prints fixed benchmark bars | `skills/ponytail-gain/SKILL.md` |
| `ponytail-help` | Skill: a reference card | `skills/ponytail-help/SKILL.md` |

The rules, in short: list every place the change must reach before writing; climb a ladder (does it need to
exist · already in the codebase · standard library · installed dependency · one line · the least code); no
abstraction nobody asked for; fix a bug at its root after finding every caller; one small test for new
non-trivial logic; mark a deliberate limit as `shortcut: <the limit>, <when to upgrade>`; never cut
validation, data-loss handling, security, or accessibility; end each reply with what was skipped and the risk.

## What it leaves on the machine

| Harness | Written by the plugin | Written by its install |
| --- | --- | --- |
| Claude Code | `~/.claude/.ponytail-active`, `~/.claude/ponytail-modes/`, `~/.claude/ponytail-statusline.sh` or `.ps1`, `~/.claude/.ponytail-statusline-nudged`, `~/.claude/.ponytail-band`, `~/.config/ponytail/` (its `config.json`, and `band/` with a CSV per project and `board.html`), a `statusLine` in `settings.json` when the adopter accepted it | `enabledPlugins` and `extraKnownMarketplaces` in `settings.json`; `plugins/installed_plugins.json`, `plugins/known_marketplaces.json`, `plugins/marketplaces/ponytail/`, `plugins/cache/ponytail/` with every version, `plugins/data/ponytail-ponytail/` |
| Cursor | `~/.cursor/.ponytail-active`, `~/.cursor/ponytail-modes/` | `~/.cursor/plugins/local/ponytail/`, copied by tightship's installer |
| Antigravity CLI | Nothing: no hooks run there | `~/.gemini/config/plugins/ponytail/`, `plugins.ponytail` in `~/.gemini/config/config.json`, its entry in `~/.gemini/config/import_manifest.json` |

Its own `scripts/uninstall.js` misses `~/.config/ponytail/band/`, `.ponytail-band`, and everything in
Antigravity CLI. The only network call is the band's webhook, off unless the adopter sets a URL.

## What the workflow used

The rules, the `shortcut:` marker the contract already states, and the codebase map. The workflow never named
the levels, the band, or the five one-shot skills.

## Overlap with tightship

- The contract's **Code** section already states the `shortcut:` marker, error handling, and validation at
  trust boundaries.
- `/audit` covers `ponytail-audit`; `/audit` on the diff and `/pr-review` cover `ponytail-review`.
- tightship has its own status line.

## Decision

| Function | Decision | Where it goes |
| --- | --- | --- |
| The rules | Keep, merged | The contract's **Code** section and its new **Replies** section, always on, with no levels |
| Levels and "stop ponytail" | Drop | A convention the adopter overrides by deciding in the session, as every convention |
| Rules in subagents | Drop | tightship's subagents do read-only analysis, so the rules about writing code have nothing to act on |
| Codebase map | Keep, rewritten | `tightship hook map`, at session start, behind the `contract` option |
| Status line badge | Drop | tightship's status line |
| Band, ledger, webhook | Drop | Harness-specific, and out of the workflow's scope |
| `ponytail-audit`, `ponytail-review` | Drop | `/audit` and `/pr-review` |
| `ponytail-debt` | Keep, merged | A step of `/audit` that lists every `shortcut:` marker |
| `ponytail-gain`, `ponytail-help` | Drop | Marketing and a reference card for what is dropped |
