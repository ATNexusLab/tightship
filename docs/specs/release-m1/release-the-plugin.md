# #8 — Release the Claude Code plugin

> **Status:** published
> **Profile:** API
> **Module:** `.github/workflows/`, `xtask/`, `tightship/launcher.sh`, `docs/install/`, `docs/release/`
> **Epic:** #8
> **Requirements:** FR-INS-07, FR-INS-08, FR-CTR-05, NFR-FLEX-01, NFR-FLEX-05, BR-01
> **Sprint:** 4

The gates run in GitHub Actions on macOS, Linux, and Windows for every pull request. A tag pushed on `main`
builds the `tightship` binary for every target, packs the plugin into one zip, and publishes it with its
catalog as a pre-release. The maintainer installs that pre-release on the three systems at the minimum
Claude Code version, runs the release checklist, and promotes it. The adopter follows the install guide to
install, configure, update, remove, or migrate to the plugin.

Terms from the SRS, as used here:

- **Adopter**: whoever installs the plugin, the maintainer included
  ([overview](../../requirements/overview.md#stakeholders)).
- **Baseline**: commit `1f41699` of the maintainer's private configuration, the pre-plugin workflow
  ([glossary](../../requirements/glossary.md)).
- **Pre-plugin install**: the workflow installed by merging the baseline's files into the harness's user
  directory ([FR-INS-07](../../requirements/functional/install.md#fr-ins-07)).
- **Companion plugins**: ponytail, i-have-adhd, and humanizer, which the plugin installs as its dependencies
  ([BR-04](../../requirements/business-rules.md#br-04)).
- **Option**: a part of the plugin the adopter turns on or off: the contract, the memory layer, or the
  status line ([FR-INS-03](../../requirements/functional/install.md#fr-ins-03)).
- **Milestone M1**: the Claude Code plugin, the first of four harnesses released one at a time
  ([BR-01](../../requirements/business-rules.md#br-01)).

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-15, on Claude Code's marketplace and plugin-loading references and its
changelog:

1. **The catalog is an asset of each release, never a committed file.** Every release attaches
   `tightship-claude-code.zip` and `marketplace.json`; the catalog's entry has an `archive` source with the
   zip's URL for that tag and its `sha256`. The adopter adds the catalog from
   `https://github.com/ATNexusLab/tightship/releases/latest/download/marketplace.json`, and Claude Code finds
   an update through the `version` of `plugin.json`, which it reads before the entry's. No pipeline commits to
   `main`. This amends decision 1 of the [epic #2 spec](../install-with-skills-commands-and-verifier/install-the-plugin.md),
   which put the catalog at the repository root.
2. **A release starts from a tag `v<version>` pushed on `main`**, and publishes a pre-release. The maintainer
   installs it through its own catalog URL on the three systems, so the bytes tested are the bytes shipped and
   the macOS binary is tested from an archive Claude Code downloaded. The maintainer then promotes it to the
   latest release.
3. **The gates also run on every pull request into `develop` or `main`**, on macOS, Linux, and Windows. This
   corrects the release pipeline's "never runs on a push or a pull request". A test that fails on one system
   in code an earlier epic built is a defect of NFR-FLEX-05, which this epic owns, and task 1 fixes it.
4. **The migration from the pre-plugin install is a section of the install guide**, with steps the adopter
   follows by hand. The list of what to remove comes from the baseline commit. No program migrates.
5. **The minimum Claude Code version is 2.1.295.** The `archive` source needs 2.1.224, and only from 2.1.295
   does the vault variable a `SessionStart` hook exports reach the Bash tool after an in-app `/resume` or
   `/branch`. The release checklist runs at exactly that version. This settles OQ-04 for Claude Code.
6. **The guide lives in `docs/install/claude-code/`**, one small file per subject with a `README.md` index,
   and a new `README.md` at the repository root points to it. The license is MIT. The first release is
   `1.0.0`.
7. **Four tasks**: the gates in CI with the fixes the three systems reveal; the release pipeline; the guide,
   once epics #6 and #7 are merged into `develop`; and the release checklist, run on the three systems. The
   release itself is cut only when the maintainer asks.

## Acceptance Criteria

### Contract

| Command or trigger | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| Pull request into `develop` or `main`, opened or pushed | Maintainer | Yes | `.github/workflows/ci.yml` runs every gate on the three systems |
| `cargo xtask build` | Maintainer | Yes | Unchanged: the plugin with the binary of the machine that builds |
| `cargo xtask build --binaries <folder>` | Release pipeline | Yes | The plugin with the four prebuilt binaries of `<folder>`, nothing compiled |
| `cargo xtask package <tag>` | Release pipeline | Yes | `dist/tightship-claude-code.zip` and `dist/marketplace.json`, from a `dist/claude-code/` holding the four binaries |
| `git push origin v<version>`, on a commit of `main` | Maintainer, when releasing | No | `.github/workflows/release.yml` publishes the pre-release `v<version>` |
| `gh release edit v<version> --prerelease=false --latest` | Maintainer, after the checklist | Yes | Promotes the pre-release; the catalog URL of the guide now serves it |
| `claude plugin marketplace add https://github.com/ATNexusLab/tightship/releases/latest/download/marketplace.json` | Adopter | Yes | Adds the catalog `tightship` |

### Request

#### `cargo xtask build --binaries <folder>`

| Argument | Type | Required | Validation |
| --- | --- | --- | --- |
| `<folder>` | path | Yes, with `--binaries` | Holds `macos/tightship`, `linux-x64/tightship`, `linux-arm64/tightship`, and `windows-x64/tightship.exe`; each a regular file |

Every check of `cargo xtask build` runs as before. Only the compile step is replaced: each of the four files is
copied to `dist/claude-code/bin/<platform>/`, with mode `0755` where the system has modes.

#### `cargo xtask package <tag>`

| Argument | Type | Required | Validation |
| --- | --- | --- | --- |
| `<tag>` | string | Yes | Exactly `v` followed by the workspace's `version` in `Cargo.toml` |

It reads `dist/claude-code/`, which must hold the four platform folders under `bin/`, and the
`[plugin] repository` of `harnesses/claude-code/harness.toml`.

#### `.github/workflows/ci.yml`

| Key | Value |
| --- | --- |
| Trigger | `pull_request` into `develop` or `main` |
| Systems | `ubuntu-latest`, `macos-latest`, `windows-latest`; on Windows, steps run in Git Bash |
| On each system | `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check --workspace --all-targets`, `cargo fmt --all --check`, `cargo xtask build`, `cargo test --workspace`, `cargo xtask scan` |
| On Linux only | `npx markdownlint-cli2@0.23.3` and `uvx zensical==0.0.70 build`, whose result is the same on every system |
| Before the gates | gitleaks 8.30.1 installed from its release; `git config user.name` and `user.email` set to the Actions bot; `.identity-terms` written from the repository secret `IDENTITY_TERMS`; the whole history and every branch fetched |
| Permissions | `contents: read` |

#### `.github/workflows/release.yml`

| Key | Value |
| --- | --- |
| Trigger | `push` of a tag matching `v*` |
| Build jobs | `macos-latest`: `aarch64-apple-darwin` and `x86_64-apple-darwin`, merged by `lipo` into `macos/tightship`. `ubuntu-latest`: `x86_64-unknown-linux-musl` into `linux-x64/tightship`. `ubuntu-24.04-arm`: `aarch64-unknown-linux-musl` into `linux-arm64/tightship`. `windows-latest`: `x86_64-pc-windows-msvc` into `windows-x64/tightship.exe` |
| Each build | `cargo build --release --locked --package tightship --target <target>`, with the path remaps of the host build |
| Pack job | `ubuntu-latest`, after the four builds: `.identity-terms` from `IDENTITY_TERMS`, `cargo xtask build --binaries binaries`, `cargo xtask package "$GITHUB_REF_NAME"`, then `gh release create "$GITHUB_REF_NAME" --prerelease --title "tightship $GITHUB_REF_NAME" --generate-notes` with the two files of `dist/` |
| Permissions | `contents: write` on the pack job, `contents: read` on the builds |

Every action the two workflows use is pinned to a full commit SHA, with its release tag in the step's `name`.

### Response

#### What `cargo xtask build --binaries <folder>` writes

`dist/claude-code/` as a host build, plus three platform folders. The success output keeps its form, its file
count the host build's plus 3:

```text
Built dist/claude-code/: 23 pieces (13 skills, 9 commands, 1 agent), 39 files.
Try it: claude --plugin-dir dist/claude-code
```

The counts are those of `develop` on 2026-10-15; epics #6 and #7 change them.

#### What `cargo xtask package <tag>` writes

`dist/tightship-claude-code.zip` holds every file of `dist/claude-code/` at the top of the zip, except
`.claude-plugin/marketplace.json`, the local catalog. `bin/tightship` and each binary under `bin/` have the
Unix mode `0755`, every other file `0644`, whatever system packs it.

`dist/marketplace.json`:

```json
{
  "name": "tightship",
  "owner": { "name": "ATNexusLab" },
  "description": "Skills, a command per step of the development loop, and a read-only verifier",
  "allowCrossMarketplaceDependenciesOn": ["ponytail", "i-have-adhd", "humanizer"],
  "plugins": [
    {
      "name": "tightship",
      "source": {
        "source": "archive",
        "url": "https://github.com/ATNexusLab/tightship/releases/download/v1.0.0/tightship-claude-code.zip",
        "sha256": "<64 lowercase hex digits>"
      },
      "description": "Skills, a command per step of the development loop, and a read-only verifier"
    }
  ]
}
```

| Key | From |
| --- | --- |
| `name`, `owner`, `description`, `allowCrossMarketplaceDependenciesOn` | As in the local catalog of epic #2 |
| `source.url` | `<repository>/releases/download/<tag>/tightship-claude-code.zip` |
| `source.sha256` | The SHA-256 of the zip just written |

Output:

```text
Packed dist/tightship-claude-code.zip (39 files) and dist/marketplace.json for v1.0.0.
sha256: <64 lowercase hex digits>
```

#### `plugin.json`

Gains `"license": "MIT"`, after `repository`, from a new `license` key of `[plugin]` in `harness.toml`. Its
`version` is the workspace's, `1.0.0` from the release.

#### The hook lines that run the binary

Each memory hook line of epic #5 runs the launcher through `sh`:

```text
[ "$CLAUDE_PLUGIN_OPTION_MEMORY" = false ] || TIGHTSHIP_VAULT="$CLAUDE_PLUGIN_OPTION_VAULT_FOLDER" TIGHTSHIP_ENV_FILE="$CLAUDE_ENV_FILE" sh "${CLAUDE_PLUGIN_ROOT}/bin/tightship" hook start
```

#### `bin/tightship`, the launcher

| `uname -s` | `uname -m` | Binary |
| --- | --- | --- |
| `Darwin` | any | `macos/tightship` |
| `Linux` | `x86_64` | `linux-x64/tightship` |
| `Linux` | `aarch64` or `arm64` | `linux-arm64/tightship` |
| Starts with `MINGW`, `MSYS`, or `CYGWIN` | `x86_64`, `aarch64`, or `arm64` | `windows-x64/tightship.exe`; Windows on arm64 runs it emulated |

When the binary it picks is not executable, the launcher runs `chmod +x` on that binary and on itself, then
runs the binary.

#### The install guide, `docs/install/claude-code/`

| File | Subject | Holds |
| --- | --- | --- |
| `README.md` | Index | The prerequisites: Claude Code 2.1.295 or later, checked with `claude --version`; Git for Windows on Windows; Git when memory is on. Then one line per page |
| `install.md` | Installation | The four `claude plugin marketplace add` lines, the companions' catalogs first, then `claude plugin install tightship@tightship` |
| `companions.md` | The companions | What ponytail, i-have-adhd, and humanizer each bring, that the install adds them, and that uninstall removes only those it added |
| `options.md` | The options | `contract`, `memory`, `status_line`, and `vault_folder`: what each turns on, the dialog, `--config <name>=<value>`, and `claude plugin configure` |
| `update.md` | Update | `claude plugin marketplace update tightship`, then `claude plugin update tightship@tightship`; the options keep their values |
| `uninstall.md` | Uninstall | Turn `status_line` off with `claude plugin configure` and open one session, which puts the previous status line back ([FR-INS-06.1](../../requirements/functional/install.md#fr-ins-06)); then `claude plugin uninstall tightship@tightship --prune -y`, then `claude plugin marketplace remove` for each catalog; the vault stays |
| `migration.md` | Migration from the pre-plugin install | Rule 9's steps |
| `without-the-contract.md` | The contract off | Rule 10's table |

Each command in the guide is the literal one of epics #2, #3, #5, #6, and #7.

#### The release checklist, `docs/release/`

`README.md` holds the steps of a release, rule 11. `checklist.md` holds one row per acceptance criterion of
every functional requirement of M1, with how it is checked: the test of `cargo test --workspace` that covers
it, or the manual steps and the result to observe. It has one result column per system: macOS, Linux,
Windows.

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | Pushes a tag, promotes a release, sets the secret `IDENTITY_TERMS` |
| Release pipeline | `GITHUB_TOKEN` with `contents: write` | Creates the pre-release and uploads its two assets |
| Adopter | None from this project | Downloads public release assets |

---

## Business Rules

### 1. Authorization

- Only someone with write access pushes a tag or edits a release. GitHub checks it at the push and at the edit.
- The release pipeline's token writes to the repository's releases only.
- A pull request from a fork gets no `IDENTITY_TERMS`, so its build stops at the identity scan with epic #1's
  error. The project takes no outside contributions in M1.

### 2. The gates hold on the three systems

- **When:** a pull request into `develop` or `main` is opened or gets a push.
- `ci.yml` runs every gate of `AGENTS.md` on Linux, macOS, and Windows. A gate that exits non-zero fails its
  job, and the pull request shows a failed check.
- The merge waits for the three green jobs, by convention, as the branch rules of `AGENTS.md` are.

### 3. The pipeline builds from prebuilt binaries

- **When:** `cargo xtask build --binaries <folder>`.
- A missing platform file fails with `BINARY_MISSING`, before anything is written. Every other check and
  error of the build stays as epic #1 set it.

### 4. A package is exactly the tagged version

- **When:** `cargo xtask package <tag>`.
- A tag other than `v<version>` fails with `TAG_MISMATCH`. A `dist/claude-code/` without the four platform
  folders fails with `PLATFORM_MISSING`. Neither writes a file.
- The zip keeps the modes of the response, so a system without file modes packs the same zip.

### 5. A tag publishes a pre-release, never the latest release

- **When:** a tag `v*` is pushed.
- The four builds run in parallel; the pack job runs only if all four succeed.
- A failed build, a failed package, or a failed `gh release create` leaves no release. The maintainer deletes
  the tag, fixes, and pushes it again.
- The pre-release never becomes the latest release by itself.

### 6. The plugin runs even when an install drops the executable bit

- **When:** a hook runs the launcher.
- The hook runs it through `sh`, so the launcher needs no mode. The launcher restores the mode of the binary
  and its own, so the agent's bare `tightship` command works after the first hook.
- On a system the launcher does not know, it exits 1 with epic #5's message, and the session goes on by
  NFR-REL-01.

### 7. The minimum Claude Code version

- The guide and the root `README.md` state "Claude Code 2.1.295 or later".
- The release checklist runs at exactly 2.1.295, installed with `claude install 2.1.295`, with auto-update
  off through `DISABLE_AUTOUPDATER=1`.
- A version below it is not supported. Nothing in the plugin checks the version, because a plugin manifest
  has no field for it.

### 8. The guide covers the six subjects

- One page per subject of FR-INS-08: installation, the companions, the options, update, uninstall, and
  migration. The page for the contract off is the seventh, for FR-CTR-05.
- Every command on a page is copied from the spec that built it, and the guide names no command that does
  not exist in the release.

### 9. The migration removes the baseline's workflow pieces and keeps everything else

The steps of `migration.md`, in order:

1. Back up the harness's user directory.
2. Note the vault's folder, `~/ai-memory` in the baseline, or the one the adopter moved it to.
3. Delete what the baseline merged: the 13 skill folders the plugin ships, the 9 commands of `commands/`,
   `agents/adversarial-verifier.md`, `hooks/vault.py`, `hooks/test_vault.py`, `hooks/generate-settings.py`,
   `hooks/test_generate_settings.py`, `vault-template/`, `setup.py`, `settings.base.json`,
   `export-workflow.py`, `SETUP.md`, `statusline-command.ts`, `package.json`, `bun.lock`, and `tsconfig.json`.
4. Delete from `CLAUDE.md` every line the baseline wrote, and the file when nothing is left.
5. Delete from `settings.json` the three hooks that run `hooks/vault.py` and the `statusLine` that runs
   `statusline-command.ts`. Every other key stays as it is.
6. Install the plugin with `--config vault_folder=<the folder of step 2>`, and turn on the status line by
   epic #7's consent when the adopter wants it.
7. Check it: `/plugin` lists `tightship@tightship` with no error, `/hooks` shows no hook running
   `vault.py`, and the session start lists the vault's pinned notes once.

A skill, a command, or a setting the baseline did not merge, such as a personal skill, is never touched. The
vault's notes are never touched.

### 10. The guide says what changes without the contract

- `without-the-contract.md` holds one row per skill or command whose text refers to a rule of the contract,
  with the rule and what the piece does when the contract is off.
- A piece belongs in the table when its canonical text names the contract or one of its sections, as found by
  reading every file of `content/skills/` and `content/commands/` when task 3 starts. On 2026-10-15 these are
  `audit`, `bootstrap`, `closeout`, `epic`, `testing-contract`, `security-audit`, `frontend-architecture`, and
  `performance-analysis`; the pieces of epics #6 and #7 are read with the rest.

### 11. A release is promoted only after its checklist passes on the three systems

- The maintainer runs `docs/release/checklist.md` on macOS, Linux, and Windows, each at Claude Code 2.1.295,
  installing from `https://github.com/ATNexusLab/tightship/releases/download/v<version>/marketplace.json`.
- Every row passes on every system, or the release is not promoted. A failing row is fixed in a new version,
  which goes through the whole checklist again.
- On macOS, the checklist checks that the hook binary runs from the archive Claude Code downloaded, with only
  the linker's ad-hoc signature. If it does not, the release is not promoted, and signing becomes a change
  request.
- Promoting `v1.0.0` releases M1: the milestone closes, the roadmap marks M1 released, and M2 may start by
  BR-01.

### 12. Persistence and audit

- **Files changed:** `dist/` by the two xtask commands; on GitHub, one release with two assets per tag.
- **Audit:** GitHub keeps every workflow run with its logs, and every release with its author and date.
- **Events/integrations fired:** the release pipeline calls GitHub's release API through `gh`. Nothing else
  uses the network.

---

## Errors

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `USAGE` | `1` | Any other argument list to `cargo xtask` | "`usage: cargo xtask build [--binaries <folder>] \| package <tag> \| scan`" |
| `BINARY_MISSING` | `1` | Rule 3 | "`<folder>/<platform>/<file> is missing; --binaries needs macos, linux-x64, linux-arm64, and windows-x64`" |
| `TAG_MISMATCH` | `1` | Rule 4 | "`the tag <tag> does not match the version <version>; tag v<version>`" |
| `PLATFORM_MISSING` | `1` | Rule 4 | "`dist/claude-code/bin/ has no <platform>; run cargo xtask build --binaries <folder> first`" |

Each message is printed on stderr after the prefix `error:`, as every xtask error is.

## Side Effects

- **Persistence:** `build --binaries` replaces `dist/claude-code/` whole, as the host build does. `package`
  replaces `dist/tightship-claude-code.zip` and `dist/marketplace.json`.
- **Concurrency:** two tags pushed together run two pipelines, each on its own tag; `gh release create` fails
  for a tag that already has a release, and that pipeline leaves the existing release untouched.
- **Transaction:** `package` writes both files into `dist/.package.tmp/` and renames them into place only when
  both are complete.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — The gates pass on the three systems (NFR-FLEX-05.1, happy path)

```gherkin
Given a pull request into develop whose gates pass on the maintainer's machine
When GitHub runs ci.yml
Then the jobs on ubuntu-latest, macos-latest, and windows-latest each succeed
And each job ran cargo test --workspace, cargo xtask build, and cargo xtask scan
```

### Scenario 2 — A gate fails on one system (exception)

```gherkin
Given a pull request with a test that fails only on Windows
When GitHub runs ci.yml
Then the windows-latest job fails at the step cargo test --workspace
And the pull request shows a failed check
```

### Scenario 3 — Build with the four binaries (happy path)

```gherkin
Given binaries/ holds macos/tightship, linux-x64/tightship, linux-arm64/tightship, and windows-x64/tightship.exe
When the pipeline runs `cargo xtask build --binaries binaries`
Then it exits 0 and reports the host build's file count plus 3
And dist/claude-code/bin/ holds the four platform folders and the launcher
And no cargo build ran
```

### Scenario 4 — A binary is missing (exception)

```gherkin
Given binaries/ has no linux-arm64/tightship
When the pipeline runs `cargo xtask build --binaries binaries`
Then it exits 1 with "error: binaries/linux-arm64/tightship is missing; --binaries needs macos, linux-x64,
  linux-arm64, and windows-x64"
And dist/claude-code/ does not exist
```

### Scenario 5 — Package a release (happy path)

```gherkin
Given dist/claude-code/ holds the four platform folders and Cargo.toml's version is 1.0.0
When the pipeline runs `cargo xtask package v1.0.0`
Then it exits 0 and prints the sha256 of dist/tightship-claude-code.zip
And the zip holds every file of dist/claude-code/ but .claude-plugin/marketplace.json
And bin/tightship and the four binaries have mode 0755 in the zip, and every other file 0644
And dist/marketplace.json names the URL of v1.0.0's zip and the same sha256
And `claude plugin validate dist/marketplace.json` reports no error
```

### Scenario 6 — A tag that is not the version (exception)

```gherkin
Given Cargo.toml's version is 1.0.0
When the pipeline runs `cargo xtask package v1.0.1`
Then it exits 1 with "error: the tag v1.0.1 does not match the version 1.0.0; tag v1.0.0"
And dist/ holds no zip and no marketplace.json
```

### Scenario 7 — Package a host build (exception)

```gherkin
Given dist/claude-code/ came from `cargo xtask build` on Linux x64
When the maintainer runs `cargo xtask package v1.0.0`
Then it exits 1 with "error: dist/claude-code/bin/ has no macos; run cargo xtask build --binaries <folder> first"
```

### Scenario 8 — Usage (exception)

```gherkin
When the maintainer runs `cargo xtask package`
Then it exits 1 with "error: usage: cargo xtask build [--binaries <folder>] | package <tag> | scan"
```

### Scenario 9 — A tag publishes a pre-release (happy path)

```gherkin
Given main's Cargo.toml says 1.0.0
When the maintainer pushes the tag v1.0.0
Then release.yml builds the four targets and packs them
And GitHub shows the pre-release v1.0.0 with tightship-claude-code.zip and marketplace.json
And the latest release, if any, is unchanged
```

### Scenario 10 — A failed build publishes nothing (exception)

```gherkin
Given the build job for aarch64-unknown-linux-musl fails
When the pipeline runs for the tag v1.0.0
Then the pack job does not run
And no release v1.0.0 exists
```

### Scenario 11 — The launcher picks the machine's binary (NFR-FLEX-05)

```gherkin
Given a plugin folder with the four platform binaries
When the launcher runs under sh with uname reporting each row of the launcher's table
Then it runs the binary that row names
And with "MINGW64_NT-10.0 aarch64" it runs windows-x64/tightship.exe
And with "SunOS i86pc" it exits 1 with "error: tightship has no binary for SunOS i86pc"
```

### Scenario 12 — An install that dropped the executable bit (alternative)

```gherkin
Given an installed plugin whose bin/tightship and binaries have mode 0644
When a session starts with memory on
Then the hook runs, and the session start lists the vault's pinned notes
And bin/tightship and the machine's binary now have mode 0755
And the agent's `tightship vault path` prints the vault folder
```

### Scenario 13 — Install from the release on each system (FR-INS-01.1, NFR-FLEX-01.1, NFR-FLEX-05.1)

```gherkin
Given a machine on macOS, Linux, or Windows with Claude Code 2.1.295 and no plugin
And the companions' three catalogs added
When the adopter adds the catalog of the pre-release v1.0.0 and runs `claude plugin install tightship@tightship`
Then `claude plugin list` shows tightship@tightship at 1.0.0 and the three companions
And a new session starts with the contract and, with a vault folder set, the vault's pinned notes
```

### Scenario 14 — The macOS binary runs from the downloaded archive

```gherkin
Given the plugin installed on macOS from the pre-release's archive
When a session starts with memory on
Then the hook binary runs, with no Gatekeeper prompt and no kill
And `codesign -dv` on bin/macos/tightship reports an ad-hoc signature
```

### Scenario 15 — Migrate from the pre-plugin install (FR-INS-07.1, FR-INS-07.2)

```gherkin
Given a machine with the baseline installed and a vault of notes
And a copy of settings.json without the keys step 5 deletes, and the vault's git HEAD
When the adopter follows migration.md
Then `/plugin` lists tightship@tightship with no error
And the session's skills and commands list each name once
And `/hooks` shows no hook that runs vault.py
And settings.json equals the copy, plus the plugin's own keys
And the vault's git HEAD is unchanged and `git status` in it is clean
And every personal skill is still in the user directory
```

### Scenario 16 — The guide covers the six subjects (FR-INS-08.1)

```gherkin
Given the Claude Code install guide
When the adopter opens docs/install/claude-code/README.md
Then it links a page for installation, the companions, the options, update, uninstall, and migration
And it states "Claude Code 2.1.295 or later"
```

### Scenario 17 — The guide says what the contract changes (FR-CTR-05.1)

```gherkin
Given the release's content/skills/ and content/commands/
When the reviewer lists every piece whose text names the contract or one of its sections
Then without-the-contract.md has one row per piece listed, with the rule and what the piece does without it
```

### Scenario 18 — The checklist passes and the release is promoted (NFR-FLEX-01.1, NFR-FLEX-05.1, BR-01)

```gherkin
Given every row of checklist.md passed on macOS, Linux, and Windows at Claude Code 2.1.295
When the maintainer runs `gh release edit v1.0.0 --prerelease=false --latest`
Then the guide's catalog URL serves v1.0.0's marketplace.json
And the milestone M1 is closed and the roadmap marks it released
```

### Scenario 19 — A checklist row fails (exception)

```gherkin
Given a row of checklist.md fails on Windows for the pre-release v1.0.0
Then v1.0.0 is not promoted
And the fix ships as v1.0.1, which runs the whole checklist on the three systems again
```

### Scenario 20 — The license (happy path)

```gherkin
Given the repository root
Then LICENSE holds the MIT license
And the built plugin.json holds "license": "MIT"
```

### Scenario 21 — A pull request from a fork (denied authorization)

```gherkin
Given a pull request from a fork, which GitHub runs without the secret IDENTITY_TERMS
When GitHub runs ci.yml
Then each job fails at cargo xtask build with "error: .identity-terms is missing; create it with one line per
  name, email address, user name, and handle of the maintainer"
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md), marked M1, #8:

- **Building blocks:**
  - Release pipeline: built at a tag, publishing a pre-release with the zip and the catalog.
  - A new block, the gate pipeline, `.github/workflows/ci.yml`: every gate on the three systems at each pull
    request.
  - Build: the `--binaries` mode and the `package` command.
  - `tightship` binary: the launcher's Windows arm64 row and its restored executable bit.
- **Deployment:** the catalog is a release asset at a fixed URL; the adopter's prerequisite Claude Code 2.1.295;
  the hook lines run the launcher through `sh`; the release is promoted after the checklist.
- **Containers diagram:** the release pipeline drawn solid, the gate pipeline added.
- **Deployment diagram:** GitHub holds the pre-release and the latest release, each with its catalog.

## Data model impact

N/A — this epic stores no data. The options and the vault keep the model epics #3 and #5 gave them.

## Out of Scope

- **Signing and notarizing the macOS binary with a Developer ID:** only if scenario 14 fails, by a change
  request.
- **Pull requests from forks:** they get no secret, so their build stops at the identity scan. No outside
  contributor is expected in M1.
- **Windows on arm64 hardware:** the launcher runs the x64 binary there, as scenario 11 checks; no runner
  tests the emulation.
- **Claude.ai and Cowork:** they refuse a plugin with a top-level `bin/`, and neither is a harness of BR-01.
- **A check of the Claude Code version at run time:** a plugin manifest has no field for it.
- **A migration program:** the adopter follows the guide's steps, by decision 4.
- **Install guides for Cursor, Antigravity CLI, and Codex, and their minimum versions:** M2 to M4. OQ-04 stays
  open for them.
- **Listing in a vendor's marketplace:** out of the SRS's scope.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on | Issue |
| --- | --- | --- | --- | --- | --- |
| 1 | Run the gates on macOS, Linux, and Windows at every pull request | `.github/workflows/ci.yml` (new), `tightship/launcher.sh` (Windows arm64, restored mode), `xtask/src/build/claude_code.rs` (hook lines through `sh`), `xtask/tests/build.rs`, `tightship/tests/launcher.rs` (new), `AGENTS.md` (the CI, the secret), plus each fix the three systems reveal in code of epics #1 to #5, about 7 files before those fixes | Scenarios 1, 2, 11 by the CI runs and tests; 21 by the build test without `.identity-terms`; 12 in an isolated Claude Code configuration | — | #36 |
| 2 | Publish a pre-release from a tag | `xtask/src/main.rs` (usage), `xtask/src/build.rs` and `xtask/src/build/binary.rs` (`--binaries`), `xtask/src/package.rs` (new), `xtask/Cargo.toml` and `Cargo.lock` (`zip`, `sha2`), `xtask/tests/build.rs`, `xtask/tests/package.rs` (new), `.github/workflows/release.yml` (new), `LICENSE` (new), `harnesses/claude-code/harness.toml` and `xtask/src/build/claude_code.rs` (`license`), `AGENTS.md` (commands), about 13 files | Scenarios 3 to 8 and 20 by tests; 9 and 10 by a tag pushed on a fork of the repository | 1 | #37 |
| 3 | Write the Claude Code install guide | `docs/install/claude-code/` (8 new pages), `README.md` at the root (new), `docs/README.md` (a row), about 10 files; starts once epics #6 and #7 are merged into `develop` | Scenarios 15 on the maintainer's machine, 16 and 17 read at review | 2 | #38 |
| 4 | Run the release checklist on the three systems | `docs/release/README.md` and `docs/release/checklist.md` (new), `Cargo.toml` (`1.0.0`), `docs/roadmap/` (M1 released), about 5 files; the tag and the promotion happen only at the maintainer's word | Scenarios 13, 14, 18, and 19 on macOS, Linux, and Windows at Claude Code 2.1.295 | 3 | #39 |
