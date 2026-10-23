# 0003. The plugin updates itself from GitHub Releases

- Status: accepted
- Date: 2026-10-23

## Context

[FR-INS-10](../requirements/functional/install.md#fr-ins-10) requires each release to reach every harness with
no step from the adopter. Each harness offers something different:

- **Claude Code** updates a plugin by itself when its marketplace has auto-update on. That is off by default
  for a marketplace outside Anthropic's, and `extraKnownMarketplaces.<name>.autoUpdate` in the user settings
  turns it on.
- **Cursor** keeps a plugin added from a repository at the commit it was added at. Only its official
  marketplace, or a paid team marketplace, updates a plugin. The installer puts the plugin in
  `~/.cursor/plugins/local`, which nothing updates.
- **Antigravity CLI** updates only the plugins of its official catalog and has no `agy plugin update`.

So in Cursor and Antigravity CLI the plugin has to update itself. ADR 0001's **Reach** let the plugin reach one
host, the git remote of the adopter's vault, and the binary's map said no hook uses the network. ADR 0002 left
the binary with no HTTP client, no hashing, and no archive reader.

## Decision

**In Cursor and Antigravity CLI, the binary updates the plugin from the repository's GitHub Releases.**

- The session-start hook `tightship hook update` reads the update state in the plugin's data folder. It
  passes on any notice the last update left. When the `auto_update` option is on and the last check was more
  than 24 hours ago, it starts `tightship update` as a detached process and returns at once. The hook itself
  never waits on the network.
- `tightship update` takes a lock file, reads `version` and `repository` from the plugin's own manifest, and
  downloads `<repository>/releases/latest/download/latest.json`. That file is a new asset of every release,
  holding the version, the tag, and each harness's archive with its sha256. When the version is greater than
  the plugin's, the binary downloads the archive by its tag's URL and checks its sha256. It then extracts the
  archive inside the plugin folder and swaps it in, file by file, rolling every rename back if one fails.
- **The binary runs the system's tools as processes, the way it runs git.** On macOS and Linux that is
  `curl`, `sha256sum` (or `shasum -a 256`), and `unzip`. On Windows it is `powershell`, with `irm`,
  `Get-FileHash`, and `Expand-Archive`. The six crates of ADR 0002 stay as they are.

**In Claude Code, the plugin turns the harness's own auto-update on.** The session-start hook
`tightship hook marketplace-auto-update` writes `autoUpdate`, set to the `auto_update` option, on the plugin's
entry in `extraKnownMarketplaces`. Claude Code then updates the plugin itself.

**This amends ADR 0001's Reach.** The plugin now also reaches the GitHub Releases of its own repository, from
a process a session-start hook starts. The binary also writes inside its own plugin folder, and only to put a
release in place. ADR 0002's rule that external programs run as processes now covers the download tools as
well as git.

### Threats, and what holds each one

| Surface | Threat | What holds |
| --- | --- | --- |
| Download | A tampered or truncated archive | The sha256 published in `latest.json` is checked before any file moves. A mismatch changes nothing, and the next session tells the adopter. |
| Download | A downgrade to an older release with a known flaw | Only a version greater than the installed one is applied. |
| Download | A pre-release reaches adopters | `releases/latest` never points at a pre-release. A release reaches adopters only after the maintainer promotes it. |
| Transport | A downgrade to cleartext | `curl` runs with `--proto =https,file`, and `powershell` forces TLS 1.2. The repository comes from the plugin's own manifest, which only someone who can already change the plugin can edit. |
| Archive | A path that escapes the plugin folder, or a symbolic link | After extraction every entry must be a regular file or folder inside the staging folder. Anything else stops the update before a file moves. |
| Apply | A half-applied update leaves a broken plugin | Each file is renamed aside before the new one takes its place. One failed rename rolls back every rename before it. |
| Apply | Two sessions update at once | A lock file taken with `create_new`. A lock older than one hour is treated as abandoned. |
| Hook | A slow network delays the session | The hook only reads a file and starts a detached process. The download runs after the hook has returned, with a 60-second limit on each request. |

The trust root stays the one the installer already has: the repository's GitHub Releases over HTTPS. A
checksum taken from the same release guards against corruption and a mixed-up asset, not against someone who
controls the repository. Signing releases is out of scope for this decision.

## Consequences

- A promoted release reaches Cursor and Antigravity CLI within a day of the adopter's next session, and Claude
  Code within its own auto-update cycle.
- The next session says what changed, once, and asks nothing.
- The binary now runs `curl` and `unzip` on macOS and Linux, and `powershell` on Windows. A missing tool fails
  the update silently. The plugin keeps its version and works as before.
- Every release must publish `latest.json`. The package step writes it and the release workflow uploads it.
- A plugin a harness installed from its own marketplace cache is still updated where it lives, and the harness
  may later put its own copy back. Both copies keep working.

## Alternatives considered

- **An HTTP crate, such as `ureq`, with `sha2` and `zip`.** Rejected: three crates and their own dependency
  trees for a once-a-day download that the system's tools already do. ADR 0002 already runs git as a process
  for the same reason.
- **Downloading inside the hook.** Rejected: a slow network would hold every session start. The detached
  process keeps the hook as fast as it was.
- **Only telling the adopter that a release exists.** Rejected by the maintainer, who wants the update
  applied, with a notice afterwards.
- **Staging the update and swapping it in at the next session.** Rejected: a running binary can be renamed on
  every system, Windows included, so the swap can happen at once and the next session already runs the new
  version.
