+++
name = "database-design"
description = "Use when modeling data, writing a schema or a migration, or designing queries — keys, constraints, NULL semantics, tenancy, indexes, transactions, safe migrations. Engine-agnostic."
+++

# Database Design

## When to use

Designing a schema, adding or altering tables/collections, writing a migration, choosing keys or indexes,
or designing the queries that read and write the data.

**Scope.** This skill decides the *model and the guarantees*: what the data means, what must always be
true, and what a change costs. Engine, ORM, and migration tooling are the repository's decisions and live
in its own `AGENTS.md`. `NULL` semantics, transactions, and constraints are concepts here, not syntax —
express them however the chosen engine does.

## Method

1. **List the access patterns first.** Every read and write the application will actually perform, with
   its filters, sort, and expected volume. A model designed without them is a guess.
2. **Name the entities and their relations** — cardinality, optionality, and which side owns the link.
   A structural model takes its shape from the product's functional source and is born complete; only
   its content grows slice by slice. "How many exist today" is the wrong question — "which shape does
   no future slice force us to redo" is the right one.
3. **State the invariants** — what must be true of the data at all times, independent of any code path.
4. **Choose identity and types** — keys, then the exact type of each attribute. Both are expensive to
   change later; get them right before anything is written against them.
5. **Index for the patterns from step 1**, not for every column that appears in a filter.
6. **Design the migration path** — how the change reaches an existing database with live data and running
   code, and how it is undone.
7. **State the operational plan** — backup and restore, seeds per environment, what to watch after it ships.

## Identity & keys

- **Prefer a surrogate key** for the primary key, and let a natural key be a unique constraint alongside
  it. Natural keys change — the "immutable" business identifier turns out to be editable in year two.
- **Ordered or random is a performance decision.** A time-ordered key keeps inserts local in the index;
  a fully random one scatters them, and the cost shows only once the table is large. Pick deliberately.
- **An exposed identifier is part of the contract.** Once a key appears in a URL or a payload, callers
  depend on it — it must be opaque and non-enumerable, and it cannot be renumbered later.
- **A join table's key is the pair.** Make the natural composite the primary key or a unique constraint —
  otherwise the same relation is stored twice and nothing prevents it.
- **Foreign keys declare their on-delete behavior explicitly.** Cascade, restrict, or nullify is a domain
  decision about what deleting the parent *means*, not a default to accept silently.

## Shape & types

- **The narrowest type that fits the domain.** Money as integer minor units or exact decimal with an
  explicit currency, never a float. Timestamps with an explicit zone, stored as an instant, never naive
  local time. A date without a time is a date, not a midnight timestamp.
- **`NULL` means unknown, not empty and not false.** Three-valued logic is where guards fail silently:
  a predicate that yields `NULL` is neither true nor false, an `AND` with it is `NULL`, and inside a
  condition it simply does not enter — no error, no log. Worse, in an `OR` it *washes out* a `false`.
  Make every predicate total: handle the unknown branch explicitly rather than assuming a comparison
  returns a boolean. Prove it with a test asserting `false`, not merely "not true" — a truthiness check
  passes on `NULL`.
- **Nullable is a claim about the domain.** Each nullable column must answer "what does absent mean here?"
  If the answer is "it shouldn't happen", it is `NOT NULL` with a default or a backfill, not a hope.
- **Closed value sets: pick the mechanism by how often it changes.** A constraint on allowed values is
  cheap to read and costly to extend; a lookup table is a join but lets values evolve as data. Values the
  business will add belong in a table; values the code branches on belong in the schema.
- **A JSON column is legitimate for genuinely open payloads** — external responses, user-defined fields,
  event bodies. It is a smell when its keys are queried, filtered, or joined: that is a schema that was
  not decided, and it carries no constraints, no types, and no foreign keys.
- **One naming convention**, applied to tables, columns, keys, and indexes alike. Consistency beats any
  particular choice.

## Constraints & invariants

- **An invariant that must always hold belongs in the database.** `NOT NULL`, `UNIQUE`, foreign keys,
  and value checks are enforced against every writer — including the migration, the console session, and
  the job nobody remembered.
- **Enforced once, not everywhere.** A rule guaranteed by a constraint does not need re-checking above it;
  a rule enforced only in application code is one background script away from being violated. Duplicating
  it in both places means two implementations that will eventually disagree.
- **Application-level checks are for what the database cannot express** — a rule spanning a request, an
  external system, or the user's permissions.
- **Derived and denormalized values need a stated owner.** Counters, totals, and cached rollups either
  come from a constraint the engine maintains or from exactly one code path, documented — never from
  "wherever we remembered to update it".

## Lifecycle

- **Record when and by whom.** Creation and update instants on anything mutable; the actor as well
  wherever a dispute is possible. This is the cheapest debugging tool that exists.
- **Soft delete is a real cost, not a free undo.** A deleted flag breaks every unique constraint (they
  must become partial or include the flag), leaks into every query that forgets it, and grows forever.
  Use it where history is genuinely required; use a hard delete where it is not.
- **Retention is designed, not discovered.** Say how long each category of data is kept and what removes
  it. Data with no expiry is a permanent liability.
- **The best handling of sensitive data is not storing it.** For what must be stored: keep it in one
  place, so that erasing it is one operation and not a search — and remember that references, backups,
  and derived copies are part of what erasure has to reach.

## Access & isolation

- **Multi-tenant data carries its tenant on every row**, and the isolation is enforced in one place —
  a row-level rule in the engine, or a single data-access chokepoint no query bypasses. Isolation that
  depends on every developer remembering a `WHERE` clause is not isolation.
