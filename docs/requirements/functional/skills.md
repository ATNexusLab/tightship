# Skills and commands — `SKL`

The skills the agent loads on demand and the commands the adopter invokes.

![Use cases: skills and commands](../diagrams/use-cases-skills.drawio.svg)

<a id="fr-skl-01"></a>

## FR-SKL-01 — Skills available

> As an adopter, I want the workflow's skills in my harness, so that the agent loads each one when a task
> matches it.

The system shall make each of the 13 skills of the baseline available to the agent.

| Attribute | Value |
| --- | --- |
| Rationale | The skills carry the method for each kind of task. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **FR-SKL-01.1** — Given the plugin is installed, when the adopter lists the skills the harness offers,
  then each of the 13 baseline skills is listed.
- **FR-SKL-01.2** — Given the plugin is installed, when a skill is loaded, then the agent receives its
  whole text and every file it points to.

<a id="fr-skl-02"></a>

## FR-SKL-02 — Commands invocable

> As an adopter, I want the workflow's commands in my harness, so that I run each step of the
> development loop by name.

The system shall make each of the 9 commands of the baseline invocable by the adopter.

| Attribute | Value |
| --- | --- |
| Rationale | The commands are how the adopter drives the development loop. |
| Source | Maintainer, elicitation 2026-10-14 |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **FR-SKL-02.1** — Given the plugin is installed, when the adopter invokes any of the 9 baseline
  commands, then the agent receives that command's instructions with the adopter's arguments.

<a id="fr-skl-03"></a>

## FR-SKL-03 — References resolve

> As an adopter, I want every name a piece refers to to exist in my install, so that no instruction
> points at something missing.

The system shall resolve every reference from one shipped piece to another skill, command, subagent, or
script in the installed plugin.

| Attribute | Value |
| --- | --- |
| Rationale | A harness can rename or namespace plugin pieces, which breaks a reference written for the pre-plugin install. |
| Source | Analysis 2026-10-14, approved by the maintainer |
| Priority | Must |
| Status | approved |
| Milestone | M1 |
| Since | v1.0.0 |

**Acceptance criteria**

- **FR-SKL-03.1** — Given the built plugin, when every reference in its shipped pieces is checked, then
  each one names a piece present in that plugin under the name the harness gives it.

<a id="fr-skl-04"></a>

## FR-SKL-04 — The bootstrap writes a lean repository contract

> As an adopter, I want my repository's `AGENTS.md` to hold only what every task in it needs, so that the
> agent does not carry what the repository already shows or what serves one area.

The system shall write, through the bootstrap command, an `AGENTS.md` that holds only what applies to every
task in the repository and that its files do not show, and that describes the architecture as the current
shape, which the adopter may change.

