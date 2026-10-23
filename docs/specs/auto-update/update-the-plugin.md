# Keep the plugin updated in every harness

> **Status:** closed
> **Profile:** API
> **Module:** `tightship/`, `xtask/`, `harnesses/`, `installer/`
> **Epic:** none, delivered on the `fix/open-issues` branch by the maintainer's decision
> **Requirements:** FR-INS-10, FR-INS-05
> **Sprint:** 2

Each release reaches every harness without a step from the adopter, and the next session says so. In Claude
Code the plugin turns the harness's own auto-update on. In Cursor and Antigravity CLI the `tightship` binary
downloads the release and puts it in place, from a process the session-start hook starts. The installer also
learns to update a Claude Code plugin it already installed, which a rerun never did.

Terms, as used here:

- **Option**: a part of the plugin the adopter turns on or off ([FR-INS-03](../../requirements/functional/install.md#fr-ins-03)).
  This spec adds `auto_update`, read by rule 4 of the [epic #3 spec](../options/choose-the-options.md): off
  only when the value is exactly `false`.
- **Plugin folder**: the folder the harness loads the plugin from, `TIGHTSHIP_ROOT`.
- **Data folder**: the folder the harness keeps across updates, `TIGHTSHIP_DATA`.
- **Promoted release**: a GitHub release the maintainer moved from pre-release to latest
  ([release checklist](../../release/checklist.md)).

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-23, and recorded in
[ADR 0003](../../decisions/0003-the-plugin-updates-itself-from-github-releases.md):

1. **The binary updates Cursor and Antigravity CLI.** A detached `tightship update` downloads with `curl` on
   macOS and Linux and with `powershell` on Windows, and adds no crate.
2. **Every release publishes `latest.json`**, and only a promoted release with a greater version is applied.
3. **The update is applied, then announced.** The next session tells the adopter, once, and asks nothing.
4. **`auto_update` is on by default**, in the three harnesses, and the installer asks for it.
5. **Claude Code updates itself**, once the plugin sets `autoUpdate` on its marketplace entry.
6. **A Claude Code install that already exists is updated by the installer.** The rerun refreshes the
   marketplaces, updates the plugin, and saves the options the adopter chose.
7. **Companion plugins are not updated by the binary.** They are third-party code with no published checksum.
   A rerun of the installer updates them.

## Acceptance Criteria

### Contract

| Command | Who runs it | Idempotent | Effect |
| --- | --- | --- | --- |
| `tightship hook update` | The session-start hook of Cursor and Antigravity CLI | Yes | Prints the pending notice, then starts `tightship update` when a check is due |
| `tightship update` | The hook, detached, or the adopter | Yes | Checks the latest release and applies it when it is newer |
| `tightship hook marketplace-auto-update` | The session-start hook of Claude Code, on `startup` | Yes | Writes `autoUpdate` on the plugin's entry in `extraKnownMarketplaces` |
| `cargo xtask package <tag>` | The release workflow | Yes | Also writes `dist/latest.json` |
| `install.sh --auto-update <bool>` · `install.ps1 -AutoUpdate <bool>` | The adopter | Yes | Sets the option in the chosen harnesses |

### Request

#### Environment

| Variable | Read by | Required | Source |
| --- | --- | --- | --- |
| `TIGHTSHIP_DATA` | `hook update`, `update` | Yes | The launcher |
| `TIGHTSHIP_ROOT` | `update` | Yes | The launcher |
| `TIGHTSHIP_MANIFEST` | `update`, `hook marketplace-auto-update` | Yes | The launcher (Cursor, Antigravity CLI) or the hook's command (Claude Code) |
| `TIGHTSHIP_HARNESS` | `update` | Yes | The launcher: `cursor` or `antigravity-cli` |
| `TIGHTSHIP_SETTINGS` | `hook marketplace-auto-update` | Yes | The hook's command: `${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json` |
| `TIGHTSHIP_AUTO_UPDATE` | Every command | No | Claude Code's `CLAUDE_PLUGIN_OPTION_AUTO_UPDATE`. Empty or absent falls back to the options file |

#### The plugin's manifest

`TIGHTSHIP_MANIFEST` names the manifest the build writes: `.cursor-plugin/plugin.json`, `plugin.json`, or
`.claude-plugin/plugin.json`. The commands read three strings from it: `name`, `version` (`X.Y.Z`, three
unsigned integers), and `repository` (a URL).

#### `latest.json`, a release asset

Written by `cargo xtask package <tag>` and uploaded with the release:

```json
{
  "version": "1.2.0",
  "tag": "v1.2.0",
  "assets": {
    "claude-code": { "archive": "tightship-claude-code.zip", "sha256": "<64 hex digits>" },
    "cursor": { "archive": "tightship-cursor.zip", "sha256": "<64 hex digits>" },
    "antigravity-cli": { "archive": "tightship-antigravity-cli.zip", "sha256": "<64 hex digits>" }
  }
}
```

### Response

#### `<TIGHTSHIP_DATA>/update.json`

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `checked_at` | string, RFC 3339 UTC | No | When `update` last started a check |
| `notice` | object | No | `{"kind": "updated" \| "refused", "from": "X.Y.Z", "to": "X.Y.Z"}`, cleared once printed |
| `error` | string | No | The message of the last failed check, cleared by the next one that succeeds |

A file that is not valid JSON, or breaks this table, reads as empty and is overwritten.

#### What `hook update` prints

With `notice.kind` = `updated`:

> "# tightship update\n\ntightship was updated from \<from\> to \<to\> since the last session. Tell the
> user, word for word: \"tightship was updated from \<from\> to \<to\>.\"\n"

With `notice.kind` = `refused`:

> "# tightship update\n\nThe tightship \<to\> release did not match the sha256 it published, so it was not
> installed, and \<from\> stays. Tell the user, word for word: \"tightship refused the \<to\> update: its
> download did not match the sha256 the release published.\"\n"

Nothing otherwise. The text goes through the hook output template, as every hook's does.

#### What `update` prints

| When | stdout |
| --- | --- |
| The latest release is not newer | "tightship \<version\> is the latest release." |
| The release was applied | "Updated tightship from \<from\> to \<to\>. The next session loads it." |
| Another update holds the lock | "Another tightship update is running." |

All three exit 0.

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Adopter | Their own user account | The commands write only in the plugin folder, the data folder, and the user settings of Claude Code |

---

## Business Rules

### 1. Authorization

The commands run as the adopter, from the adopter's harness, on the adopter's files. The repository they
download from is the `repository` of the plugin's own manifest, which only someone who can already change the
plugin can edit.

### 2. The option decides

`auto_update` off (exactly `false`, in `TIGHTSHIP_AUTO_UPDATE` or in the options file) means:

- `hook update` prints a pending notice but starts nothing;
- `update` prints nothing and exits 0;
- `hook marketplace-auto-update` writes `"autoUpdate": false`.

Absent or anything else is on.

### 3. One check a day

`hook update` starts `update` only when `checked_at` is absent or more than 86,400 seconds old. `update`
writes `checked_at` before it downloads, so a check that fails also waits a day.

### 4. The hook never waits

`hook update` starts the binary it runs from as `update`, with stdin, stdout, and stderr closed, in a process
group of its own (`DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP` on Windows), and returns without waiting.

### 5. One update at a time

`update` creates `<TIGHTSHIP_DATA>/update.lock` with `create_new` and removes it when it ends. A lock older than
3,600 seconds is removed and taken. A lock younger than that ends `update` with "Another tightship update is
running.".

### 6. Only a newer, promoted release

`update` downloads `<repository>/releases/latest/download/latest.json`. When its `version` is not greater
than the manifest's, compared as three integers, it changes nothing. `releases/latest` never points at a
pre-release.

### 7. The checksum decides

`update` downloads `<repository>/releases/download/<tag>/<archive>` for `TIGHTSHIP_HARNESS` into
`<TIGHTSHIP_DATA>/update/`, and computes its sha256. A sha256 other than the one in `latest.json` changes
nothing in the plugin folder, writes the `refused` notice, and fails with `UPDATE_INTEGRITY`.

### 8. The tools

| System | Download | Checksum | Extract |
| --- | --- | --- | --- |
| macOS, Linux | `curl --fail --location --silent --show-error --max-time 60 --proto =https,file` | `sha256sum`, or `shasum -a 256` when it is absent | `unzip -q` |
| Windows | `powershell -NoProfile -NonInteractive`, TLS 1.2, `Invoke-RestMethod -TimeoutSec 60 -OutFile` | `Get-FileHash -Algorithm SHA256` | `Expand-Archive -Force` |

Each value reaches PowerShell through an environment variable, never inside the script's text.

### 9. Only plain files

The archive is extracted to `<TIGHTSHIP_ROOT>/.update/`, emptied first. Any entry there that is not a regular
file or a folder, such as a symbolic link, fails with `UPDATE_ARCHIVE_UNSAFE` before a file moves.

### 10. The swap rolls back

The release's files replace the plugin folder's:

1. Each file of the plugin folder, outside `.update/` and other than `*.tightship-old`, is renamed to
   `<file>.tightship-old`. A file inside a top-level folder whose name starts with a dot, such as `.git/` or
   a marketplace's catalog, is renamed only when the release carries a file at its path; otherwise it stays.
2. Each file of `.update/` is renamed to its place, its folders created first.
3. A rename that fails puts back, in reverse order, every rename done before it, and fails with
   `UPDATE_APPLY_FAILED`.
4. Then every `*.tightship-old` that can be deleted is deleted, `.update/` is deleted, and so is every folder
   left empty.

A running binary can be renamed on every system, so the binary in use moves aside like any file. The next
`update` deletes a `*.tightship-old` that could not be deleted.

A `hooks.windows.json` at the root of the release takes the place of `hooks.json` on Windows and is left out
elsewhere, as the installer does.

### 11. The notice

A successful swap writes the `updated` notice, and rule 7 writes the `refused` one. `hook update` prints the
notice and clears it, so it shows once. A failure other than rule 7 only writes `error`.

### 12. Claude Code's marketplace entry

`hook marketplace-auto-update` reads the user settings. When `extraKnownMarketplaces.<name>` exists, it sets its
`autoUpdate` to the option. Otherwise it creates the entry, with a `source` taken from `repository`:
`{"source": "github", "repo": "<owner>/<repo>"}` for a `https://github.com/` URL, and
`{"source": "git", "url": "<repository>"}` for any other. The file is written only when a value changed,
atomically and keeping its permissions, as the status line writes it.

### 13. A rerun of the installer updates Claude Code

When `claude plugin list --json` lists `tightship@tightship`, the installer runs
`claude plugin marketplace update` for the plugin's marketplace and each companion's, then
`claude plugin update tightship@tightship`, then `claude plugin configure tightship@tightship --values-stdin`
with the options chosen. Otherwise it installs, as before, with `--config auto_update=<bool>` added. The
default of each option on a rerun is its current value, read from `claude plugin configure tightship@tightship
--json`.

### 14. A failure never blocks the session

`hook update` and `hook marketplace-auto-update` exit 1 with their message on a failure, which no harness
treats as a blocking code ([NFR-REL-01](../../requirements/non-functional/reliability.md#nfr-rel-01)). The
detached `update` has no output to show; its failure stays in `error`.

### 15. Persistence and audit

- **Files changed:** the plugin folder (rule 10); `update.json`, `update.lock`, and `update/` in the data
  folder; `extraKnownMarketplaces.<name>` in Claude Code's user settings.
- **Audit:** `update.json` keeps the last check's time and its failure.
- **Events/integrations fired:** two downloads from the plugin's repository on GitHub, at most once a day.

---

## Errors

Every message goes to stderr, prefixed by `error:` and a space, and exits 1.

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `MANIFEST_INVALID` | `1` | The manifest cannot be read, or lacks a string `name`, an `X.Y.Z` `version`, or a `repository` | "`` <path> is not a plugin manifest with a name, a version, and a repository ``" |
| `UPDATE_TOOL_MISSING` | `1` | Rule 8: a tool is not on `PATH` | "`` <tool> is not on PATH, and the update needs it ``" |
| `UPDATE_DOWNLOAD_FAILED` | `1` | Rule 8: the download tool exits non-zero | "`` cannot download <url>: <the tool's message> ``" |
| `UPDATE_RELEASE_INVALID` | `1` | `latest.json` breaks its table | "`` <url> is not a release description tightship can read ``" |
| `UPDATE_NO_ARCHIVE` | `1` | `assets` has no entry for `TIGHTSHIP_HARNESS` | "`` the release <tag> has no archive for <harness> ``" |
| `UPDATE_CHECKSUM_FAILED` | `1` | Rule 8: the checksum tool exits non-zero or prints no sha256 | "`` cannot compute the sha256 of <archive>: <the tool's message> ``" |
| `UPDATE_INTEGRITY` | `1` | Rule 7 | "`` <archive> has sha256 <actual>, and the release <tag> published <expected>; nothing changed ``" |
| `UPDATE_EXTRACT_FAILED` | `1` | Rule 8: the extract tool exits non-zero | "`` cannot extract <archive>: <the tool's message> ``" |
| `UPDATE_ARCHIVE_UNSAFE` | `1` | Rule 9 | "`` <archive> holds <entry>, which is not a plain file; nothing changed ``" |
| `UPDATE_APPLY_FAILED` | `1` | Rule 10 | "`` cannot replace <path>: <the system's message>; every file was put back ``" |
| `UPDATE_STATE_UNWRITABLE` | `1` | `update.json`, the lock, or `update/` cannot be written | "`` cannot write <path>: <the system's message> ``" |
| `UPDATE_START_FAILED` | `1` | Rule 4: the process cannot start | "`` cannot start the update: <the system's message> ``" |

The variables of the environment table fail with the binary's existing `VARIABLE_NOT_SET`, and the user
settings with the status line's `SETTINGS_UNREADABLE`, `SETTINGS_INVALID`, and `SETTINGS_UNWRITABLE`.

## Side Effects

- **Persistence:** a release replaces the plugin folder's files and never touches the data folder's options
  ([FR-INS-05](../../requirements/functional/install.md#fr-ins-05)).
- **Concurrency:** the lock of rule 5. A harness that reads a file while it is swapped finds either the old one
  or the new one, since each step is a rename.
- **Transaction:** the swap of rule 10 is all or nothing, through its rollback.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — A newer release is applied (FR-INS-10.1, happy path)

```gherkin
Given a plugin folder at 1.1.1 whose options file sets contract to false
And a release 1.2.0 whose latest.json matches its archive
When `tightship update` runs
Then the plugin folder holds the 1.2.0 files, and a file 1.2.0 dropped is gone
And the options file still sets contract to false
And it prints "Updated tightship from 1.1.1 to 1.2.0. The next session loads it."
```

### Scenario 2 — The next session announces it (FR-INS-10.2)

```gherkin
Given update.json holds the notice updated from 1.1.1 to 1.2.0
When `tightship hook update` runs twice
Then the first run prints the updated notice
And the second run prints nothing
```

### Scenario 3 — The option is off (FR-INS-10.3)

```gherkin
Given TIGHTSHIP_AUTO_UPDATE is false and a release 1.2.0 exists
When `tightship update` runs
Then the plugin folder is unchanged and nothing is printed
```

### Scenario 4 — A mismatched checksum is refused (FR-INS-10.4, exception)

```gherkin
Given a release 1.2.0 whose latest.json publishes another sha256
When `tightship update` runs
Then it fails with UPDATE_INTEGRITY and the plugin folder is unchanged
And the next `tightship hook update` prints the refused notice once
```

### Scenario 5 — The latest release is not newer (alternative)

```gherkin
Given the plugin folder is at 1.2.0 and the latest release is 1.2.0
When `tightship update` runs
Then the plugin folder is unchanged
And it prints "tightship 1.2.0 is the latest release."
```

### Scenario 6 — Claude Code's marketplace entry (FR-INS-10.1, FR-INS-10.3)

```gherkin
Given user settings with other keys and a tightship entry in extraKnownMarketplaces without autoUpdate
When `tightship hook marketplace-auto-update` runs with the option on, then with it off
Then autoUpdate is true, then false
And every other key keeps its value and its order
```

### Scenario 7 — A rerun of the installer updates Claude Code (FR-INS-10.1, regression)

```gherkin
Given `claude plugin list --json` lists tightship@tightship
When the adopter runs the installer for Claude Code
Then it runs `claude plugin marketplace update tightship` and `claude plugin update tightship@tightship`
And it saves the chosen options with `claude plugin configure tightship@tightship --values-stdin`
And it never runs `claude plugin install`
```

### Scenario 8 — The release carries latest.json (FR-INS-10.1)

```gherkin
Given a built dist/ at version 1.2.0
When `cargo xtask package v1.2.0` runs
Then dist/latest.json holds version 1.2.0, tag v1.2.0, and each archive with the sha256 the package printed
```

### Scenario 9 — The installer asks for the option (FR-INS-03)

```gherkin
Given an interactive terminal
When the adopter presses Enter on every question
Then "Keep tightship updated automatically?" is asked after the status line
And auto_update is true in the options of each chosen harness
```

---

## Architecture impact

- **Building blocks:** the [tightship binary](../../architecture/building-blocks/tightship-binary.md) gains the
  `update/` domain and two hooks; the [release pipeline](../../architecture/building-blocks/release-pipeline.md)
  publishes `latest.json`; the plugins of [Cursor](../../architecture/building-blocks/cursor-plugin.md),
  [Antigravity CLI](../../architecture/building-blocks/antigravity-cli-plugin.md), and
  [Claude Code](../../architecture/building-blocks/claude-code-plugin.md) gain one session-start hook each.
- **Concepts:** [security](../../architecture/concepts/security.md) adds the plugin's repository to the reach,
  and [options](../../architecture/concepts/options.md) adds `auto_update`.
- **Decisions:** [ADR 0003](../../decisions/0003-the-plugin-updates-itself-from-github-releases.md) amends ADR
  0001's Reach and ADR 0002's processes.

## Data model impact

[Options](../../data-model/options.md) adds `auto_update`, and the data folder adds `update.json` and
`update.lock`, with the tables above.

## Out of Scope

- **Signing releases.** The checksum shares the release's trust root; signing is a decision of its own.
- **The official Cursor and Antigravity marketplaces.** Applying to them is an outward action the maintainer
  takes separately.
- **Updating companion plugins from the binary.** They publish no checksum; the installer updates them.
- **A pre-release channel.** Adopters receive only promoted releases.

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- |
| 1 | Publish latest.json with every release | `xtask/src/package.rs`, `xtask/tests/release.rs`, `.github/workflows/release.yml`, the release docs | Scenario 8 | — |
| 2 | Update the plugin from the binary | `tightship/src/update.rs`, `tightship/src/update/`, `tightship/src/hook/`, `options.rs`, `main.rs`, `failure.rs`, `tightship/tests/update.rs` and its fixtures | Scenarios 1 to 6 | 1 |
| 3 | Wire the option and the hooks into every plugin | `content/options.toml`, `xtask/src/build/`, the launchers, `xtask/tests/` | Scenarios 1 and 6 in a built plugin | 2 |
| 4 | Ask for the option and update Claude Code on a rerun | `installer/`, `harnesses/*/install.*`, `xtask/tests/install.rs` | Scenarios 7 and 9 | 3 |
| 5 | Document the update in every guide | `docs/install/`, `docs/architecture/`, `docs/data-model/` | Read in the docs site | 4 |
