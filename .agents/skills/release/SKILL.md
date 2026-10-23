---
name: release
description: Use when cutting a release of the plugins — the tag, what release.yml builds and publishes, the commands it runs, and the checklist that promotes a pre-release.
---

# Release

A release is cut only when the maintainer asks for it. It starts from `develop` merged into `main` as a merge
commit, and a tag `v<version>` pushed on `main`, where `<version>` is the built version.

The tag runs `release.yml`: its first job fails when the tagged commit is not on `main`, before anything is
built. Then it builds the binary for the four platforms, then the two commands below, and
publishes the zips, catalogs, and installers as a pre-release. It is the only workflow with
`contents: write`, on its pack job.

| Command | Does |
| --- | --- |
| `cargo xtask build --binaries <folder>` | The plugins with the four prebuilt binaries of `<folder>`, as `macos/tightship`, `linux-x64/tightship`, `linux-arm64/tightship`, and `windows-x64/tightship.exe`, nothing compiled |
| `cargo xtask package <tag>` | Packs the plugins into `dist/tightship-claude-code.zip`, `dist/tightship-antigravity-cli.zip`, and `dist/tightship-cursor.zip`, writes `dist/marketplace.json` and `dist/cursor-marketplace.json`, and `dist/install.sh` and `dist/install.ps1` with the tag and the Antigravity CLI and Cursor zips' `sha256` |

The pre-release is promoted only when every row of [the release checklist](../../../docs/release/checklist.md)
passes on macOS, Linux, and Windows.
