# Release

How a version of the plugin reaches adopters. The maintainer cuts every release; nothing here runs without
their word. The pipeline is in the [release spec](../specs/release-m1/release-the-plugin.md).

## Steps

1. **Set the version.** `version` in the root `Cargo.toml` is the release's, and `cargo check --workspace`
   updates `Cargo.lock`. Merge that change into `develop` through its pull request.
2. **Release `develop` into `main`.** Open a pull request from `develop` to `main`, wait for its gates, and
   merge it as a merge commit, so every `Closes #<n>` it carries closes its issue.
3. **Push the tag on `main`.**

   ```sh
   git checkout main && git pull
   git tag v<version>
   git push origin v<version>
   ```

   `.github/workflows/release.yml` builds the binary for the four platforms and publishes the pre-release
   `v<version>` with `tightship-claude-code.zip` and `marketplace.json`.
4. **Run the [checklist](checklist.md)** on macOS, Linux, and Windows, each at Claude Code 2.1.295 or later,
   installing from the release's own `marketplace.json`, as the checklist's preparation says.
5. **Promote it**, only when every row passes on all three:

   ```sh
   gh release edit v<version> --prerelease=false --latest
   ```

   The repository's catalog, `.claude-plugin/marketplace.json`, points at the latest release's zip, so
   adopters now install and update to this version.
6. **Close the milestone** the release completes, and mark it released in the
   [roadmap](../roadmap/README.md) with the version and the date.

## When a row fails

The pre-release is not promoted. The fix ships as the next patch version, `v<major>.<minor>.<patch + 1>`,
from step 1, and the whole checklist runs again on the three systems. On macOS, a binary that Gatekeeper
stops or kills is a failing row, and signing becomes a requirement change.