- **The application's user is not the schema's owner.** Grant only the privileges the running code needs;
  schema changes use a different, more privileged identity.
- **A row-level rule must fail closed.** Any path where the caller's identity is absent — a service-level
  connection, an internal call, a scheduled job — must be denied explicitly, not left to a comparison
  that yields `NULL`.

When the question stops being how to model the access and becomes how it is attacked — which caller
defeats the rule, what a leaked credential reaches, what the replicas and exports carry — continue in
`{{skill:security-audit}}`.

## Indexing & queries

- **Index the access patterns, not the columns.** Composite index order follows equality first, then
  range, then sort. An index no query matches is pure write cost.
- **Read the execution plan before claiming an index works.** A plan that still scans, or ignores the new
  index, is the answer — not the assumption that adding it helped.
- **Every foreign key on a large table is indexed** — otherwise each parent delete or update scans the child.
- **Keyset pagination for large or changing sets.** Offset pagination re-scans everything it skips and
  silently shifts rows when the data changes underneath.
- **Design against N+1 in the model.** If the natural flow is "fetch the list, then fetch each row's
  relation", the fix is a join, a batched fetch, or a different shape — decided here, before the loop exists.
- **Never build a query by concatenating input.** Where dynamic structure is genuinely needed — a sortable
  column, a table name — the identifier comes from an allow-list, never from the caller's string.
- **Bounded results always.** No unbounded fetch into memory, and no select of columns the caller does
  not use on a hot path.

When the question stops being how to model the access and becomes how slow it actually is — counting
queries, reading a plan at real volume, separating lock and pool waits from query time — continue in
`{{skill:performance-analysis}}`.

## Transactions & concurrency

- **A transaction wraps an invariant, not a function.** Its boundary is the set of writes that must all
  hold or none — no wider, and never narrower than the rule requires.
- **Choose the isolation level deliberately.** The default is a choice someone else made; know what
  anomaly it permits and whether this operation can tolerate it.
- **Read-modify-write needs a guard.** Either a version column checked on update (the caller retries on
  conflict) or a lock taken before the read. Reading, deciding in application code, and writing back is a
  lost update waiting for concurrency.
- **Acquire locks in a consistent order** across the codebase. Deadlocks come from two paths taking the
  same two rows in opposite orders.
- **Never hold a transaction open across external I/O.** An HTTP call inside a transaction holds locks
  for as long as someone else's service is slow.

## Migrations & operations

- **Expand → migrate → contract.** Add the new, backfill it, switch reads and then writes, and only then
  remove the old — never a destructive change in one irreversible step against live data.
- **Schema and code deploy separately, so version N of the schema must serve version N−1 of the code.**
  Every migration is written to be compatible with the application currently running against it. This is
  what makes a rename impossible in one step: add the new column, write both, migrate readers, drop later.
- **Backfills are batched and interruptible** on large tables, with a way to resume and a way to tell how
  far they got.
- **Know what each step locks and for how long.** An operation that is instant on a small table can hold
  a write lock for minutes on a large one — check before running it against real volume.
- **Every migration has a rollback plan**, even when the rollback is "restore and replay". State it before
  running, not after something fails.
- **An untested backup is not a backup.** A restore is rehearsed before any destructive step ships, and
  the recovery point and recovery time are known numbers, not assumptions.
- **Seeds are per environment and idempotent** — the minimum reference data the system needs to boot,
  never a snapshot of someone's local state.

## Document & key-value stores

The relational sections above still apply to identity, lifecycle, tenancy, and migrations. These are the
differences that bite.

- **Model by access pattern, not by domain shape.** Without joins, the document's structure *is* the query
  plan. Design the reads first and let them dictate the layout.
- **The partition/shard key is effectively permanent** and decides both distribution and which operations
  stay cheap. A key that concentrates traffic on one partition is a rewrite, not a tuning exercise.
- **Denormalization is a copy, and copies drift.** For every duplicated field, name what reconciles it and
  how stale it may be. "It'll be updated together" is not a mechanism.
- **Atomicity usually stops at the document.** An invariant spanning two documents is not guaranteed —
  either it fits in one document, or the model must tolerate it being temporarily false, deliberately.
- **Unbounded growth inside a document is a design error.** Arrays that append forever hit a size limit
  and make every read of that document more expensive. Split before that.
- **Schemaless still has a schema** — it just lives in the code. Version it, validate on write, and state
  how old shapes are read.

## Validation checklist

- [ ] Access patterns listed before the model; indexes match them and the plan confirms it.
- [ ] Keys chosen deliberately: surrogate PK, natural key constrained, exposed IDs opaque.
- [ ] Invariants enforced by constraints, once — not duplicated in application code.
- [ ] Every nullable column has a stated meaning; no predicate can silently yield `NULL`.
- [ ] Types exact: money, instants, and closed value sets modeled correctly.
- [ ] Tenancy and privileges enforced at one chokepoint that fails closed.
- [ ] Concurrent writes guarded (version or lock); transactions wrap invariants and no external I/O.
- [ ] Migration is N−1 compatible, staged, batched, lock-aware, with a rollback plan and a rehearsed restore.
- [ ] Retention stated; sensitive data minimized and erasable in one place.

## Never do

- Run a destructive change against live data without a staged plan and a restore you have actually tested.
- Ship a guard or row-level rule that can return `NULL` instead of `false`.
- Concatenate input into a query, or take an identifier from the caller without an allow-list.
- Rename or drop in the same deploy that changes the code reading it.
- Store money as a float, a timestamp without its zone, or a secret in a column.
- Leave a foreign key unindexed on a large table, or a collection growing without a bound.
