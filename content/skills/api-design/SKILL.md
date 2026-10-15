+++
name = "api-design"
description = "Use when designing or reviewing an API contract — REST, GraphQL, RPC, webhooks, events. Resources, methods, status codes, validation, error shape, idempotency, caching, versioning."
+++

# API Design

## When to use

Designing a new endpoint, service, or event contract — or reviewing an existing one for correctness,
consistency, and evolvability.

**Scope.** This skill decides the *contract*: what the API promises to callers. It never assumes a
language, framework, runtime, or architecture — those are the repository's decisions and live in its
own `AGENTS.md`. Apply the contract to whatever stack the repo already chose; when a framework idiom
conflicts with a rule here, name the conflict instead of silently bending either side.

## Method

1. **Name the consumers** and what each needs. A contract with no named consumer is speculation.
2. **Model the resource** — its identity, representation, and lifecycle (states and legal transitions).
3. **Specify each operation**: method, path, request schema, response schema, status codes, authz rule.
4. **Specify the failure modes** — every error a caller must handle, with its shape and status.
5. **Decide collections, concurrency, and idempotency** for the operations that need them.
6. **Decide evolution** — versioning strategy and what counts as a breaking change here.
7. **Write the contract artifact** (schema/IDL) and treat it as the source of truth, not a byproduct.

## Contract fundamentals

- **Model resources, not actions.** Nouns + verbs. When an operation genuinely isn't CRUD, model it as
  a resource anyway (a `refund`, a `transfer`, an `export-job`) rather than inventing an RPC verb path.
- **One naming convention, applied everywhere** — path casing, collection plurality, field casing,
  timestamp suffixes. Pick once, never mix. Inconsistency costs more than any single choice.
- **Stable, opaque identifiers.** Callers must not parse or infer meaning from an ID. Never expose a
  sequential internal key that leaks volume or enables enumeration.
- **Unambiguous scalars.** Timestamps as UTC in a single explicit format; money as minor units or
  decimal + currency code, never a float; enums as closed string sets, documented and additive-only.
- **Representations are explicit.** The same resource returns the same shape everywhere. Absent means
  absent — decide whether a field is omitted or `null` and hold that rule across the API.
- **Nesting is shallow.** Sub-resources only for genuine containment; otherwise link by ID and let the
  caller fetch. Deep nesting freezes today's hierarchy into a permanent URL.

## Operations

- **Correct status codes.** 200 · 201 (+ location of the created resource) · 202 accepted (async) ·
  204 no content · 400 malformed · 401 unauthenticated · 403 unauthorized · 404 · 409 conflict ·
  412 precondition failed · 422 semantically invalid · 429 rate-limited · 5xx *only* for real server
  faults. A handled business rejection is never a 500.
- **Safe means safe.** A read never mutates. GET with side effects breaks caches, retries, and prefetch.
- **Full replace vs partial update** are different operations with different semantics — replace
  requires the whole representation; partial update states exactly which fields it touches and what an
  explicit `null` means. Never let one endpoint silently behave as both.
- **Long-running work returns a job.** Accept, return a resource the caller can poll or subscribe to,
  and give that resource a terminal state and a failure reason. Never hold a request open for minutes.
- **Bulk operations declare their atomicity.** All-or-nothing, or per-item results with per-item status
  — stated in the contract, never left to the caller to discover.

## Collections

- **Always bounded.** Every collection paginates, with a documented default and maximum page size.
  cursor-based for large or changing sets; offset only for small, stable ones.
- **Filtering and sorting are a closed set.** Allow-list the filterable fields, comparison operators,
  and sort keys. Never pass caller-supplied expressions into a query engine.
- **Consistent envelope.** One shape for every collection: items plus paging metadata. Don't return a
  bare array from one endpoint and an object from another.
- **Total counts are optional and expensive.** Offer them only where they're cheap and actually needed.

## Errors

- **One error shape across the entire API**, e.g. `{ error: { code, message, details? } }`, with a
  machine-readable, stable `code` — callers branch on the code, never on the prose.
- **Field-level detail for validation failures**: which field, which rule. Report all failures at once
  rather than one per round-trip.
- **Messages are for humans, safe for strangers.** No stack traces, SQL, internal paths, hostnames,
  library names, or upstream payloads.
- **Errors carry the correlation ID** so a user's report maps to a server-side trace.
- **Retryability is explicit** — say whether the caller may retry, and after how long (`Retry-After`).

## Reliability & concurrency

- **Idempotency for unsafe retries.** Client-supplied idempotency key, scoped to the caller, with a
  stated retention window: same key + same payload → the original result, executed once; same key +
  different payload → conflict.
- **Optimistic concurrency on mutable resources.** Return a version/entity tag, require it on update,
  reject a stale one with a precondition failure. Last-write-wins is a data-loss decision, not a default.
