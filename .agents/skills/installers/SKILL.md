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

Both installers print ASCII only, and `xtask/tests/install/` rejects any other byte: Windows PowerShell 5.1
decodes `irm` in the local code page, and a console font or a `LANG=C` terminal may lack the glyph.
`install.ps1` also runs on Windows PowerShell 5.1, which lacks what pwsh 7 adds: `` `e `` is a plain `e`
there, so escapes use `$esc`; `$PSStyle` does not exist, so colors come from `$ansi`; `Set-Content
-Encoding utf8` writes a BOM, so a file another tool parses goes through `[System.IO.File]::WriteAllText`
with `[System.Text.UTF8Encoding]::new($false)`; and `Test-Path` throws on a character .NET Framework
rejects, so a path the adopter typed is tested with `-LiteralPath ... -ErrorAction Ignore`.

CI runs the scenarios on pwsh only. To run them on 5.1, copy
`C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe` as `pwsh.exe` into an empty folder put first
on the `PATH`, and set `PSExecutionPolicyPreference=Bypass` and `PSModulePath` to
`C:\Program Files\WindowsPowerShell\Modules;C:\WINDOWS\system32\WindowsPowerShell\v1.0\Modules`: a 5.1
started under a different name keeps pwsh 7's module path and cannot find `Get-FileHash`.
