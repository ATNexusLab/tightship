# Audit: i-have-adhd

Version audited: 0.4.1, from `ayghri/i-have-adhd`, read on 2026-11-10 from the copy Claude Code installed.
License: MIT, copyright 2026 Ayoub Ghriss. Nothing of its text is copied into tightship; the section that
replaces it is written anew.

## What it does

| Function | How it runs | Files |
| --- | --- | --- |
| Ten rules that shape a reply for a reader with ADHD | The skill `/i-have-adhd`, invoked by the adopter only, for the rest of the session | `skills/i-have-adhd/SKILL.md` |
| Always on | A `SessionStart` hook injects the skill's body when the flag file `~/.claude/.i-have-adhd-always` exists; the adopter creates it by hand | `hooks/hooks.json`, `hooks/always-on.mjs`, `.sh`, `.ps1` |
| Off | "stop adhd mode" or "normal mode" | The skill's text |

The ten rules: lead with the next action · number multi-step tasks · end with one concrete next action ·
suppress tangents · restate where the work stands · give time estimates in concrete units · make completed
work visible · state errors flatly · cap a list at five items per group · no preamble, recap, or closing
pleasantry. They break for "explain", a destructive step, a debugging spiral, a genuinely ambiguous request,
and an instruction of the harness.

## What it leaves on the machine

| Harness | Written by the plugin | Written by its install |
| --- | --- | --- |
| Claude Code | Nothing; it only reads `~/.claude/.i-have-adhd-always`, which the adopter creates | `enabledPlugins` and `extraKnownMarketplaces` in `settings.json`; `plugins/installed_plugins.json`, `plugins/known_marketplaces.json`, `plugins/marketplaces/i-have-adhd/`, `plugins/cache/i-have-adhd/`, `plugins/data/i-have-adhd-i-have-adhd/` |
| Cursor | Nothing | `~/.cursor/plugins/local/i-have-adhd/`, copied by tightship's installer |
| Antigravity CLI | Nothing | `~/.gemini/config/plugins/i-have-adhd/`, `plugins.i-have-adhd` in `~/.gemini/config/config.json`, its entry in `~/.gemini/config/import_manifest.json` |

It makes no network call.

## What the workflow used

The response shape. In Claude Code the rules reached a session only when the adopter invoked the skill or
created the flag file; the plugin installed neither, so most sessions ran without them.

## Overlap with tightship

The contract's hard line on outward and destructive actions already covers confirming a destructive step.
Nothing else overlaps.

## Decision

| Function | Decision | Where it goes |
| --- | --- | --- |
| The rules | Keep, condensed | The contract's new **Replies** section, always on |
| Time estimates in concrete units, "make wins visible" | Merged | **Replies**: "where it stands, what is done and what is next" |
| The flag file, the skill, "stop adhd mode" | Drop | Always on; the adopter overrides a convention by deciding in the session |
| The break for a debugging spiral | Drop | The **Code** section's bug rule and the grilling skill already cover a wrong assumption |
