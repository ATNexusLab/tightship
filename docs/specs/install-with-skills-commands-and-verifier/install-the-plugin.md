# #2 — Install the plugin with its companions

> **Status:** published
> **Profile:** API
> **Module:** `harnesses/claude-code/`, `xtask/`
> **Epic:** #2
> **Requirements:** FR-INS-01, FR-INS-02, FR-SKL-01, FR-SKL-02, FR-VER-01, FR-VER-02, BR-04
> **Sprint:** 2

The adopter adds the plugin's catalog and installs `tightship` with the harness's own plugin commands. The
harness installs the three companion plugins with it, and the session offers the 13 skills, the 8 commands,
and the verifier. The build of epic #1 gains the catalog and the plugin's dependencies; nothing else is
built.

Terms from the SRS, as used here:

- **Adopter**: whoever installs the plugin, the maintainer included
  ([overview](../../requirements/overview.md#stakeholders)).
- **Companion plugins**: ponytail, i-have-adhd, and humanizer, third-party plugins the workflow is used with
  ([BR-04](../../requirements/business-rules.md#br-04)).
- **Catalog**: the harness's file that lists installable plugins and where each one comes from. Claude Code
  calls it a marketplace, `.claude-plugin/marketplace.json`.
- **Verifier**: the read-only subagent `adversarial-verifier`, which tries to refute one claim
  ([FR-VER-01](../../requirements/functional/verifier.md#fr-ver-01)).

## Decisions this spec rests on

Confirmed by the maintainer on 2026-10-14, after two installs in an isolated Claude Code 2.1.299
configuration:

1. The build writes the catalog next to the plugin, in `dist/claude-code/.claude-plugin/marketplace.json`,
   with the plugin's source `./`. Until epic #8 publishes a release, the catalog is added from the local
   folder. Epic #8 commits the catalog to the repository root with an `archive` source pointing at the
   release's zip.
2. The companions are dependencies of the plugin, each named in its own catalog. The adopter adds four
   catalogs and installs once. A companion already installed is reused, never installed a second time. This
   answers the SRS's open question OQ-05 for Claude Code.
3. No companion's version is constrained. Claude Code resolves a constraint against tags named
   `<plugin>--v<version>`; ponytail and humanizer tag `v<version>`, and i-have-adhd has no tag.
4. The verifier keeps `Bash`, because its method mutates a private copy of the repository and runs the
   tests there. It gets no file-editing tool of the harness. FR-VER-02 is changed to say so, in requirements
   v1.1.0.
5. `handoff`, the 9th command of FR-SKL-02, arrives with epic #6, as epic #1's spec decided. This epic
   verifies FR-SKL-02 on the 8 commands that exist.
6. No new `xtask` command. The tests check the generated catalog and manifest; a real install is the
   implementer's run-to-verify, and loading the pieces in a session is on the maintainer's checks.
7. The minimum Claude Code version, OQ-04, stays with epic #8, which tests the release and owns
   NFR-FLEX-01.

## Acceptance Criteria

### Contract

The build of epic #1 keeps its command, its exit codes, and its errors. It writes two more things into
`dist/claude-code/`. The adopter runs Claude Code's own plugin commands.

| Command | Who | Idempotent | Effect |
| --- | --- | --- | --- |
| `cargo xtask build` | Maintainer | Yes | Writes `dist/claude-code/`, now with the catalog and the dependencies |
| `claude plugin marketplace add <owner>/<repo>` | Adopter | Yes | Adds one companion's catalog; run once per companion |
| `claude plugin marketplace add ./dist/claude-code` | Adopter | Yes | Adds the `tightship` catalog |
| `claude plugin install tightship@tightship` | Adopter | Yes | Installs the plugin and the companions it depends on |

The companions' catalogs, in the order an adopter adds them:

| Companion | Catalog name | Repository |
| --- | --- | --- |
| ponytail | `ponytail` | `DietrichGebert/ponytail` |
| i-have-adhd | `i-have-adhd` | `ayghri/i-have-adhd` |
| humanizer | `humanizer` | `blader/humanizer` |

### Request

#### `harnesses/claude-code/harness.toml`

One new array of tables, `[[companions]]`, one entry per companion:

| Key | Type | Required | Validation |
| --- | --- | --- | --- |
| `plugin` | string | Yes | The companion's plugin name in its catalog |
| `marketplace` | string | Yes | The name of the catalog that lists it |

```toml
[[companions]]
plugin = "ponytail"
marketplace = "ponytail"

[[companions]]
plugin = "i-have-adhd"
marketplace = "i-have-adhd"

[[companions]]
plugin = "humanizer"
marketplace = "humanizer"
```

Any other key, or a missing one, fails the build as every other key of `harness.toml` does (rule 5). The
array may be empty. The maintainer writes it by hand.

### Response

#### `dist/claude-code/`

| Path | From | Change |
| --- | --- | --- |
| `.claude-plugin/plugin.json` | `[plugin]`, the workspace version, and `[[companions]]` | Gains `dependencies` |
| `.claude-plugin/marketplace.json` | `[plugin]` and `[[companions]]` | New |
| `skills/`, `agents/` | Canonical content | None |

`plugin.json`, keys in this order, with `dependencies` in the order of `[[companions]]`, each entry
`<plugin>@<marketplace>`:

```json
{
  "name": "tightship",
  "version": "0.1.0",
  "description": "Skills, a command per step of the development loop, and a read-only verifier",
  "author": { "name": "ATNexusLab" },
  "repository": "https://github.com/ATNexusLab/tightship",
  "dependencies": ["ponytail@ponytail", "i-have-adhd@i-have-adhd", "humanizer@humanizer"]
}
```

`marketplace.json`, keys in this order:

```json
{
  "name": "tightship",
  "owner": { "name": "ATNexusLab" },
  "description": "Skills, a command per step of the development loop, and a read-only verifier",
  "allowCrossMarketplaceDependenciesOn": ["ponytail", "i-have-adhd", "humanizer"],
  "plugins": [
    {
      "name": "tightship",
      "source": "./",
      "description": "Skills, a command per step of the development loop, and a read-only verifier"
    }
  ]
}
```

| Key | From |
| --- | --- |
| `name` | `[plugin].name` |
| `owner.name` | `[plugin].author` |
| `description`, `plugins[0].description` | `[plugin].description` |
| `allowCrossMarketplaceDependenciesOn` | Each distinct `marketplace` of `[[companions]]`, in order of first appearance |
| `plugins[0].name` | `[plugin].name` |
| `plugins[0].source` | `"./"`: the catalog and the plugin share `dist/claude-code/` |

Both files are written as pretty-printed JSON, two-space indentation, ending in a newline. With no
companion, `plugin.json` has no `dependencies` key and `marketplace.json` no
`allowCrossMarketplaceDependenciesOn` key.

The build's success output keeps its lines; the file count grows by one:

```text
Built dist/claude-code/: 22 pieces (13 skills, 8 commands, 1 agent), 28 files.
Try it: claude --plugin-dir dist/claude-code
```

#### What the adopter sees

Literal output of Claude Code 2.1.299, observed on 2026-10-14:

| Moment | Output |
| --- | --- |
| Install, every companion catalog added | `✔ Successfully installed plugin: tightship@tightship (scope: user) (+ 3 dependencies: ponytail, i-have-adhd, humanizer)` |
| Install, a companion catalog missing | `Warning: dependency "ponytail@ponytail" was not installed (no marketplace you have added lists it); … "tightship" will not load without them` |
| `claude plugin list`, a companion missing | `Status: ✘ failed to load` and `` Error: Dependency "ponytail@ponytail" is not installed — run `claude plugin install ponytail@ponytail`, or check that its marketplace is added `` |
| `claude plugin details tightship` | `Skills (21)` naming the 13 skills and 8 commands, and `Agents (1)  adversarial-verifier` |

Claude Code prints these messages, and a later version may word them differently. The spec quotes them so
the scenarios can be checked.

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| Maintainer | Write access to the repository | Edits `[[companions]]` and runs the build |
| Adopter | None from this project | Runs Claude Code's plugin commands on their own machine |

---

## Business Rules

### 1. Authorization

N/A. The build runs on a local clone and the install on the adopter's machine, with no service of this
project behind either.

### 2. The companions are dependencies

- **When:** `cargo xtask build` renders `plugin.json`.
- Each `[[companions]]` entry becomes one `dependencies` entry, `<plugin>@<marketplace>`, with no version.
- Claude Code installs every dependency it can resolve with the plugin, at the same scope, and reuses one the
  adopter already has enabled at that scope.

### 3. The catalog allows exactly the companions' catalogs

- **When:** `cargo xtask build` renders `marketplace.json`.
- `allowCrossMarketplaceDependenciesOn` lists each catalog named in `[[companions]]` once, and nothing else.
  Without it, Claude Code installs `tightship` and skips every companion.

### 4. A missing companion catalog stops the plugin, not the install

- **When:** the adopter installs `tightship` before adding a companion's catalog.
- Claude Code installs the plugin, warns that the companion was not installed, and does not load `tightship`
  until the companion is installed: the workflow never runs without its companions (BR-04).
- Adding the missing catalog with `claude plugin marketplace add` installs the companion; `tightship` then
  loads at the next session start or `/reload-plugins`.

### 5. `harness.toml` is valid

- **When:** while `build` reads `harnesses/`, as in epic #1.
- A `[[companions]]` entry with an unknown key, a missing key, or a value that is not a string fails with
  `HARNESS_INVALID`, and the build writes no `dist/claude-code/`.

### 6. Every piece reaches the session

- Each of the 13 skills and the 8 commands ships as `skills/<name>/SKILL.md`, the verifier as
  `agents/adversarial-verifier.md`, as epic #1 renders them.
- A skill's supporting files ship next to its `SKILL.md`: `spec-writing/templates.md`, and
  `project-docs/docs-theme.css`, `image-viewer.css`, and `image-viewer.js`.
- A command receives the adopter's arguments where its canonical body holds `{{capability:arguments}}`.

### 7. The verifier edits no file through the harness

- The verifier's `tools` is `Read, Grep, Glob, Bash`, from `[access]` of `harness.toml`: no tool that writes
  or edits a file.
- Its instructions confine every mutation to a private copy in its scratchpad and require the adopter's
  working tree to be as it found it when it returns.

### 8. Persistence and audit

- **Files changed:** `dist/claude-code/.claude-plugin/plugin.json` and `marketplace.json`, by the build. On
  the adopter's machine, only what Claude Code's own plugin commands write.
- **Audit:** N/A. Git records every change to `harness.toml`; Claude Code records installed plugins in its
  own configuration.
- **Events/integrations fired:** Claude Code clones each companion's catalog from GitHub when the adopter
  adds it. The build uses no network.

---

## Errors

| Code | Exit | When | Message |
| --- | --- | --- | --- |
| `HARNESS_INVALID` | `1` | Rule 5 | "`harnesses/<harness>/harness.toml: <reason>`" |

`<reason>` is the TOML parser's message, such as ``missing field `marketplace` ``. Epic #1 already reports
an invalid `harness.toml` this way; this spec gives the error its code.

## Side Effects

- **Persistence:** the build replaces `dist/claude-code/` whole, as in epic #1, now with one more file.
- **Concurrency:** N/A. One maintainer runs one build at a time; Claude Code serializes its own plugin
  commands.
- **Transaction:** the catalog and the manifest are written into `dist/.claude-code.tmp/` with the rest of
  the plugin and published by the same rename.

---

## Acceptance Scenarios (Gherkin)

### Scenario 1 — Build the catalog and the dependencies (happy path)

```gherkin
Given harnesses/claude-code/harness.toml lists the three companions
When the maintainer runs `cargo xtask build`
Then it exits 0 and reports 28 files
And dist/claude-code/.claude-plugin/plugin.json lists the dependencies
  "ponytail@ponytail", "i-have-adhd@i-have-adhd", "humanizer@humanizer", in that order
And dist/claude-code/.claude-plugin/marketplace.json is named "tightship", lists the plugin "tightship"
  with source "./", and allows exactly "ponytail", "i-have-adhd", "humanizer"
And `claude plugin validate dist/claude-code/.claude-plugin/plugin.json` reports "Validation passed" with no warning
```

### Scenario 2 — Install with the companions (FR-INS-01.1, FR-INS-02.1, happy path)

```gherkin
Given a Claude Code configuration with no plugin installed
And the adopter added the catalogs DietrichGebert/ponytail, ayghri/i-have-adhd, and blader/humanizer
When the adopter runs `claude plugin marketplace add ./dist/claude-code`
And runs `claude plugin install tightship@tightship`
Then Claude Code reports "(+ 3 dependencies: ponytail, i-have-adhd, humanizer)"
And `claude plugin list` shows tightship@tightship, ponytail@ponytail, i-have-adhd@i-have-adhd, and
  humanizer@humanizer, each "✔ enabled"
```

### Scenario 3 — A companion already installed is reused (alternative)

```gherkin
Given ponytail@ponytail is installed and enabled at user scope
When the adopter installs tightship@tightship at user scope
Then `claude plugin list` shows ponytail once, as ponytail@ponytail
```

### Scenario 4 — A companion's catalog is missing (BR-04, exception)

```gherkin
Given the adopter added only the tightship catalog
When the adopter runs `claude plugin install tightship@tightship`
Then Claude Code warns that "ponytail@ponytail", "i-have-adhd@i-have-adhd", and "humanizer@humanizer" were
  not installed
And `claude plugin list` shows tightship@tightship as "✘ failed to load"
When the adopter adds the three companion catalogs
Then the three companions are installed and tightship@tightship shows "✔ enabled"
```

### Scenario 5 — Every skill and command is listed (FR-SKL-01.1)

```gherkin
Given the plugin is installed
When the adopter runs `claude plugin details tightship`
Then it lists 21 skills: the 13 baseline skills and the 8 commands
And it lists 1 agent, adversarial-verifier
And typing "/tightship:" in a session offers each of the 21
```

### Scenario 6 — A skill arrives with its supporting file (FR-SKL-01.2)

```gherkin
Given the plugin is installed
When the agent loads tightship:spec-writing in a session
Then it receives the whole SKILL.md
And it reads templates.md from the same folder when the skill points to it
```

### Scenario 7 — A command receives its arguments (FR-SKL-02.1)

```gherkin
Given the plugin is installed
When the adopter invokes `/tightship:spec #42`
Then the agent receives the spec command's instructions with "#42" where the body says
  {{capability:arguments}}
```

### Scenario 8 — The verifier returns a verdict (FR-VER-01.1)

```gherkin
Given the plugin is installed
When the agent dispatches tightship:adversarial-verifier with a claim
Then the verifier returns a block with "verdict:" set to refuted, holds, or unverifiable
```

### Scenario 9 — The verifier cannot edit a file (FR-VER-02.1, exception)

```gherkin
Given dist/claude-code/agents/adversarial-verifier.md has tools "Read, Grep, Glob, Bash"
When the verifier, running, is asked to edit a file with the harness's file-editing tool
Then the tool is not available to it and no file changes
```

### Scenario 10 — The verifier leaves the working tree as it found it (FR-VER-02.2)

```gherkin
Given a repository with a clean `git status`
When the verifier checks a claim by mutating code and running the tests
Then its mutations happen in a copy under its scratchpad
And `git status` of the repository is clean when it returns
```

### Scenario 11 — A companion entry misses a key (exception)

```gherkin
Given harness.toml holds a [[companions]] entry with plugin = "ponytail" and no marketplace
When the maintainer runs `cargo xtask build`
Then it exits 1
And stderr holds "error: harnesses/claude-code/harness.toml: missing field `marketplace`"
And dist/claude-code/ does not exist
```

### Scenario 12 — No companion (alternative)

```gherkin
Given harness.toml has no [[companions]] entry
When the maintainer runs `cargo xtask build`
Then plugin.json has no "dependencies" key
And marketplace.json has no "allowCrossMarketplaceDependenciesOn" key
```

---

## Architecture impact

Applied to [docs/architecture/](../../architecture/README.md), marked M1, #2:

- **Context and scope:** the companion plugins are installed as dependencies, each from its own catalog.
- **Building blocks:**
  - Harness definitions gain `[[companions]]`.
  - Claude Code plugin gains its catalog and dependencies, and is installed from `dist/claude-code/`.
- **Deployment:** the adopter's machine installs the plugin by adding four catalogs and installing once;
  epic #8 moves the catalog to the repository root with an `archive` source.

## Data model impact

N/A: this epic reads and writes no vault note and no option state. The catalog and the manifest are the
harness's formats, described under Response.

## Out of Scope

- **The catalog in the repository root, the release zip, and its `archive` source:** epic #8, which builds
  the release.
- **The install guide, and companions on a harness whose plugin mechanism installs no dependency**
  (FR-INS-02.2, FR-INS-08): epic #8 writes the Claude Code guide; the other harnesses are M2 to M4.
- **Removing the companions on uninstall:** Claude Code keeps auto-installed dependencies until
  `claude plugin prune`; FR-INS-06 belongs to epic #3.
- **The options:** epic #3.
- **The `handoff` command:** epic #6.
- **The minimum Claude Code version (OQ-04):** epic #8.
- **Constraining the companions' versions:** none of them tags releases the way Claude Code resolves a
  constraint (decision 3).

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on | Issue |
| --- | --- | --- | --- | --- | --- |
| 1 | Install the plugin with its companions from the built catalog | `harnesses/claude-code/harness.toml` (`[[companions]]`), `xtask/src/harness.rs`, `xtask/src/claude_code.rs` (dependencies and catalog), `xtask/tests/build.rs` and its fixture in `xtask/tests/common/`, `AGENTS.md` (the install command), about 6 files | Scenarios 1, 11 and 12 by tests; 2 to 4 by an install in an isolated Claude Code configuration; 5 to 10 in a session | — | #14 |
