+++
name = "performance-analysis"
description = "Use when investigating slowness or checking a change for performance impact — set a target, measure at real volume, find where the time goes, fix at the cheapest rung."
+++

# Performance Analysis

## Core rule

**Measure before changing anything.** Optimization without a baseline is guessing. Never trade
simplicity for speed on a path that isn't proven hot.

**Scope.** This skill decides **evidence** — where the time actually goes, and whether a change helped.
*Shape* belongs to its neighbours: the model and its indexes to `{{skill:database-design}}`, contract
caching and payload shape to `{{skill:api-design}}`, component structure and state to
`{{skill:frontend-architecture}}`. Which profiler, which budget threshold, and which load tool are the
repository's decisions.

## What "slow" means

- **Percentiles, not averages.** A mean of 200 ms with one request in twenty taking 4 s is a broken
  experience that the mean hides. Users live in the tail: state p95 or p99, and say which.
- **Latency and throughput are different goals, and fixing one can cost the other.** Batching raises
  throughput and *raises* latency. Know which one the complaint is about before optimizing.
- **The number that matters is time to the user's next useful action**, not the duration in a server
  log. A 40 ms response that the client then blocks on for a second is a 1 s problem.
- **The target comes before the fix.** "Faster" has no stopping condition; "p95 under 300 ms at 200
  rows" does. Without a target you cannot tell a finished optimization from an abandoned one.
- **Spend effort where the time is.** Halving a stage that is 5% of the total buys 2.5%. Find the
  dominant term first — a large win on a small term is a rounding error.

## Method

1. **State the target** — the number, the percentile, and the layer the user actually feels it in.
2. **Reproduce at real volume.** Data size first, concurrency second. A bug invisible at 10 rows *is*
   the finding at 100 000, and a single request never reveals where the system collapses under load.
3. **Locate with one instrument, not five hypotheses.** A profile, an execution plan, or a waterfall
   answers "where" directly. Run it more than once — a single sample is not a measurement, and the
   instrument's own overhead is part of what you're reading.
4. **Fix at the cheapest rung** that meets the target (see the ladder).
5. **Re-measure against the same baseline, then lock it.** An unlocked fix comes back.

## The ladder — do less before doing it faster

Stop at the first rung that meets the target.

| Rung | What it is |
| --- | --- |
| **Don't do it** | Deleted work is the fastest work: a field nobody reads, an eagerly loaded relation, a call whose result is discarded. |
| **Do it later** | Move it off the critical path: background job, lazy load, streaming, pagination. |
| **Do it once** | Cache, memoize, batch. The real cost is invalidation and staleness — budget both, or the bug moves from slow to wrong. |
| **Do it closer** | Take the work to the data: one query instead of round trips, compute where the data already is. |
| **Do it faster** | The actual optimization. Last, because it is the only rung that buys speed with complexity. |

## Where the time goes

Four buckets with opposite fixes. Identify the bucket before proposing anything.

- **Busy (CPU)** — the process has work. A profile shows one dominant path. Fix: less work, a better
  algorithm, fewer allocations.
- **Waiting (I/O)** — blocked on network, disk, or another service. Fix: parallelize, batch, cache.
  Adding CPU does nothing at all here.
- **Contending** — serialized behind a lock, a mutex, a single connection, a single-threaded stage.
  Fix: shorten the critical section, order lock acquisition consistently, partition the contended thing.
- **Queueing** — the request waited before it started running. **As utilization approaches 1, waiting
  time goes to infinity** — which is why a system comfortable at 70% falls over at 85%, and why the fix
  is capacity or shedding load, never micro-optimization.

**The one-line test: is the resource busy, or waiting?** Those are opposite fixes, and the number that
separates them is utilization, not latency. Latency alone cannot tell you which bucket you are in.

## Frontend

- **Load and interaction are two problems.** First paint / LCP is a network and critical-path problem.
  Interaction latency / INP is a main-thread problem. Same page, unrelated fixes — say which one is slow.
- **Measure on a mid-tier phone with a throttled network**, not your laptop on fibre. The gap is an
  order of magnitude; every decision made on the laptop number is a decision made on fiction.
- **The waterfall is the diagnosis.** Read it before touching code: requests chained that could have
  been parallel, render-blocking resources, a font that delays text, a redirect nobody knew about.
