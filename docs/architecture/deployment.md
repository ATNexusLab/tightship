# Deployment

![C4 deployment](diagrams/deployment.drawio.svg)

Dashed boxes are not built yet; each belongs to the epic in its label.

| Environment | Runs | Milestone |
| --- | --- | --- |
| Maintainer's machine, macOS, Linux, or Windows | The build, `cargo xtask build` and `cargo xtask scan`, and the gates. The plugin is tried with `claude --plugin-dir dist/claude-code` | M1, #1 |
| GitHub | GitHub Actions runs the gates on the three systems at each pull request, and at a tag builds every target; a GitHub Release holds each harness's archive and its catalog | M1, #8 |
| Adopter's machine | Claude Code, the installed plugin, and the `tightship` binary its hooks run | M1, #2 and #5 |

## Packaging

The archive of a harness holds its generated plugin and the `tightship` binary for every target:

| Target | Binary |
| --- | --- |
| macOS, arm64 and x86_64 | One universal binary |
| Linux x64 | Statically linked with musl |
| Linux arm64 | Statically linked with musl |
| Windows x64 | `.exe`; Windows arm64 runs it emulated |

`bin/tightship` is a `sh` launcher that picks the binary for the machine, and the hooks run it through `sh`.
Until epic #8, `cargo xtask build` compiles only the target of the machine that builds (M1, #5). On macOS, the
binary carries the linker's ad-hoc signature only; epic #8 tests that it runs from an archive Claude Code
downloaded.

`cargo xtask package` writes the Claude Code archive, `tightship-claude-code.zip`, with mode `0755` on the
launcher and the binaries, and its catalog, `marketplace.json`, with an `archive` source naming the zip's URL
for the tag and its `sha256` (M1, #8).

## Installing

The adopter adds the catalogs of ponytail (`DietrichGebert/ponytail`), i-have-adhd
(`ayghri/i-have-adhd`), and humanizer (`blader/humanizer`), then the plugin's catalog, and runs `claude
plugin install tightship@tightship`. Claude Code installs the three companions as the plugin's
dependencies (M1, #2). Until epic #8 publishes a release, the plugin's catalog is the folder
`dist/claude-code/`. From it, the adopter adds the catalog committed at the repository root,
`.claude-plugin/marketplace.json`, with `claude plugin marketplace add ATNexusLab/tightship`. Its entry
is an `archive` source at the latest release's zip, with no `sha256`, so an update arrives through the
`version` of the promoted release's `plugin.json` and no release changes the file (M1, #8, #48). The
install guide, `docs/install/claude-code/`, holds every step, the migration from the pre-plugin install
included.

The adopter chooses the [options](concepts/options.md) in the dialog of `/plugin install`, or with `--config
<name>=false` on the shell's install, and changes them later with `/plugin configure` or `claude plugin
configure`. An update keeps them. Before uninstalling, the adopter turns `status_line` off and opens one
session, which puts their own status line back; Claude Code runs no plugin code on uninstall (M1, #7).
`claude plugin uninstall tightship@tightship --prune -y` then deletes the options, the plugin's data folder,
and the companions it installed; the adopter then removes the four catalogs with `claude plugin marketplace
remove` (M1, #3).

## Adopter prerequisites

Claude Code 2.1.295 or later (M1, #8). Nothing else on macOS and Linux. Git for Windows on Windows, because
Claude Code runs hooks under Git Bash and falls back to PowerShell without it. Git on every system when the
memory option is on.

## Delivery

The gates in `AGENTS.md` run on the maintainer's machine, and on the three systems at each pull request
in the [gate pipeline](building-blocks/gate-pipeline.md) (M1, #8). A release is cut from `main` by a tag
the maintainer pushes when they decide to release; the [release
pipeline](building-blocks/release-pipeline.md) publishes it as a pre-release, and the maintainer promotes
it after the [release checklist](../release/checklist.md) passes on macOS, Linux, and Windows at Claude
Code 2.1.295 or later (M1, #8).
