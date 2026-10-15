# Sprint 4 — 2026-11-02 → 2026-11-08

**Goal:** An adopter hands a session off and claims it on another machine, turns on the status line after
consenting, and installs M1 by following the install guide on macOS, Linux, and Windows.
**Requirements baseline:** v1.4.1
**Items:** [#6](https://github.com/ATNexusLab/tightship/issues/6),
[#7](https://github.com/ATNexusLab/tightship/issues/7),
[#8](https://github.com/ATNexusLab/tightship/issues/8)

None of the three epics has a spec yet. The sprint opens with `/spec` then `/epic` for #6, then #7, then #8,
before any code. Handoff (#6) builds on the vault of #5 and also restores the Closeout step's memory item.
Status line (#7) adds its command to the `tightship` binary. Release M1 (#8) comes last: it builds the binary
for every platform and releases what the other epics shipped.
