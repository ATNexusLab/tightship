# 0004. Documentation follows an order of standards

- Status: accepted
- Date: 2026-11-16

## Context

The docs the plugin asks an adopter to write follow standards picked one at a time. The SRS follows
ISO/IEC/IEEE 29148 and ISO/IEC 25010, the architecture arc42 and C4, the data model Chen and crow's foot,
the use cases and sequences UML, and the ADRs a template of our own. Nothing said which standard wins when
two cover the same doc, or what to use for a doc no standard was picked for. Issue
[#137](https://github.com/ATNexusLab/tightship/issues/137) asked for that rule.

Four of the skills define doc types: `project-docs`, `requirements-engineering`, `spec-writing`, and
`technical-writing`. A rule that lives in one of them reaches every adopter; one that lives only in this
repository does not.

## Decision

**A doc type follows one standard, chosen in this order:**

1. a global standard the industry already uses;
2. an ISO standard;
3. the convention of the ecosystem the doc belongs to, as Keep a Changelog is for a changelog;
4. a convention of our own, only where none of the above exists.

`project-docs` holds the table of every doc type and its standard. The other three skills link it and
never repeat it.

| Doc type | Standard | Rung |
| --- | --- | --- |
| SRS | ISO/IEC/IEEE 29148 and ISO/IEC 25010, with SemVer on the document | ISO |
| SRS changelog | Keep a Changelog 1.1.0 | ecosystem |
| Architecture | arc42 and C4 | global |
| Data model | Chen, crow's foot | global |
| Use cases, sequences, states | UML | global |
| Business processes | BPMN | global |
| Design tokens | the names and types of W3C DTCG Design Tokens 2025.10, in a Markdown table | global |
| Brand, screens | ours; a screen cites ISO 9241-161 for its components | own |
| Sprints | Scrum Guide 2020 | global |
| Roadmap | ours: milestones, then epics | own |
| Feature spec | ours, with its scenarios in Gherkin | own |
| ADR | Nygard, plus `Alternatives considered` | global |
| User-facing pages | Diátaxis | global |
| README, runbook | ours | own |

Each row was checked against what the market and ISO offer for that doc type:

- **SRS:** ISO is the rung that applies, since no industry template is used more widely than 29148. IEEE
  830 was replaced by it.
- **Architecture:** arc42 and C4 come before ISO/IEC/IEEE 42010. That standard says what an architecture
  description must contain but gives no section structure.
- **Design tokens:** the DTCG format reached its first stable version, 2025.10, in October 2025. It is a
  W3C Community Group report, not a W3C Recommendation; Microsoft's Fluent publishes its tokens in it and
  Style Dictionary reads it. The docs take its names and types and keep the values in a Markdown table,
  since no tool here reads token files.
- **Brand and screens:** no standard was found for a brand book or a screen specification. ISO 9241-161
  sets requirements for UI components, not a format for documenting a screen.
- **Roadmap:** no standard was found. Timeline and Now/Next/Later are vendor conventions, and the
  milestones on GitHub are already this roadmap's axis.
- **Feature spec:** PRDs, design docs, and RFCs differ from one company to the next, ISO/IEC/IEEE 15289
  names document kinds without a template, and IEEE 1016 is inactive. Only the scenarios have a standard,
  Gherkin. The API profile keeps its tables rather than an OpenAPI fragment.
- **ADR:** Nygard's format is the original and the one used most. MADR 4.0 adds weighed options at the
  cost of length, and a 2026 comparison of ADR templates scored Nygard's best overall.
- **User-facing pages:** Diátaxis is used in hundreds of documentation projects, Cloudflare's and
  Gatsby's among them. ISO/IEC/IEEE 26514 covers the process of writing user information, not the shape of
  a page.
- **README and runbook:** Standard Readme targets open-source libraries, and runbook templates are each
  vendor's own.

## Consequences

- Every doc type an adopter writes names its standard, and a new doc type has a rule to pick one.
- A design token is named by its DTCG path, such as `color.accent`, and `tokens.md` stays the one table
  of values the site shows.
- The SRS changelog moves to Keep a Changelog's headings. Its entries keep their text.
- The sprint template uses the Scrum terms: Sprint Goal, Product Goal, and Definition of Done.
- A user-facing page in `docs/` is one Diátaxis type, and its folder's index names it.
- Where the rule lands on our own convention, a standard published later replaces it through a new ADR.

## Alternatives considered

- **A standard per doc type, with no order.** Rejected: it is how the docs got here, and it leaves the
  next doc type with no rule.
- **ISO first, then the industry.** Rejected: it would put ISO/IEC/IEEE 42010 ahead of arc42 and leave the
  architecture with no section structure, and ISO has nothing for most of the other doc types.
- **The tokens as DTCG files**, a `.tokens.json` per theme and a `.resolver.json`. Tried and rejected:
  the tokens page on the site lost its values, a color read as three sRGB fractions instead of its hex,
  and with no tool reading the files they were one more copy to keep in step.
- **A generator for the tokens, such as Style Dictionary.** Rejected: it would bring Node into a
  repository whose scripts are Rust, for a handful of values copied by hand.
