+++
name = "architecture-reading"
description = "Use before a non-trivial change in an unfamiliar or large codebase — read it ground → data → backend → frontend → seam, then say where the change belongs and what constrains it."
+++

# Architecture Reading

## When to use

- Before planning a feature, refactor, or bugfix in code you don't already understand.
- When a change crosses a module, layer, or app boundary.
- When reality is suspected to diverge from the repo's documented architecture.
- To produce the map the `arch` axis of `{{command:audit}}` returns.

## Scope the read to the change

Depth is set by the change, not by the size of the repo. A change confined to one layer reads that layer
deeply and the neighbours only where they touch it; a change that crosses layers reads every pass it
crosses, end to end. Mapping a codebase completely is a documentation project — this is not that.
Skip any pass the repo doesn't have (no database, no frontend) and say you skipped it.

## Why this order

Read **ground → data → back → front**, then close the seam.

Each layer is constrained by the one before it. Data outlives the code that reads it: the schema records
what the domain actually is, migrations record how it got there, and both survive every rewrite above.
Backend logic is written against that shape whether or not it admits it. The frontend is written against
what the backend exposes.

Reading in the opposite direction builds a mental model out of screens and component props, then spends
the rest of the session revising it every time a constraint below contradicts it — and the contradictions
surface as bugs, not as corrections. Each pass hands the next a set of facts to verify, never assume.

---

## Pass 0 — Ground (the organizational architecture)

Where the code lives and how it ships, before reading any of it.

1. **Repo shape.** One app or many? Monorepo (workspaces, package graph, shared packages) or a single
   deployable? Name every app and library and what each is *for* — a name is not a purpose.
2. **The decided architecture.** The repo's own `AGENTS.md` Architecture section, ADRs, README, docs.
   Record what it *claims*. This is the reference every later pass is checked against.
3. **How it runs and ships.** Package/task scripts, container and compose files, CI/CD pipelines,
   environments and their config sources. The build graph reveals real dependencies that imports hide.
4. **What the history says.** Recent commits and their scope: which areas change together, which are
   frozen, which conventions are current versus fossilized. A convention alive in the last months
   outranks one written in a doc two years ago.
5. **Ownership and boundaries.** Which parts are third-party, generated, vendored, or legacy — and which
   are off-limits. Code you must not touch is architecture too.

**Hand off:** the claimed architecture, the deployable units, and where the change's blast radius ends.

## Pass 1 — Data

The most honest description of the domain, because it is the hardest to change.

1. **The entities and their relations.** Read the schema or migrations, not the model classes. Keys,
   foreign keys with their on-delete behavior, unique constraints, nullability.
2. **The invariants the data enforces itself.** Constraints, checks, defaults, triggers, generated
   columns. Anything guaranteed here does *not* need re-guaranteeing above — and anything guaranteed
   only above is a rule the database will happily let something else violate.
3. **The state machines hiding in enums and status columns.** List the legal values and infer the legal
   transitions. Most domain logic upstream is a partial implementation of these.
4. **The real query patterns.** Indexes are the record of what the application actually asks for.
   An index nobody's query matches, or a hot filter with no index, is a finding.
5. **How code reaches the data.** ORM, query builder, raw statements, stored procedures — and whether
   more than one mechanism coexists. Note transaction boundaries and any row-level access rules.
6. **How the schema changes.** Migration tooling, naming, whether migrations are reversible, whether
   destructive steps are staged. This is the cost of any change you propose here.

**Hand off:** the domain's true shape, the invariants already enforced, and the transition rules.
When this pass stops being a reading and becomes a modeling decision — a new table, an index, a
migration — continue in `{{skill:database-design}}`.

## Pass 2 — Backend

How the domain shape becomes behavior and a contract.

1. **Entrypoints.** Bootstrap file, server start, route or handler registration, background workers,
   scheduled jobs, message consumers. Establish every way work enters the system, not just the HTTP one.
2. **Trace one real operation end to end**, with `path:line` at each hop: entry → validation →
   authorization → application logic → domain rules → data access → response. This reveals the actual
   architecture faster than any folder tree.
3. **Name the layers and the crossing rules.** What may call what, in which direction, and whether the
   rule is *enforced* (types, lint boundaries, module structure) or merely *conventional*. Note where
   dependencies invert and where the boundaries between layers are defined.
4. **Locate the domain rules.** Are they in one layer, or scattered across handlers, services, and the
   database? Every duplicate of a rule is a place it can drift.
5. **Cross-cutting mechanics.** Validation, error handling and mapping, authorization, configuration and
   secrets access, logging and correlation, caching, transactions. Each has exactly one idiom here —
   find it and match it.
