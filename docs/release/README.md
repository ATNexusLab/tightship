# Release

How a version of the plugin reaches adopters. The maintainer cuts every release; nothing here runs without
their word. The pipeline is in the [release spec](../specs/release-m1/release-the-plugin.md).

## Steps

1. **Set the version.** `version` in the root `Cargo.toml` is the release's, and `cargo check --workspace`
   updates `Cargo.lock`. Merge that change into `develop` through its pull request.
2. **Homologate `develop`.** Run the [checklist](checklist.md) on macOS, Linux, and Windows, each at
   Claude Code 2.1.295 or later, against a `cargo xtask build` of `develop`, as the checklist's preparation
   says. `develop` goes to `main` only when every row passes on all three.
3. **Release `develop` into `main`.** Open a pull request from `develop` to `main`, wait for its gates, and
   merge it as a merge commit, so every `Closes #<n>` it carries closes its issue.
4. **The merge publishes it.** The push to `main` runs `.github/workflows/release.yml`, which tags the merge
   commit `v<version>` from `Cargo.toml`, unless that version already has a release. It then builds the
   binary for the four platforms and publishes `v<version>` as the latest release, with
   `tightship-claude-code.zip`, `tightship-cursor.zip`, `tightship-antigravity-cli.zip`, `marketplace.json`,
   `cursor-marketplace.json`, `latest.json`, `install.sh`, and `install.ps1`. Adopters install and update to
   it from then on.
5. **Close the milestone** the release completes, and mark it released in the
   [roadmap](../roadmap/README.md) with the version and the date.

## When a row fails

`develop` does not go to `main`. The fix lands on `develop`, and the whole checklist runs again on the three
systems. On macOS, a binary that Gatekeeper
stops or kills is a failing row, and signing becomes a requirement change.
