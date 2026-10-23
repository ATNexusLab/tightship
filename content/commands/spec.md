+++
name = "spec"
description = "Map a feature into a closed, unambiguous spec — ready to become an epic"
argument-hint = "[the feature to spec, or a path to a draft spec to continue]"
+++

Produce the design document **before** any implementation, for: {{capability:arguments}}

This command writes the spec. It does **not** write code and does **not** open issues — `{{command:epic}}` publishes
a closed spec. Use the `{{skill:spec-writing}}` skill for the templates and the closed-spec gate.

## 1. Locate before asking

Find the facts yourself; spend my attention only on decisions.

- The issue this work belongs to. No work starts without one: the chain is requirement ID (`FR-…`,
  `NFR-…`, `BR-…`) → epic and its spec → sub-issues, each a delivery read in one sitting → the epic's
  branch, commits, and PR. A missing link is a gap to fix, never a shortcut. Check the open issues for
  work that already owns this, duplicates it, or breaks with it; what already has an issue is folded into
  that issue.

- For an epic, its branch: cut it from the updated integration branch before writing any file. The
  spec, its global-docs changes and `{{command:epic}}`'s update are commits there.
- Whether the feature is an item of the current sprint, with an epic issue on `docs/roadmap/`. If it is
  not, say so and ask whether to go on, before spending work on it.
- `docs/requirements/` — name the requirement IDs this feature covers (the epic lists them), and flag any
  `Open question` there that this spec will settle. If the feature contradicts what the requirements
  say, stop and raise it: one of the two documents is wrong.
- `docs/architecture/` and `docs/data-model/` — the containers, components, and entities this
  feature touches.
- The specs already in `docs/specs/<epic>/` and how they are named.
- The repo's `Architecture` section, its doc language, and the module this feature lands in.
- Existing permissions, roles, error codes, message strings, and field conventions to reuse — never invent
  a new one when the codebase already has the concept.
- Any draft of this spec already on disk.

State the profile you picked (**UI** or **API**, per `templates.md`) and why, in one line.

## 2. Grill me — this is the default, not an option

Run a `{{skill:grilling}}` session over the whole feature before writing anything: state the assumptions the spec
rests on, let me correct them in one pass, and turn only what I contest into a question.

**A spec grilling ends later than a plain one.** The standalone method stops when nothing left can change
the shape of the work; a spec has to be *closed*, so it continues until every line of the chosen template
has an answer and the closed-spec gate would pass. Once the assumption list is settled, decide every
routine value — lengths, lifetimes, quotas, error codes — write the whole draft, and hand back one table
of the proposed values to confirm in a single pass. A term taken from the SRS is explained the first time
it appears, with the requirement it comes from. Only a value that changes the shape of the work is asked,
one at a time. Do not stop while any of these is open:

- Who can do this, and which permission gates it.
- What each field is: type, obligatoriness, domain, ordering, limit, where its data comes from.
- Every literal message the user sees, word for word.
- What happens on the exception paths — bad input, missing permission, state changed underneath.
- What persists, what is audited, what is emitted.
- What is deliberately **out of scope**.

A fact I could not possibly know (a table name, an existing enum) is yours to look up, not to ask.

## 3. Write it

Fill the profile's template from `templates.md`, in the repo's doc language, at
`docs/specs/<epic>/<feature>.md`.
Nothing invented: every value in the document traces to something I confirmed or something you found in
the code.

## 4. Break it into tasks

Propose the Task Breakdown and walk me through it before finalizing. Slicing rules are in
`templates.md` — vertical slices, ordered by dependency, every scenario owned by a task.

## 5. Close it

Run the closed-spec gate from the skill line by line and report the result. If it passes, set the status
to `closed` and tell me the spec is ready for `{{command:epic}}`. If it does not, name exactly what is
still open — never close a spec that has a soft spot.

Then close the loop with the requirements: for every question in `docs/requirements/open-questions.md` this spec
answered, write the answer where it belongs, delete the question, fix every link to it, and record the
removal in the SRS changelog. Tell me what changed there, separately.

And with the global docs: what the spec's Architecture impact and Data model impact sections declare is
applied to `docs/architecture/`, `docs/data-model/`, and their diagrams, marked with the feature's
milestone. The feature's use cases go into its area's diagram in `docs/requirements/diagrams/`, and its
screens into `docs/design/pages/<feature>/`. Tell me what changed there separately too. The feature's PR
later makes the map match what was built.

## Also handles

- **ADR** (an architectural decision) → `docs/decisions/NNNN-<slug>.md`, template in the skill. Must
  conform to — or explicitly amend — the repo's `Architecture` section. One can come out of any spec;
  when one does, ask me where it lands: with this spec, with another feature, or on its own. An ADR never
  waits for a sprint.
