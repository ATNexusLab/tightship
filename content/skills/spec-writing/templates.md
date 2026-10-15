# Spec templates

Two profiles. Pick one at the start of `{{command:spec}}` — the profile is a header field, not a guess made later.

- **UI** — the feature has a screen, a modal, or a form a human operates.
- **API** — the feature is an endpoint, a job, or a service contract with no screen of its own.

A feature that has both gets **two** specs (one per profile) linked to the same epic — never one hybrid
document, because the field-level detail of each profile is what makes the spec testable.

**The spec is written entirely in the repo's doc language**: headings, header labels, status values,
table columns, and the Gherkin keywords of that language. This file names them in English; the names
are roles, not literals. When the repo already has specs, their wording of each heading and status is
the one to reuse, so every spec in a repo reads the same. **Identifiers, routes, table and column names
follow the repo's code language** — they are code.

Every section is mandatory. A section that does not apply is written as `N/A` with one line saying why.
`N/A` proves it was considered; a missing section proves nothing.

The status moves `draft` → `closed` (`{{command:spec}}`, after the gate) → `published` (`{{command:epic}}`).

---

## Profile: UI

````markdown
# <ID> — <Verb> <object> (<actor>)

> **Status:** draft
> **Profile:** UI
> **Module:** <where it lives in the repo>
> **Epic:** —
> **Requirements:** FR-<AREA>-NN, NFR-<CHAR>-NN
> **Sprint:** <N>

## Acceptance Criteria

### Visual reference

<Link to the screen's page in `docs/design/pages/<feature>/`, or `N/A — no design yet; layout set by the rules below`>

### Screen specification

<What the screen is: page, modal, drawer; what it shows when it opens; what it lists.>

### Menu path

`Menu` → `Submenu` → `Feature`

### Roles and privileges

| Role / Action | Privilege | Note |
| --- | --- | --- |
| <role that sees the feature> | `<PERMISSION>` | — |
| <specific action> | `<ACTION_PERMISSION>` | <what the permission grants> |

---

## Business Rules

Numbered, one per behavior. Each rule settles three things: **when** it fires, **what** the system does,
and **which literal message** appears (in quotes, exact text). Numeric limits are numbers, never
"a reasonable limit".

### 1. Access restriction

- Access is restricted to users with the `<PERMISSION>` privilege.
- **Check:** <the exact moment the permission is checked>. Without the permission, processing is
  blocked and the system shows:
  > "<literal message>"

  and redirects to <destination>.

#### Flow origin

<Where the user arrives at this screen from — the list, the menu action, the previous story.>

### 2. <Next rule>

### N. Persistence and audit

- **User boundary (visible):** <the action the user clicks>.
- **System action (internal):** writes to the audit log *who* (user/IP), *what* (<the operation and
  the data changed>) and *when* (UTC).

---

## Acceptance Scenarios (Gherkin)

Minimum coverage: **1 happy path**, **1 alternative per dynamic behavior**, and **1 exception per rule
that can fail**. A business rule with no matching scenario is a rule that cannot be tested.

### Scenario 1 — <name> (happy path)

```gherkin
Given <initial state and permission>
And <precondition>
When <user action>
Then <observable result>
And <side effect: persistence, message, redirect>
```

### Scenario 2 — <name> (alternative path)

### Scenario 3 — <name> (exception path)

---

## Screen Data Dictionary (Fields)

| Field name | Type | Enabled | Required | Rule / Validation |
| --- | --- | --- | --- | --- |
| `<Displayed label>` | <Dropdown / Text (N) / Date / Checkbox> | Yes / No / Conditional (<condition>) | Yes / No / Conditional | <data source, domain, ordering, limit, mask> |

## Screen Actions

| Action name | Target / Effect | Enabling rule | Messages |
| --- | --- | --- | --- |
| <Button label> | <what it runs and where it goes> | <always enabled / condition / required permission> | Success: "<literal>" · Error: "<literal>" |

---

## Architecture impact

<The building blocks, runtime scenarios, deployment and concepts of `docs/architecture/` this feature
creates or changes — each by the name the map uses.>

## Data model impact

