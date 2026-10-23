# Gate pipeline

**Lives in:** `.github/workflows/rust.yml`, `docs.yml`, and `scan.yml` · **Technology:** GitHub Actions ·
**Milestone:** M1, epic #8

Runs the gates of the `gates` project skill, `.agents/skills/gates/SKILL.md`, at each pull request into
`develop` or `main`, each workflow only when the pull request touches its paths. `rust.yml` runs clippy,
`cargo check`, rustfmt, `cargo xtask build`, and `cargo test --workspace` on Linux, macOS, and Windows, when
the Rust workspace or the content it builds changes; on Windows the steps run in Git Bash, as the plugin's
hooks do. `docs.yml` runs markdownlint and the docs build on Linux when Markdown or the docs site changes.
`scan.yml` runs `cargo xtask scan` on Linux at every pull request, since a secret can enter through any file.
It is the automated half of [NFR-FLEX-05](../../requirements/non-functional/flexibility.md#nfr-flex-05); the
release checklist is the other.

`rust.yml` installs gitleaks 8.30.1 for the scan's tests, checked against the sha256 pinned in the
workflow, and writes `.identity-terms` from the repository
secret `IDENTITY_TERMS`, so a pull request from a fork stops at the identity scan. It clears `USER`, since the
runner's user name, `runner`, is no identity of the maintainer and is a word the content uses. `scan.yml`
checks gitleaks the same way and fetches every branch with the whole history.

**It never** writes to the repository: its token has `contents: read`.

**To change it safely:** the triggers, paths, systems, and steps are in the
[epic #8 spec](../../specs/release-m1/release-the-plugin.md).
