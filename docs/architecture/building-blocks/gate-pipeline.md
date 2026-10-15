# Gate pipeline

**Lives in:** `.github/workflows/ci.yml` · **Technology:** GitHub Actions · **Milestone:** M1,
epic #8

Runs every gate of `AGENTS.md` on Linux, macOS, and Windows at each pull request into `develop` or `main`:
clippy, `cargo check`, rustfmt, `cargo xtask build`, `cargo test --workspace`, and `cargo xtask scan` on each
system, and markdownlint and the docs build on Linux. On Windows the steps run in Git Bash, as the plugin's
hooks do. It is the automated half of
[NFR-FLEX-05](../../requirements/non-functional/flexibility.md#nfr-flex-05); the release checklist is the
other.

It installs gitleaks 8.30.1, fetches every branch with the whole history, and writes `.identity-terms` from the
repository secret `IDENTITY_TERMS`, so a pull request from a fork stops at the identity scan.

**It never** writes to the repository: its token has `contents: read`.

**To change it safely:** the triggers, systems, and steps are in the
[epic #8 spec](../../specs/release-m1/release-the-plugin.md).
