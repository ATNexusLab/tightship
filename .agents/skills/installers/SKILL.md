---
name: installers
description: Use when changing the installers — installer/install.sh and install.ps1, or a harness's install.sh and install.ps1 — how they are joined and how install.ps1 must fail.
---

# Installers

`cargo xtask build` joins the neutral `installer/install.sh` and `installer/install.ps1` with each
harness's `harnesses/<harness>/install.sh` and `install.ps1` into `dist/install.sh` and `dist/install.ps1`.
Every installer scenario of `cargo test --workspace` runs against both, so a change to one shell has its
counterpart in the other.

`install.ps1` and its harness parts fail with `throw "error: …"`, never `exit`: under `irm … | iex`, `exit`
closes the adopter's terminal. The one `catch` at the top prints the message as a plain line, since pwsh's
own rendering wraps it across decorated lines, and exits 1 only when the installer runs as a file.