- **Layout shift is a performance bug with a visual symptom.** CLS above zero means the reserved space
  is wrong — the fix is the loading state occupying the final content's box (see the global UI rule),
  not smoothing the movement.
- **Shipped is not the same as used.** Measure what parses and executes on the critical path, not total
  transfer size. Duplicated dependency, no code-splitting, a polyfill for browsers you don't support.
- **Render count is a number — get it before memoizing.** Memoization has a cost, and with unstable
  dependencies it is pure loss. Structure decisions live in `{{skill:frontend-architecture}}`; proving
  the need is here.

## Backend

- **Trace one request end to end with time per hop.** The distribution across hops is the finding; the
  total is only the symptom.
- **Serial I/O is the default bug.** Independent calls awaited one after another. Done concurrently the
  ceiling is the slowest one, not the sum — and most "slow endpoint" reports are exactly this.
- **Timeouts and retries amplify.** A retry storm turns a slow dependency into an outage. Every outbound
  call needs a timeout shorter than its caller's, bounded retries with backoff and jitter, and a way to
  stop calling entirely.
- **Work that doesn't have to happen inside the request shouldn't.** Mail, thumbnails, webhooks,
  analytics, search indexing — off the response path.
- **Payload size is latency.** Over-fetching, unbounded collections, and serializing objects to render
  three fields. The contract's shape is `{{skill:api-design}}`'s call; measuring its cost is this one's.
- **Memory pressure looks like CPU.** Collection pauses appear as latency spikes with no hot path in the
  profile. Check allocation rate before optimizing an algorithm that isn't the problem.

## Database

- **Count queries as a function of rows.** Run the same operation at n = 1, 10, 100 and count. Linear
  growth per row is N+1 — the most common backend killer, and invisible on a seeded dev database.
  Designing the model against it is `{{skill:database-design}}`; catching it at runtime is here.
- **Read the execution plan at production-like volume.** A plan on a near-empty table is fiction: the
  engine picks a scan because the table is small, and the index you added is never used where it counts.
- **The slow query log is the cheapest instrument that exists.** Turn it on before profiling application
  code — the answer is usually already sitting in it.
- **An unused index is pure write cost.** Check usage statistics, not existence. The reverse is a finding
  too: a hot filter with nothing supporting it.
- **Pool exhaustion looks like a slow database.** The request is waiting for a *connection*, not for the
  query. Measure wait-for-connection separately from query time — one is capacity, the other is the
  query, and they have nothing to do with each other.
- **Lock wait is not query time.** A fast query blocked for two seconds reports two seconds. Separate
  them before optimizing a query that was never slow.

## Budgets & regression

- **A numeric budget per critical path** (p95, query count, bundle size), enforced where a regression is
  actually caught — CI or a synthetic check. A baseline living in someone's terminal rots the next day.
- **A fix with no lock comes back.** The regression test asserts the *shape*: query count, render count,
  bytes on the critical path. Those are stable. Wall-clock milliseconds in CI are not — they fail on a
  noisy runner and get disabled within a month.
- **Field data tells you whether it matters; lab data tells you why.** Lab numbers alone optimize for a
  machine nobody uses.

## Validation checklist

- [ ] A target exists: a number, a percentile, and the layer it is felt in.
- [ ] Measured at production-like volume, more than once, before anything changed.
- [ ] The bucket is named — busy, waiting, contending, or queueing — before the fix was chosen.
- [ ] The dominant cost was fixed, not the most familiar one.
- [ ] The chosen ladder rung is the cheapest that meets the target.
- [ ] A before/after number on the same baseline proves the improvement.
- [ ] The gain is locked by a budget or a shape-based regression test.
- [ ] Simplicity preserved — the fix didn't add magic for a marginal gain.

## Never do

- Optimize without a measurement, or micro-optimize a cold path.
- Benchmark on toy data, or measure on your own machine and call it the user's experience.
- Propose a fix before naming which bucket the time is in.
- Reach for caching before checking whether the work can be deleted or deferred.
- Add caching or complexity the data doesn't justify — invalidation is a permanent cost.
- Ship an improvement with nothing locking it against regression.
- Report "it feels faster" as a result.