- **Timeouts and payload limits are part of the contract** — maximum request size, maximum processing
  time, and what the caller sees when either is exceeded.
- **Every API is rate limited**, counted on two keys per limited route — the caller's IP and an identity
  (an account, a session, a network range) — and the limits are documented and observable: the limit,
  the window, and the headers that expose remaining quota and reset time.

## Security

- **Every endpoint states its authz rule before it ships.** Who may call it, and over which records.
- **Authorization is server-side and ownership-aware.** Never trust a client-supplied identity, role,
  tenant, or ownership claim, even on an authenticated request.
- **Reject unknown fields.** Parse input against a schema that fails on extra properties — this is the
  block for mass assignment. Validate types, formats, lengths, and ranges before any logic runs.
- **Never accept the fields the server owns** — identity, role, tenant, balance, status, timestamps,
  computed totals. If a caller can send it, assume a caller will.
- **Don't leak existence.** An unauthorized read of someone else's record answers like a missing one;
  keep enumeration, timing, and error-code differences from distinguishing the two.
- **Minimal responses.** Return what the consumer needs, not the whole record. Field-level redaction is
  part of the contract, not a serializer accident.
- **Transport and origin locked down** — TLS only, an explicit origin allow-list, credentials never
  paired with a wildcard origin.
- **Secrets never travel in a URL** — no tokens or keys in paths, query strings, or redirects; they end
  up in logs, referrers, and browser history.

## Caching & performance

- **Declare cacheability per endpoint** — cacheable and for how long, or explicitly not, with private vs
  shared stated for anything user-specific.
- **Support conditional requests** on cacheable reads (entity tag / last-modified) so repeat reads are
  cheap and updates stay safe.
- **Design against N+1 at the contract level.** If the natural client flow is "list, then fetch each
  one", either embed the needed fields or expose a batch read.
- **Let the caller choose the payload** where it matters — a documented field-selection or expansion
  parameter beats shipping every relation to every consumer.

## Evolution

- **Additive changes only within a version**: new optional fields, new endpoints, new enum values the
  caller is told to tolerate. Removing or renaming a field, tightening validation, or changing a status
  code is breaking — even when "no one uses it".
- **One versioning strategy, decided and documented** (path, header, or media type). Consistent beats
  clever.
- **Deprecation is a process, not a note**: announce, mark it in the response, give a sunset date,
  provide the migration path, then remove. Never silently drop a field.
- **Every route is in the contract artifact** (OpenAPI for HTTP), and the artifact and its UI are served
  only in development.
- **The contract artifact and the implementation are checked against each other.** Drifting docs are
  worse than no docs — a caller trusts them.

## Observability

- **Every request carries a correlation ID** — accepted from the caller when present, generated
  otherwise, echoed in the response and in every log line and error.
- **Log the contract, not the payload.** Route, status, latency, caller identity, correlation ID —
  never credentials, tokens, or personal data.

## Other API styles

Everything above holds; these add their own failure modes.

- **Graph APIs** — depth and complexity limits, per-field authorization (not per-endpoint), pagination
  on every list field, batching to kill N+1 resolvers, introspection off in production, and errors that
  don't leak the schema's internals.
- **RPC / IDL-based** — the schema is the contract: field numbers/tags never reused, fields deprecated
  rather than deleted, every method's error set enumerated, streaming methods stating their termination
  and back-pressure behavior.
- **Webhooks (outbound)** — signed payloads with a timestamp to block replay, at-least-once delivery so
  the receiver must dedupe by event ID, documented retry schedule with backoff, and a versioned event
  payload with the same evolution rules as above.
- **Events / messages** — an explicit schema with a compatibility policy, a stable event key, stated
  ordering guarantees, and a poison-message path. "The consumer will figure it out" is not a contract.

## Validation checklist

- [ ] Every operation has: schemas, status codes, error cases, and an authz rule.
- [ ] Input schema-validated with unknown fields rejected; server-owned fields unassignable.
- [ ] One error shape, stable codes, no internal leakage, correlation ID present.
- [ ] Collections bounded and paginated; filters and sorts allow-listed.
- [ ] Unsafe retries idempotent; mutable resources use optimistic concurrency.
- [ ] Caching, rate limits, timeouts, and payload limits stated.
- [ ] Versioning and deprecation path decided; contract artifact matches the implementation.

## Never do

- Ship an endpoint without its authorization rule defined.
- Trust client-supplied identity, role, tenant, or ownership.
- Return an unbounded collection, or a mutation behind a safe method.
- Expose internal errors, internal identifiers, or the difference between "forbidden" and "not found".
- Pair a wildcard origin with credentials, or put a secret in a URL.
- Make a breaking change inside an existing version, or remove a field without a deprecation window.
