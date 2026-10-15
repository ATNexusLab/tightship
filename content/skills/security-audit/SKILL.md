+++
name = "security-audit"
description = "Use when auditing code or a diff for vulnerabilities, or hardening a security-relevant change — enumerate the surfaces it touches, then check each against its required threat cases."
+++

# Security Audit

Read-only review method: **enumerate the surfaces a change touches, then check each against its
required cases.** A surface with a missing case is a finding.

**Scope.** This skill names failures, never technologies. The language, the framework, the validation
library, and the scanner belong to the repository's own `AGENTS.md`.

## When to use

Reviewing a path or a diff for vulnerabilities; hardening any change touching authentication,
authorization, input, data, secrets, files, or the network. Backs the `sec` axis of `{{command:audit}}`.

## Method — how the surface list is built

The checklist below is worthless against a surface nobody listed. Enumerate before you check, in this
order, and write the list down:

1. **What the change touches.** Read the diff and the code around it — not only the changed lines. A
   changed function is a surface; so is every caller that now reaches new behavior.
2. **Who can reach it, and the least-trusted one.** Enumerate callers by what they pass, not by where
   they live. If an anonymous caller, another tenant, or a lower role can reach the same path, that is
   the caller you audit against.
3. **What data crosses it, and whose.** Anything belonging to someone other than the caller raises the
   bar on every surface below. Personal data, credentials, and money each carry their own cases.
4. **What trust it assumes.** Every "already validated upstream", "only called internally", and
   "the gateway handles that" is an assumption. Name it, then find the path that bypasses it.
5. **What happens when the layer before it fails.** A guard that only holds while the previous guard
   holds is one layer, not two — and a rule that fails **open** is not a rule.

State the enumerated list in the report. **A surface you did not name is one you did not audit**, and
that is a gap to declare, not silence.

## Surface → required cases

- **Authentication** — anonymous reaches a protected path → rejected; expired, forged, or replayed
  credential → rejected; the credential is verified, not merely decoded; sessions expire, rotate on
  login, and die on logout; a lost credential can be revoked.
- **Authorization** — checked per **resource**, not per route: another owner's identifier → denied
  (IDOR). Every axis the product gates on — role, tenant, plan, ownership, state — is checked, and the
  code cannot be made to skip one by omitting a field. Privilege never escalates through a body,
  header, or query parameter the caller controls. **The rule fails closed**: when identity is missing
  or ambiguous, the answer is deny, never allow. Denied and non-existent do not leak apart when the
  difference itself is sensitive.
- **Input** — validated against a schema at the boundary, rejecting unknown fields (mass assignment),
  with type, length, range, and encoding bounds. Injection: never build a query, a command, a path, a
  template, or markup by concatenating input — use the parameterized or escaping mechanism of that
  layer, and never hand input to a shell. Server-initiated requests to a caller-supplied address are
  allowlisted, with internal ranges and non-network schemes blocked; so are redirects.
- **Output & errors** — no secret, credential, token, or personal data in a response, a log, or an
  error. Nothing internal leaks outward: stack traces, queries, file paths, versions, or the shape of
  the infrastructure. An error is actionable to the caller without describing the system.
  Unverified input is never relayed verbatim to a third party the system reaches on the user's behalf;
  it is restricted or omitted.
- **Mutation & state** — a state-changing request cannot be forged by another origin. Ownership is
  re-checked on the server at the moment of the write, not read from what the client sent. Concurrent
  or duplicated requests are guarded: check-then-act is a race unless the check and the act are atomic
  or the record is locked. Retried and repeated calls are idempotent where they must be.
- **Data & persistence** — the running application's credential holds only the privileges it needs,
  never the schema owner's. A row-level rule fails closed for every path that reaches the data,
  including the ones that arrive without an end-user identity. Tenancy travels on the row and is
  enforced in one place, never re-derived per query. Sensitive data is minimized before it is
  protected: what is stored is stated, kept in one place, and has a retention that something actually
  executes. What holds for the primary holds for its backups, replicas, exports, and non-production
  copies. Continue in `{{skill:database-design}}` for the modelling itself — here the question is only how that
  model is attacked.
