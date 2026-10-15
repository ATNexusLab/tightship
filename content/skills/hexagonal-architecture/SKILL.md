+++
name = "hexagonal-architecture"
description = "Use when structuring an app with real domain logic — business modules with hexagonal layers inside, the dependency rule, the port catalog, inbound vs outbound adapters. Not for thin CRUD."
+++

# Hexagonal Architecture (module-sliced)

Organize the code by **business module**, and inside each module apply the hexagon (domain / application
/ infrastructure). The top level reads as the business; the inside enforces the dependency rule. Compose
with `{{skill:architecture-reading}}` — map an existing codebase before restructuring it.

**Scope.** This skill names roles, never technologies. The language, the framework, the ORM, the
validation library, and the file-extension convention belong to the repository's own `AGENTS.md`.

## When to use

- The app has real domain rules, multiple cohesive business areas, and is expected to live and grow.
- You want changes to stay local: replacing the database, the transport, or one module touches one place.

## When NOT to use

Thin-domain CRUD, small/few-endpoint services, prototypes. There the full hexagon is **over-engineering** —
folders and indirections per feature before any logic exists. Use a flat structure and revisit later.

## The structure (placeholders, not real names)

```text
modules/
├── <module>/
│   ├── domain/                 # pure rules; imports NOTHING external
│   │   ├── <name>.entity
│   │   ├── <name>.value-object
│   │   └── <module>.errors
│   ├── application/
│   │   ├── <verb>-<name>.use-case      # the verbs of this module
│   │   ├── <module>.dto                # command/result shapes — no transport
│   │   └── ports/
│   │       ├── <module>-repository.port
│   │       ├── clock.port
│   │       └── payment-gateway.port
│   ├── infrastructure/
│   │   ├── inbound/            # who drives the app
│   │   │   ├── <module>-http.adapter
│   │   │   └── <module>.schema         # wire validation
│   │   └── outbound/           # what the app drives
│   │       ├── <module>-repository.<tech>.adapter
│   │       ├── payment-gateway.<tech>.adapter
│   │       ├── clock.system.adapter
│   │       └── <module>.mapper
│   └── <module>.wiring         # composition root + public surface
└── <module>/ …

shared/
├── kernel/   # cross-module DOMAIN types + the Result shape
└── infra/    # cross-module INFRA: connections, middleware, transport errors
```

The boot file registers each module and starts listening — wiring only, no logic.

## Naming: two axes, both in the name

The **word** for a role is fixed. How the word attaches to the file name follows the language —
dotted suffix, hyphenated suffix, CamelCase, or a folder. Pick one mechanism per repo and never mix.

| Role | Word | Where it lives |
| --- | --- | --- |
| Pure rule, with identity / without | `entity` · `value-object` | `domain/` |
| The business verb | `use-case` | `application/` |
| **The need** (an interface) | **`port`** | `application/ports/` |
| **The technology that satisfies it** | **`adapter`** | `infrastructure/inbound/` · `outbound/` |
| Translation between two shapes | `mapper` | beside its adapter |
| Wire validation · internal shape | `schema` · `dto` | `inbound/` · `application/` |

An adapter carries the technology in its name (`<capability>.<tech>.adapter`); a port never does.
That is the whole trick: `grep '\.port'` returns every need the app has, `grep '\.adapter'` returns
every technology currently satisfying them, and swapping one is a search that ends in one folder.

**Acceptance test:** from a file name alone you can state its layer and its role — and, for anything in
`infrastructure/`, which side it sits on. If you can't, the name is wrong — fix the name, not the reader.

## The port catalog

A hexagon is not "a repository interface." Everything the application needs from the outside is a port:

- **Persistence** — `<module>-repository.port`. Speaks entities, never rows.
- **Time** — `clock.port`. The single most common reason a test can't be deterministic.
- **Identity** — `id-generator.port`. Same reason.
- **Events** — `event-publisher.port`. Fire-and-forget facts leaving the module.
- **External capability** — `payment-gateway.port`, `notifier.port`, `file-storage.port`.

Rules: **name the port after the capability, never after the technology** (`clock.port`, not
`system-time.port`); no `I` prefix. And a port whose single implementation has no plausible second —
not even a test double you actually need — is not a port, it's indirection. Delete it.

## Direction: the two sides

- **Inbound** — whatever drives the application: transport handlers, CLI commands, queue consumers,
  scheduled jobs. It translates a protocol into a DTO, calls **one** use case, translates the result
  back. No rules live here.
- **Outbound** — whatever the application drives: databases, queues, external services, the clock.
  It implements a port and knows nothing about who called it.

The use cases *are* the inbound port — the application's public verbs. That is why an inbound adapter
that reaches into `domain/` and skips the use case has broken the hexagon even though its imports
still point inward.

## Layer & dependency rules

- Exactly three layers per module. The arrow is `infrastructure → application → domain`, **never** backwards.
- Domain imports nothing external — no framework, no persistence, no transport, no other module's internals.
- **Wire validation lives in the inbound adapter.** The application speaks plain command/result DTOs;
  the transport contract never leaks inward, and no adapter type ever appears in a use case signature.

## Error model — one, decided once

What matters is that a predictable outcome (not found, invalid transition, rejected by a rule) is
distinguishable from a genuine fault, and that the whole repo does it the same way. Typed exceptions
and returned results both satisfy that; the language's own convention decides which, and the repo's
`AGENTS.md` records it. Two competing error models in one repo is worse than either.

Define the chosen shape once in `shared/kernel`. Whichever it is, the transport-level error type never
crosses into `application` or `domain`.

## Cross-module rule

A module may import another **only through its public surface** (`<module>.wiring`) — never reach into
another module's `domain/` or `infrastructure/`. Prefer referencing other aggregates **by ID**, not by
importing their entities. Escalate to domain events only when there's a real cross-module side effect.
Shared domain types belong in `shared/kernel`.

## Keeping it lean (this is the point)

- **No folder for a single file.** Start flat in `domain/`; promote to a subfolder (e.g. `value-objects/`,
  or `outbound/persistence/`) only at ~3+ of a kind. The tree grows into structure; it doesn't start there.
- **Tests co-located** (`<verb>-<name>.use-case` and its test side by side). No parallel test tree.
- **No empty placeholder files** for symmetry. A module without a value object has no such file.
- The per-module composition root keeps wiring out of one giant central file — but it's one small file,
  not a framework.

## Validation checklist

- [ ] Every file name states its layer and its role; every file in `infrastructure/` states its side.
- [ ] `grep` for the port word returns every external need; `grep` for the adapter word returns every
      technology. No file is ambiguous between the two.
- [ ] Ports are named by capability, live in `application/ports/`, and name no technology.
- [ ] Domain imports nothing external; the dependency arrow points only inward.
- [ ] Every inbound adapter goes through a use case — none reaches the domain directly.
- [ ] You can name where a new entry point, its logic, its data access, and its test go.
- [ ] No single-file folders; structure matches current volume.

## Never do

- Apply the full hexagon to thin CRUD — that's ceremony, not maintainability.
- Name a port after the technology behind it, or prefix it with `I`.
- Invent a port for a need with one implementation and no plausible second.
- Let an inbound adapter call the domain directly, or an adapter type appear in a use case signature.
- Put transport/validation shapes in `application`, or business rules in an adapter.
- Import another module's internals, or reference its entity instead of its ID.
- Sell this as the one true structure — it's a strong default with trade-offs, not a law.
