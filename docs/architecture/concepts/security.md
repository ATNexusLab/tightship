# Security

The repository is public, and the plugin runs on machines that are not the maintainer's.

## What never ships

| What | Requirement | Held by |
| --- | --- | --- |
| The maintainer's name, email addresses, user names, and home paths | [NFR-SEC-01](../../requirements/non-functional/security.md#nfr-sec-01) | The identity scan, the last step of `cargo xtask build`, over every file of `dist/` |
| Any secret, in any commit | [NFR-SEC-02](../../requirements/non-functional/security.md#nfr-sec-02) | gitleaks 8.30.1 over the whole history, in `cargo xtask scan` |
| Any vault note | [NFR-SEC-02](../../requirements/non-functional/security.md#nfr-sec-02) | `cargo xtask scan`, over every tracked file |

The identity scan reads its terms from git's configuration, the operating system, and `.identity-terms`, a
file git ignores. The terms never enter the repository, and an error names the file and line without
printing the term, so a CI log cannot leak it.

## What the plugin may reach

From [ADR 0001](../../decisions/0001-rust-build-and-tokenized-canonical-content.md):

- **Network:** only the git remote of the adopter's vault, when the vault is synced
  ([FR-MEM-11](../../requirements/functional/memory.md#fr-mem-11), M1, #5), which also carries the handoff
  (M1, #6). A handoff claim runs `git fetch` in the project's repository, through the adopter's git.
- **Disk:** the adopter's vault, and the `statusLine` key of the harness's user settings, after the adopter
  consents ([FR-STL-02](../../requirements/functional/status-line.md#fr-stl-02), [status line](status-line.md),
  M1, #7). In the harness's own places, the plugin also writes, in the plugin's data folder, the checkpoint's
  prompt counter ([memory](memory.md), M1, #5) and `status-line.json` (M1, #7), and one `export` line per
  session in the session's environment file (M1, #5). The options' state is the harness's own: it keeps it in
  its user settings ([options](options.md), M1, #3).

The build writes only to `dist/`.
