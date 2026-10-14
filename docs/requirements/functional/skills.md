# Skills and commands — `SKL`

The skills the agent loads on demand and the commands the adopter invokes.

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
