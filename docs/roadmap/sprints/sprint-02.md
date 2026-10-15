# Sprint 2 — 2026-10-19 → 2026-10-25

**Goal:** An adopter installs the plugin from this repository's catalog, gets the skills, the commands, the
verifier, and the companions, and chooses which options are on.
**Requirements baseline:** v1.0.2
**Items:** [#2](https://github.com/ATNexusLab/tightship/issues/2),
[#3](https://github.com/ATNexusLab/tightship/issues/3)

Neither epic has a spec yet. The sprint opens with `/spec` then `/epic` for #2, then for #3, before any code.
Options (#3) builds on the installed plugin of #2.

## Result — 2026-10-15

Goal reached. Epic #2 merged into `develop` with [PR #16](https://github.com/ATNexusLab/tightship/pull/16),
and epic #3 with [PR #17](https://github.com/ATNexusLab/tightship/pull/17). They close when the M1 release
reaches `main`.

| Item | Result |
| --- | --- |
| [#2](https://github.com/ATNexusLab/tightship/issues/2) Install with skills, commands and verifier | Done |
| [#14](https://github.com/ATNexusLab/tightship/issues/14) Install the plugin with its companions from the built catalog | Done |
| [#13](https://github.com/ATNexusLab/tightship/issues/13) Let the verifier run tests in a private copy (FR-VER-02) | Done |
| [#3](https://github.com/ATNexusLab/tightship/issues/3) Options | Done |
| [#15](https://github.com/ATNexusLab/tightship/issues/15) Declare the options and let the harness keep them | Done |

The sprint ran before its planned start and finished early; its dates stay as planned. Change request #13 moved
the requirements to v1.1.0 during the sprint, and epic #3's spec amended ADR 0001: the options' state belongs
to the harness, not to the plugin. Nothing carries over.
