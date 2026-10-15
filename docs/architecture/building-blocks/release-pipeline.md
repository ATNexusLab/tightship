# Release pipeline

**Lives in:** `.github/workflows/`, not built yet · **Technology:** GitHub Actions · **Milestone:** M1,
epic #8

Builds the `tightship` binary on macOS, Linux, and Windows runners at release, packs it with each harness's
generated plugin, and attaches one archive per harness to a GitHub Release. The targets and the packing
are in [deployment](../deployment.md).

**It never** runs on a push or a pull request: until epic #8, the gates run on the maintainer's machine.