- **Files & uploads** — type decided by content, never by the name or the declared type; size bounded
  before the payload is consumed; the stored path derived from a server-generated name, never from
  caller input; nothing uploaded is ever executed or interpreted by the server, and what is served
  back is served in a way that cannot execute in a viewer's context.
- **Network & transport** — transport encrypted end to end, including between internal services when
  the network is not trusted. Cross-origin access is granted to named origins, never to any origin
  when credentials travel. The response carries the headers that constrain what a browser may do with
  it. Inbound calls from third parties are authenticated by signature, compared in constant time, and
  bound to a time window so a captured call cannot be replayed. Abusable endpoints are rate-limited by
  at least two independent keys, never one: the network (IP, IPv6 grouped by prefix) and an identity —
  the account or e-mail in the body on login and sign-up, the session on an authenticated route, the
  address on a route that sends e-mail; a route with no identity uses a global ceiling as its second key.
  On login the account key counts only failures and blocks before the password hash, even with the right
  password.
- **Crypto & secrets** — randomness for anything security-bearing comes from a cryptographic source,
  never a general random generator. Standard, current algorithms with correct parameters; passwords
  through a slow, salted, purpose-built function, never a plain hash. Nonces and initialization
  vectors are unique per use. Secrets compared in constant time. Secrets are injected from the
  environment or a secret manager, never committed, never logged, never in a URL — and a leaked one
  can be rotated without a code change.
- **Dependencies & what runs in CI** — the lockfile is committed and the tree is free of known
  high-severity advisories; a newly added package is checked for a name that imitates another. What
  the pipeline runs is as privileged as the code: a workflow triggered by untrusted input must not
  receive the secrets, jobs hold the narrowest permission that works, and no secret reaches a step
  that echoes it.

**This list is the known failure modes, not the limit of what counts.** A real vulnerability that no
line here names is reported at its real severity — never downgraded for being off-list.

## Severity — what the attacker gets, over what they need

Rank on two axes: the **impact**, and the **precondition** the attacker must already satisfy. Impact
alone inflates every finding; a level with no stated precondition is a level with no meaning.

| Level | What it takes | What it gets |
| --- | --- | --- |
| **Critical** | Nothing, or any account | Remote execution, mass data access, authentication bypass, full takeover |
| **High** | Any authenticated account | Another user's or tenant's data, privilege escalation, destructive write |
| **Medium** | A specific role, or a chained condition | Limited exposure or damage, bounded by scope |
| **Low** | An unlikely position or a narrow window | Marginal effect, or hardening that is genuinely absent |

Each finding: `surface · scenario · precondition · concrete fix · severity`, with `path:line`. The
scenario says in plain words who does what and what goes wrong; a risk that cannot be written as one is
not a finding.

## What is not a finding

- **Intended behavior.** What a requirement, an ADR, the spec, or the issue states as intended. If the
  decision itself looks risky, raise it as a decision to review, citing its source — never as a bug.
- **A documented trade-off**, when the weakness is a deliberate decision with its mitigations written
  down. Where it is recorded is the contract's **Security** rule; `security-debt` is for what should be
  fixed and was not.
- **A missing second layer** where the first one genuinely closes the hole. Say the depth is thin;
  do not rank it as an exposure.
- **A theoretical weakness with no path to it.** No path, no finding.

## Deferral rule

Only what the work under audit did not build may be deferred; what it built is fixed. A deferred
finding is **never dropped**: it is reported, and becomes a GitHub issue labeled `security-debt`
(surface · risk · fix · severity) when the user says so. Critical and High are promoted, not parked.

## Never do

- Treat "no obvious bug" as secure — check the full case list of every surface you enumerated, and
  declare the ones you did not. Zero findings after checking every case is a valid result.
- Rank a severity without stating its precondition.
- Modify code during an audit — report only.
- Drop a finding: each one is fixed, or reported as deferred.