<The entities, attributes, relations and files/tables of `docs/data-model/` this feature creates or
changes, or `N/A — <why>`.>

## Out of Scope

- <What this spec deliberately does not cover, and where it will be handled.>

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- |
| 1 | <imperative issue title> | <files/layer the task touches> | <Scenario N, or rule N verified> | — |
| 2 | <...> | <...> | <...> | 1 |
````

---

## Profile: API

````markdown
# <ID> — <Verb> <resource>

> **Status:** draft
> **Profile:** API
> **Module:** <where it lives in the repo>
> **Epic:** —
> **Requirements:** FR-<AREA>-NN, NFR-<CHAR>-NN
> **Sprint:** <N>

## Acceptance Criteria

### Contract

| Method | Route | Auth / Role | Idempotent |
| --- | --- | --- | --- |
| `POST` | `/v1/<resource>` | `<ROLE>` | Yes / No |

### Request

| Field | Type | Required | Validation |
| --- | --- | --- | --- |
| `field` | `string` | Yes | <limit, format, domain> |

```json
{ "field": "example" }
```

### Response

**`201 Created`**

```json
{ "id": "uuid", "field": "example" }
```

| Status | When |
| --- | --- |
| `201` | <condition> |
| `400` | <condition> |
| `403` | <condition> |

### Roles and privileges

| Role | Permission | Note |
| --- | --- | --- |
| <role> | `<PERMISSION>` | <what it grants> |

---

## Business Rules

The same discipline as the UI profile: numbered, with the trigger, the effect, and the literal error
message.

### 1. Authorization

<Who may call it, when the check happens, what happens without the permission.>

### 2. <Domain rule>

### N. Persistence and audit

- **Tables/columns changed:** <list>.
- **Audit:** writes *who* (user/IP), *what* (<operation and data>) and *when* (UTC).
- **Events/integrations fired:** <list, or `N/A`>.

---

## Errors

| Code | HTTP | When | Message |
| --- | --- | --- | --- |
| `<ERROR_CODE>` | `400` | <exact condition> | "<literal message>" |

## Side Effects

- **Persistence:** <what it writes, in which state>
- **Concurrency:** <what happens if the state changes between the read and the write>
- **Transaction:** <what is atomic with what>

---

## Acceptance Scenarios (Gherkin)

Minimum coverage: **1 happy path**, **1 per error in the table above**, **1 denied authorization**.

### Scenario 1 — <name> (happy path)

```gherkin
Given <actor> holds the <PERMISSION> permission
And <resource state>
When it sends `POST /v1/<resource>` with <payload>
Then the system responds `201`
And persists <what, in which state>
And writes the operation to the audit log
```

### Scenario 2 — <name> (exception)

---

## Architecture impact

<The building blocks, runtime scenarios, deployment and concepts of `docs/architecture/` this feature
creates or changes — each by the name the map uses.>

## Data model impact

<The entities, attributes, relations and files/tables of `docs/data-model/` this feature creates or
changes, or `N/A — <why>`.>

## Out of Scope

- <What this spec deliberately does not cover.>

## Task Breakdown

| # | Title | Scope | Acceptance criterion | Depends on |
| --- | --- | --- | --- | --- |
| 1 | <imperative issue title> | <files/layer the task touches> | <Scenario N verified> | — |
````

---

## Writing the task breakdown

The breakdown table is the contract `{{command:epic}}` publishes verbatim — it does not re-read the spec to invent
tasks. So the table has to hold up on its own:

- **One task = one delivery**: a sub-issue on the epic's branch, sized to be reviewed in one sitting. If
  a task cannot be verified without another task's code, it is half of one — merge them. A task too small
  to be worth its own review cycle joins its neighbour.
- **A task that deletes or moves something counts its consumers.** The importers and the tests that
  reference it are searched for, and the number goes in the task's Scope. A sweep hidden behind
  "delete `<file>`" is what turns one task into a diff nobody can read.
- **Order by dependency.** Depends on carries task numbers, never prose.
- **Every acceptance scenario in the spec maps to at least one task.** A scenario nobody owns is a feature
  nobody builds.
- Slice vertically along the repo's own architecture: one behavior through every part it crosses beats
  one part for every behavior.