6. **The exposed contract.** Endpoints, schemas, events, or IDL — and where its types are defined.
   This is the frontend's entire view of everything above.

**Hand off:** the layer map, the one traced path, the conventions to match, and the contract's surface.
When this pass stops being a reading and becomes a structural decision — where a new module or layer
belongs — follow the repo `AGENTS.md` Architecture section; where it does not cover the case, the
decision goes back to the user.

## Pass 3 — Frontend

How the contract becomes a screen, and where state legitimately lives.

1. **Routing and rendering model.** File- or config-based routes, which parts render on the server versus
   the client, and where that boundary is drawn. Getting this wrong invalidates everything below it.
2. **Where data enters the UI.** Fetch/query layer, cache, revalidation strategy, mutations. Whether
   components fetch for themselves or receive data from a route-level loader.
3. **State ownership.** Sort every piece into: server cache (remote data), URL (shareable view state),
   form state, local component state, global client state. Most frontend architecture damage is remote
   data copied into global state.
4. **Component boundaries.** What is a page, a feature, a shared primitive. Where the design system or
   styling convention lives and how a new component is expected to be composed.
5. **Where the contract's types come from.** Generated from the backend schema, hand-written, or
   duplicated? A hand-copied type is a drift source — note it.
6. **The user-facing invariants.** Loading and empty states, error surfaces, permission-driven rendering,
   and which of these are enforced by a shared shell versus reimplemented per screen.

**Hand off:** the render/state model, the component placement rule, and how the contract is consumed.

## Pass 4 — The seam

The passes are separate readings; the architecture is what happens between them. Take **one real field
or one real rule** and follow it across every pass: column → domain type → contract payload →
component prop → rendered value. Then ask:

- **Where is each rule enforced, and how many times?** A rule implemented once is a design. The same rule
  implemented in the database, the service, and the form is three implementations that will disagree.
- **Where is the trust boundary?** Everything the client sends is input. Any rule enforced only in the
  frontend is not enforced.
- **Does the vocabulary survive the trip?** A concept renamed at each layer means no shared language, and
  every future change pays translation cost.
- **What breaks if this changes?** Trace a schema change forward and a UI requirement backward. The two
  answers bound the real cost of the planned work.
- **Where does reality diverge from Pass 0's claim?** Name each divergence explicitly — it is either a
  finding to fix or a decision to record, never something to quietly work around.

---

## The verdict

Unify the passes into one map. Every claim carries a `path:line`.

1. **The architecture as it actually is** — deployable units, layers, boundary rules, and which are
   enforced versus conventional.
2. **The traced path** — one operation from entry to data and back to screen.
3. **The conventions to conform to**, per layer, with the file that best exemplifies each.
4. **Where the planned change's code and its tests belong** — concrete paths, per layer it touches.
5. **What constrains it** — invariants already enforced below, contracts other consumers depend on,
   migrations required, and the blast radius.
6. **Divergences, risks, and unknowns** — documented-versus-real gaps, duplicated rules, leaky
   boundaries, god-modules, ambiguous ownership. Mark anything the plan must resolve before it starts.

## Evidence rules

- **Cite or don't claim.** Every structural statement points at a file and line.
- **Three occurrences make a convention.** One file is an example; two may be a coincidence. Before
  calling something "the pattern here", find it again elsewhere — and prefer the most recently touched
  instance when two idioms coexist.
- **The code outranks the doc, and the doc still matters.** When they disagree, describe the code as the
  truth and the gap as a finding.
- **Say "unknown" out loud.** An unresolved question named in the map is useful; a confident guess that
  turns out wrong poisons the plan built on it.

## Validation checklist

- [ ] Pass 0's claimed architecture recorded, and every later pass checked against it.
- [ ] One operation traced end to end, with `path:line` at each hop.
- [ ] Layers, responsibilities, and crossing rules explicit — and marked enforced vs conventional.
- [ ] Data invariants listed, and every rule duplicated above them flagged.
- [ ] Frontend state sorted by owner; anything trusting the client identified.
- [ ] The planned change's code and test locations named concretely, per layer.
- [ ] Divergences and unknowns stated, not smoothed over.

## Never do

- Propose the implementation here — map the ground; planning is a separate step.
- Read front-to-back, or skip the data pass because the change "is only frontend". The constraint you
  didn't read is the one that breaks the plan.
- Generalize from a single file, or treat a folder name as evidence of what it contains.
- Describe the architecture the repo *should* have. This pass reports what is there.
