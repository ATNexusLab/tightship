# Deployment

![C4 deployment](diagrams/deployment.drawio.svg)

Dashed boxes are not built yet; each belongs to the epic in its label.

| Environment | Runs | Milestone |
| --- | --- | --- |
| Maintainer's machine, macOS, Linux, or Windows | The build, `cargo xtask build` and `cargo xtask scan`, and the gates. The plugins are tried with `claude --plugin-dir dist/claude-code` and `agy plugin validate dist/antigravity-cli --strict` | M1, #1; M3, #59 |
| GitHub | GitHub Actions runs the gates on the three systems at each pull request, and at a tag builds every target; a GitHub Release holds each harness's archive, the catalog, and `install.sh` / `install.ps1` | M1, #8; M3, #59 |
| Adopter's machine | Claude Code and Antigravity CLI, the installed plugin, and the `tightship` binary its hooks run | M1, #2 and #5; M3, #59 |

## Packaging

The archive of a harness holds its generated plugin and the `tightship` binary for every target:

| Target | Binary |
| --- | --- |
| macOS, arm64 and x86_64 | One universal binary |
| Linux x64 | Statically linked with musl |
| Linux arm64 | Statically linked with musl |
| Windows x64 | `.exe`; Windows arm64 runs it emulated |

`bin/tightship` is a `sh` launcher that picks the binary for the machine, and `bin/tightship.cmd` is a Windows
batch launcher that every plugin ships, for PowerShell, cmd, and hooks run under `cmd /c` (M3, #59, #116).
Until epic #8, `cargo xtask build` compiles only the target of the machine that builds (M1, #5). On macOS, the
binary carries the linker's ad-hoc signature only; epic #8 tests that it runs from an archive Claude Code
downloaded.

`cargo xtask package` writes the Claude Code archive (`tightship-claude-code.zip`) and the Antigravity CLI
archive (`tightship-antigravity-cli.zip`, M3, #59), with mode `0755` on the launchers and the binaries, and the
catalog `marketplace.json`, with an `archive` source naming the Claude Code zip's URL for the tag and its
`sha256` (M1, #8).

## Installing

An adopter can install for Claude Code, Antigravity CLI, or both in one command with `install.sh` (`curl -fsSL
.../install.sh | sh`) on macOS, Linux, and Git Bash or `install.ps1` (`irm .../install.ps1 | iex`) on Windows
PowerShell, which prompts interactively for the harnesses and the [options](concepts/options.md) (or takes CLI
flags), installs the three companions and `tightship`, and writes the options (M3, #59).

**Claude Code** (M1, #2, #3, #8): the adopter (or `install.sh` / `install.ps1`) adds the catalogs of ponytail
(`DietrichGebert/ponytail`), i-have-adhd (`ayghri/i-have-adhd`), and humanizer (`blader/humanizer`), then the
plugin's catalog, and runs `claude plugin install tightship@tightship`. Claude Code installs the three
companions as the plugin's dependencies. The catalog committed at the repository root,
`.claude-plugin/marketplace.json`, has an `archive` source at the latest release's zip, with no `sha256`, so an
update arrives through the `version` of the promoted release's `plugin.json` and no release changes the file
(M1, #8, #48). The adopter chooses the [options](concepts/options.md) in the dialog of `/plugin install`, or
with `--config <name>=false` on the shell's install, and changes them later with `/plugin configure` or `claude
plugin configure`. Before uninstalling, the adopter turns `status_line` off and opens one session, which puts
their own status line back; `claude plugin uninstall tightship@tightship --prune -y` then deletes the options,
the plugin's data folder, and the companions it installed.

**Antigravity CLI** (M3, #59): `install.sh` / `install.ps1` (or the manual commands in
`docs/install/antigravity-cli/`) runs `agy plugin install` for `DietrichGebert/ponytail` and
`ayghri/i-have-adhd`, installs `blader/humanizer`'s `SKILL.md` into `~/.gemini/config/skills/humanizer/`,
extracts `tightship-antigravity-cli.zip` and runs `agy plugin install`, writes
`~/.gemini/config/plugin_data/tightship/options.toml`, and runs `tightship hook status-line`. The adopter
changes options later by editing `options.toml` or rerunning the installer, and updates by rerunning the
installer or `agy plugin install`. Before uninstalling, the adopter sets `status_line = false` in
`options.toml` and runs `tightship hook status-line` (or opens one session) to restore their previous
`statusLine`, then removes `~/.gemini/config/plugins/tightship` and `~/.gemini/config/plugin_data/tightship`.

## Adopter prerequisites

Claude Code 2.1.295 or later (M1, #8) or Antigravity CLI 1.4.0 or later (M3, #59). Nothing else on macOS and
Linux. Git for Windows on Windows for Claude Code, because Claude Code runs hooks under Git Bash and falls back
to PowerShell without it (Antigravity CLI on Windows runs `bin/tightship.cmd` under `cmd /c`, M3, #59). Git on
every system when the memory option is on.

## Delivery

The gates of `.agents/skills/gates/SKILL.md` run on the maintainer's machine, and on the three systems at
each pull request in the [gate pipeline](building-blocks/gate-pipeline.md) (M1, #8). A release is cut from
`main` by a tag the maintainer pushes when they decide to release; the
[release pipeline](building-blocks/release-pipeline.md) publishes it as a pre-release, and the maintainer promotes
it after the [release checklist](../release/checklist.md) passes on macOS, Linux, and Windows (M1, #8;
M3, #59).
