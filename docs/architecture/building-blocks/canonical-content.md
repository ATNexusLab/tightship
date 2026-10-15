# Canonical content

**Lives in:** `content/` · **Milestone:** M1, epics #1, #3, and #4

The only authored copy of every workflow piece
([NFR-MNT-01](../../requirements/non-functional/maintainability.md#nfr-mnt-01)): `skills/<name>/SKILL.md`
with its supporting files, `commands/<name>.md`, and `agents/<name>.md`. Each file opens with TOML front
matter between `+++` lines. `options.toml` lists the adopter's [options](../concepts/options.md) (M1, #3), and
`contract.md` is the [contract](../concepts/contract.md), its front matter naming the sections an option turns
off (M1, #4).

**It never** names a harness, a harness path, or a harness tool
([NFR-FLEX-06](../../requirements/non-functional/flexibility.md#nfr-flex-06)), and never writes another piece's
name literally. Both go through [tokens](../concepts/tokens.md). The build rejects a symbolic link in it, so
no file from outside the repository ships. `.gitattributes` checks every file out with LF line endings.

**To change it safely:** edit the one file, run `cargo xtask build`, and read the errors it lists. The
front matter keys and every rule the build enforces are in the [epic #1 spec](../../specs/canonical-source-and-build/build-the-plugin.md).
