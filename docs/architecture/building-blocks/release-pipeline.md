# Release pipeline

**Lives in:** `.github/workflows/release.yml` · **Technology:** GitHub Actions ·
**Milestone:** M1, epic #8; M3, epic #59

Runs when the maintainer pushes a tag `v<version>` on `main`. It builds the `tightship` binary on macOS, Linux,
and Windows runners, then `cargo xtask build --binaries` packs the four binaries with the generated plugins and
`cargo xtask package` writes `tightship-claude-code.zip`, `tightship-antigravity-cli.zip` (M3, #59), and the
catalog `marketplace.json`, whose `archive` source names the Claude Code zip's URL for that tag and its
`sha256`. It publishes the archives, the catalog, `install.sh`, and `install.ps1` (M3, #59) as a pre-release.
The targets and the packing are in [deployment](../deployment.md).

The maintainer installs the pre-release on the three systems, runs the [release
checklist](../../release/checklist.md), and promotes it to the latest release, whose catalog URL the
install guide gives.

**It never** runs on a push of a branch or on a pull request, which the [gate pipeline](gate-pipeline.md)
covers; never commits to the repository; and never makes a release the latest by itself.

**To change it safely:** the jobs, the package, and their errors are in the
[epic #8 spec](../../specs/release-m1/release-the-plugin.md) and the
[epic #59 spec](../../specs/antigravity-cli-plugin/antigravity-cli-plugin.md).
