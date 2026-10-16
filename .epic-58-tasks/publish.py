import json
import subprocess
from pathlib import Path

SPEC = "docs/specs/cursor-plugin/ship-for-cursor.md"
PARENT = 58
PARENT_ID = "I_kwDOVKAVWs8AAAABXii-3g"
REPOSITORY_ID = "R_kgDOVKAVWg"
MILESTONE = "M2 — Cursor"
ROOT = Path(__file__).parent

CREATE_ISSUE = """
mutation($repositoryId: ID!, $title: String!, $body: String!, $parentIssueId: ID!) {
  createIssue(
    input: {
      repositoryId: $repositoryId
      title: $title
      body: $body
      parentIssueId: $parentIssueId
    }
  ) {
    issue { number url }
  }
}
"""

tasks = [
    {
        "title": "Probe Cursor hooks on macOS and Linux",
        "labels": "docs",
        "rules": "7",
        "scope": "Maintainer probes only; update `.claude/skills/cursor-plugin/SKILL.md` on macOS and Linux.",
        "criterion": "Documented in the skill; contract limit stays 10000 unless a probe forces a change request.",
        "gherkin": "",
        "impacts": "May amend `[contract] limit` in the spec through a change request.",
        "test": [
            "Run the probe steps in the Cursor plugin skill on macOS and Linux.",
            "Update the skill with hook shell, paths, and any cap measured.",
        ],
    },
    {
        "title": "Write companion and marketplace install steps",
        "labels": "docs,feat",
        "rules": "4",
        "scope": "`docs/install/cursor/companions.md`, rule 4 sentence in `docs/install/cursor/install.md`.",
        "criterion": "Scenario 2",
        "gherkin": """### Scenario 2 — Install companions then plugin (FR-INS-02.2)

```gherkin
Given an adopter followed docs/install/cursor/companions.md
When they install tightship from the marketplace
Then ponytail, i-have-adhd, and humanizer are already installed
And tightship's skills and commands appear in Customize
```""",
        "impacts": "Epic install guide task 8 links here.",
        "test": [
            "Read companions.md and install.md against rule 4.",
            "Walk through install in Cursor when task 4 ships a build.",
        ],
    },
    {
        "title": "Add harnesses/cursor/harness.toml and capability entries",
        "labels": "feat,area:harnesses",
        "rules": "2, 3",
        "scope": "`harnesses/cursor/harness.toml` with references, capabilities, neutrality, companions, and `[contract] limit`.",
        "criterion": "Scenario 1",
        "gherkin": """### Scenario 1 — Build both harnesses (happy path)

```gherkin
Given content/ and harnesses/claude-code/ and harnesses/cursor/ are valid
When the maintainer runs cargo xtask build
Then dist/claude-code/ and dist/cursor/ exist
And dist/cursor/commands/spec.md holds a rendered /spec reference
And dist/cursor/skills/grilling/SKILL.md holds no Claude Code literal
```""",
        "impacts": "Neutrality check gains Cursor terms; content/ unchanged.",
        "test": [
            "Add harness.toml; run build once task 4 lands or stub renderer exists.",
            "cargo test on harness fixtures when present.",
        ],
    },
    {
        "title": "Render dist/cursor pieces and manifest",
        "labels": "feat,area:xtask",
        "rules": "2, 3, 11",
        "scope": "`xtask/src/build/cursor.rs`, `build.rs` multi-harness render, `xtask/tests/build.rs` and fixtures.",
        "criterion": "Scenario 1",
        "gherkin": """### Scenario 1 — Build both harnesses (happy path)

```gherkin
Given content/ and harnesses/claude-code/ and harnesses/cursor/ are valid
When the maintainer runs cargo xtask build
Then dist/claude-code/ and dist/cursor/ exist
And dist/cursor/commands/spec.md holds a rendered /spec reference
And dist/cursor/skills/grilling/SKILL.md holds no Claude Code literal
```""",
        "impacts": "Build output lists both harnesses; Claude Code path unchanged.",
        "test": ["cargo xtask build", "cargo test --package xtask build"],
    },
    {
        "title": "Pack contract and emit Cursor hooks",
        "labels": "feat,area:xtask,area:tightship",
        "rules": "6, 7, 8",
        "scope": "Contract pack for cursor, `hooks/hooks.json`, `tightship hook contract-part`, tests in `xtask/tests/`.",
        "criterion": "Scenarios 3, 4, 8",
        "gherkin": """### Scenario 3 — Contract after compaction (FR-CTR-01.2, editor)

```gherkin
Given tightship is installed with the contract option on
And a new Agent conversation has started
When the adopter compacts the context
And sends another prompt
Then the agent's context still holds the contract from its first rule to its last
```

### Scenario 4 — Memory off (FR-CTR-03.1)

```gherkin
Given options.json has "memory": false
When sessionStart and beforeSubmitPrompt hooks run
Then no vault text is injected
And contract parts omit vault rules
```

### Scenario 8 — Option off leaves no contract text (FR-CTR-02.1)

```gherkin
Given options.json has "contract": false
When a new conversation starts
Then the agent's context holds no rule of the contract
```""",
        "impacts": "Shares contract pack logic with Claude Code.",
        "test": [
            "cargo test contract/build hooks fixtures",
            "Manual session: compact then prompt with contract on",
        ],
    },
    {
        "title": "Options file and tightship options subcommands",
        "labels": "feat,area:tightship",
        "rules": "5",
        "scope": "`tightship/src/options/`, hook guards reading options.json, manifest `variables`.",
        "criterion": "Scenarios 4, 8, 9, 10",
        "gherkin": """### Scenario 9 — Set an option (FR-INS-04.1, happy path)

```gherkin
Given options.json does not exist
When the adopter runs tightship options set memory false
Then stdout is "Set memory to false."
And options.json holds "memory": false
```

### Scenario 10 — Unknown option (exception)

```gherkin
When the adopter runs tightship options set not_an_option true
Then stderr is "error: unknown option \\"not_an_option\\""
And the exit code is 1
```""",
        "impacts": "Data model options.md already documents the file.",
        "test": [
            "cargo test tightship options",
            "Run subcommands against a temp HOME",
        ],
    },
    {
        "title": "Wire memory and handoff hooks for Cursor",
        "labels": "feat,area:tightship,area:xtask",
        "rules": "7, 10",
        "scope": "`tightship/hook/*`, cursor `hooks/hooks.json`, handoff unchanged in content.",
        "criterion": "Scenario 5",
        "gherkin": """### Scenario 5 — Same vault in Claude Code and Cursor (NFR-COMP-01.1)

```gherkin
Given the adopter set the same vault_folder in Claude Code and in options.json
When they write a note from Claude Code
Then a Cursor session with memory on shows that note at session start
```""",
        "impacts": "Depends on #60 for `~` vault paths.",
        "test": ["cargo test vault/hook", "Cross-harness note visible in Cursor editor"],
    },
    {
        "title": "Install guide and try-it line in AGENTS.md",
        "labels": "docs,feat",
        "rules": "4, 11",
        "scope": "`docs/install/cursor/` (README, install, options, uninstall), `AGENTS.md`, root `README.md` try-it.",
        "criterion": "Scenarios 2, 6",
        "gherkin": """### Scenario 6 — CLI loads plugin (NFR-FLEX-02.1, interactive)

```gherkin
Given dist/cursor is built
When the adopter runs agent --plugin-dir dist/cursor
And asks which slash commands are available
Then the answer lists /spec and the other workflow commands
```""",
        "impacts": "Portuguese install pages follow BR-03 when English pages change.",
        "test": [
            "Follow install guide on a clean Cursor config",
            "agent --plugin-dir dist/cursor",
        ],
    },
    {
        "title": "CLI status line probe and enable/render path",
        "labels": "feat,area:tightship",
        "rules": "9",
        "scope": "`tightship/status-line` for CLI config path, probe notes in cursor-plugin skill.",
        "criterion": "Business rule 9 (CLI status line); NFR-FLEX-02.1 for FR-STL in CLI only.",
        "gherkin": "",
        "impacts": "Editor status line remains out of scope.",
        "test": [
            "Probe statusLine in ~/.cursor/cli-config.json",
            "Enable, render, decline, and restore previous",
        ],
    },
    {
        "title": "Package tightship-cursor.zip and catalog",
        "labels": "build,feat,area:xtask",
        "rules": "12, 11",
        "scope": "`xtask/package.rs`, `.github/workflows/release.yml`, `.cursor-plugin/marketplace.json` at repo root.",
        "criterion": "Release contract in spec decision 12.",
        "gherkin": "",
        "impacts": "Release workflow ships two zips per tag.",
        "test": [
            "cargo xtask build --binaries binaries && cargo xtask package vX.Y.Z",
            "Inspect zip and marketplace.json",
        ],
    },
    {
        "title": "Release checklist rows for Cursor",
        "labels": "docs",
        "rules": "11",
        "scope": "`docs/release/checklist.md`, NFR-FLEX-02.1 sign-off on three OS.",
        "criterion": "Scenario 6",
        "gherkin": """### Scenario 6 — CLI loads plugin (NFR-FLEX-02.1, interactive)

```gherkin
Given dist/cursor is built
When the adopter runs agent --plugin-dir dist/cursor
And asks which slash commands are available
Then the answer lists /spec and the other workflow commands
```""",
        "impacts": "Checklist gates M2 release.",
        "test": [
            "Run checklist on macOS, Linux, Windows at Cursor 3.25.6+ and agent 2026.10.14+",
        ],
    },
]


