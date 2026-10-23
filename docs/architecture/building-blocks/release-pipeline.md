# Release pipeline

**Lives in:** `.github/workflows/release.yml` · **Technology:** GitHub Actions ·
**Milestone:** M1, epic #8; M3, epic #59

Runs at each push to `main`: it tags the commit `v<version>`, the version of `Cargo.toml`, and stops when
that version already has a release. It builds the `tightship` binary on macOS, Linux, and Windows runners,
then `cargo xtask build --binaries` packs the four binaries with the generated plugins and
`cargo xtask package` writes `tightship-claude-code.zip`, `tightship-antigravity-cli.zip` (M3, #59), and
`tightship-cursor.zip`, and the catalogs `marketplace.json` and `cursor-marketplace.json`, each with an
`archive` source that names its own zip's URL for that tag and its `sha256`. It fills the tag and the `sha256`
of the Antigravity CLI and Cursor zips into the `install.sh` and `install.ps1` (M3, #59; #85) that the build
assembled, and the installers check that `sha256` before installing each zip. It also writes `latest.json`,
with the version, the tag, and each zip with its `sha256`, which the plugin's self-update reads from the latest
release (M4, FR-INS-10, [ADR 0003](../../decisions/0003-the-plugin-updates-itself-from-github-releases.md)).
It publishes the archives, the catalogs, `latest.json`, and the two installers as the latest release.
The targets and the packing are in [deployment](../deployment.md).

The [release checklist](../../release/checklist.md) runs on a build of `develop` on the three systems before
the merge into `main`; the merge is the promotion.

**It never** publishes a tag on a commit outside `main`, which its first job stops before any build; never
runs on a push of a branch or on a pull request, which the [gate pipeline](gate-pipeline.md)
covers; and never commits to the repository.

**To change it safely:** the jobs, the package, and their errors are in the
[epic #8 spec](../../specs/release-m1/release-the-plugin.md) and the
[epic #59 spec](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md).