| Attribute | Value |
| --- | --- |
| Rationale | The repository contract is read at every session, like the contract. Commands the build files state, procedures of one area, and an architecture to conform to strictly made it long, and made the agent defend the current shape against the adopter's decision to change it. |
| Source | Maintainer, change request [#118](https://github.com/ATNexusLab/tightship/issues/118) |
| Priority | Must |
| Status | approved |
| Milestone | M3 |
| Since | v2.0.0 |

**Acceptance criteria**

- **FR-SKL-04.1** — Given a repository with no `AGENTS.md`, when the adopter runs the bootstrap command,
  then the `AGENTS.md` it writes states no command, path, or structure the repository's files state, and no
  procedure of one area or step.
- **FR-SKL-04.2** — Given a repository whose `AGENTS.md` holds such text, when the adopter runs the
  bootstrap command, then it proposes the cut and changes the file only once the adopter approves.

<a id="fr-skl-05"></a>

## FR-SKL-05 — The bootstrap writes project skills

> As an adopter, I want each procedure of one area or step of my repository as a project skill, so that the
> agent loads it only when the task needs it.

The system shall write, through the bootstrap command, each procedure of one area or step of the repository
as a skill in the repository's `.agents/skills/` folder, and make it reachable from every supported harness.

| Attribute | Value |
| --- | --- |
| Rationale | The workflow gave a repository only its `AGENTS.md`, so every procedure went there and was read at every session. `.agents/skills/` is the folder of the open Agent Skills format, the most general one the harnesses read. |
| Source | Maintainer, change request [#118](https://github.com/ATNexusLab/tightship/issues/118) |
| Priority | Must |
| Status | approved |
| Milestone | M3 |
| Since | v2.0.0 |

**Acceptance criteria**

- **FR-SKL-05.1** — Given a repository with a procedure of one area or step, when the adopter runs the
  bootstrap command, then `.agents/skills/<name>/SKILL.md` holds that procedure.
- **FR-SKL-05.2** — Given a supported harness that does not read `.agents/skills/`, when a session starts
  in a repository with project skills, then the harness lists them.

<a id="fr-skl-06"></a>

## FR-SKL-06 — A skill edits prose for people

> As an adopter, I want a skill that removes the tells of machine writing from prose meant for people, so
> that docs, pull requests, and issues read as written by their author.

The system shall ship a skill that edits prose written for people so that it reads as written by its
author, without changing what it says.

| Attribute | Value |
| --- | --- |
| Rationale | The contract sent every text for people through a companion that did this. With the companions gone ([BR-05](../business-rules.md#br-05)), the workflow carries the skill itself. |
| Source | Maintainer, change request [#149](https://github.com/ATNexusLab/tightship/issues/149); the [spec of epic #140](../../specs/own-the-companions/own-the-companions.md) |
| Priority | Must |
| Status | approved |
| Milestone | M3 |
| Since | v4.0.0 |

**Acceptance criteria**

- **FR-SKL-06.1** — Given the built plugin in a supported harness, when the skill list is read, then the
  prose skill is listed with its description, and the contract's rule on prose names it.
- **FR-SKL-06.2** — Given a text the skill edited, when the result is compared with the original, then
  every claim, fact, name, number, date, and quote of the original is in it.

<a id="fr-skl-07"></a>

## FR-SKL-07 — The bootstrap sets up the editors the project uses

> As an adopter, I want the bootstrap to set up VS Code, Zed, or both, so that the project opens in my
> editor with its formatter, linter, tasks, and diagrams working and no manual step.

The system shall write, through the bootstrap command, the setup of each editor the adopter confirms,
VS Code in `.vscode/` and Zed in `.zed/`, and export every diagram so that both editors render it.

| Attribute | Value |
| --- | --- |
| Rationale | The bootstrap wrote `.vscode/` only, and the diagrams it exports turned black in Zed's preview, which resolves no `light-dark()` color and shows a transparent background on a dark editor. |
| Source | Maintainer, [#156](https://github.com/ATNexusLab/tightship/issues/156); the [spec of epic #158](../../specs/zed-and-lean-artifacts/zed-and-lean-artifacts.md) |
| Priority | Should |
| Status | approved |
| Milestone | M3 |
| Since | v4.1.0 |

**Acceptance criteria**

- **FR-SKL-07.1** — Given a repository the adopter bootstraps for Zed, when it opens in Zed, then files
  format on save, the linter the gates run shows its diagnostics, and the `docs: serve` task runs, with no
  manual setup.
- **FR-SKL-07.2** — Given a diagram the workflow exported, when the file is read, then it holds 0
  `light-dark(` colors, its background is white, and draw.io still opens the embedded diagram.

<a id="fr-skl-08"></a>

## FR-SKL-08 — A skill keeps production artifacts lean

> As an adopter, I want the agent to build production artifacts that carry only what runs, so that no
> image or bundle ships development dependencies or files the program never reads.

The system shall ship a skill, loaded when the agent writes or changes what builds or ships a production
artifact and by the audit command when its target does, that keeps the artifact to the build output, the
runtime dependencies, and the files read at run time.

| Attribute | Value |
| --- | --- |
| Rationale | In two projects the production image installed the development dependencies, and no ignore file kept the rest of the repository out. |
| Source | Maintainer, [#157](https://github.com/ATNexusLab/tightship/issues/157); the [spec of epic #158](../../specs/zed-and-lean-artifacts/zed-and-lean-artifacts.md) |
| Priority | Should |
| Status | approved |
| Milestone | M3 |
| Since | v4.1.0 |

**Acceptance criteria**

- **FR-SKL-08.1** — Given the built plugin in a supported harness, when the skill list is read, then the
  production artifact skill is listed with its description, and the delivery skill names it.
- **FR-SKL-08.2** — Given a project whose artifact carries a development dependency, has no ignore file or
  an open one, or builds and runs in one step, when the adopter runs the audit command on it, then each
  case is reported as a finding.