def render_body(task: dict) -> str:
    gherkin = task["gherkin"]
    acceptance = task["criterion"]
    if gherkin:
        acceptance = f"{acceptance}\n\n{gherkin}"
    tests = "\n".join(f"{index}. {step}" for index, step in enumerate(task["test"], 1))
    return f"""## Context

Business rule(s) {task['rules']} — spec: `{SPEC}`

## Scope

{task['scope']}

## Acceptance criterion

{acceptance}

## Possible impacts

{task['impacts']}

## How to test

{tests}
"""


def main() -> None:
    numbers: list[int] = []
    for index, task in enumerate(tasks, 1):
        path = ROOT / f"{index:02d}.md"
        path.write_text(render_body(task), encoding="utf-8")
        body = path.read_text(encoding="utf-8")
        payload = json.dumps(
            {
                "query": CREATE_ISSUE,
                "variables": {
                    "repositoryId": REPOSITORY_ID,
                    "title": task["title"],
                    "body": body,
                    "parentIssueId": PARENT_ID,
                },
            }
        )
        result = json.loads(
            subprocess.check_output(
                ["gh", "api", "graphql", "--input", "-"],
                input=payload,
                text=True,
            )
        )
        if "errors" in result:
            raise RuntimeError(result["errors"])
        issue = result["data"]["createIssue"]["issue"]
        number = issue["number"]
        subprocess.run(
            [
                "gh",
                "issue",
                "edit",
                str(number),
                "--repo",
                "ATNexusLab/tightship",
                "--milestone",
                MILESTONE,
                "--add-label",
                task["labels"],
            ],
            check=True,
        )
        numbers.append(number)
        print(number, task["title"])
    (ROOT / "issues.json").write_text(json.dumps(numbers), encoding="utf-8")


if __name__ == "__main__":
    main()
