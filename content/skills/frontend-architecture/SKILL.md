+++
name = "frontend-architecture"
description = "Use when structuring UI code — component boundaries, state ownership, where data enters, the server/client split, forms, async surfaces, accessibility. Framework-agnostic."
+++

# Frontend Architecture

How the UI is built and how it behaves.

**Scope.** Visual craft — palette, type, layout, motion, copy voice — is `frontend-design`. Measuring
what it costs is `{{skill:performance-analysis}}`. Framework, router, and data library are the repository's
decisions and live in its own `AGENTS.md`; nothing here names one.

Written for browser UI: the URL section and bundle cost assume the web. Everything else — state
ownership, data entry, forms, async surfaces, contract types, accessibility — holds on native too.

## A screen

- **Mobile-first.** The layout is decided at phone width and only then expanded. A control added to an
  existing row gets a place at 390px before anything else.
- **A screen is structured before it is filled.** Each area gets its responsibility, its planes, framing,
  proportions, and depth, and a color atmosphere; components dropped on a white background are not a
  layout. No section is pure white: backgrounds alternate light brand tones and solid bands of the main
  color, white is for card surfaces with a shadow, and the first fold carries a strong brand color. The
  strong secondary color stays on accents.
- **Assets come from the repository first.** An existing image, icon, SVG, or logo is searched for before
  one is sourced or drawn. Files you hand over are all used, and your originals are never moved or deleted.

## Component boundaries

- **Small, single-purpose, composable.** One component does one thing.
- **Separate what renders from what decides.** Presentational components take props and return UI;
  logic and data live above them, in a composable unit — never in a base class, never in a deep leaf.
- **Props are a contract** — typed, minimal, no boolean-flag soup. Prefer composition (slots, children)
  over configuration: a component that grew five booleans wanted to be three components.
- **A component that fetches, decides, and renders is three responsibilities** wearing one name. Split
  it the moment the second one needs a test.

## State — five owners, pick the narrowest

Every piece of state has exactly one owner. Naming it is most of the architecture.

| Owner | What belongs to it |
| --- | --- |
| **Server cache** | Remote data. Fetched, cached, invalidated — never copied into anything else. |
| **URL** | Anything that must survive a reload or be shareable: filters, tab, page, selection, sort. |
| **Form** | In-flight edits, until submit resolves. |
| **Component-local** | State nothing outside this component reads. |
| **Global client** | Only the genuinely cross-cutting: session, theme, notifications. |

- **Derive, don't duplicate.** Compute from the source instead of syncing a copy; two copies of one
  truth drift, and the bug appears in whichever one you weren't looking at.
- **The most common damage is remote data copied into global state** — now the cache and the copy
  disagree, and invalidation fixes only one of them.
- **The second most common is view state trapped in a component** when it belonged in the URL. That
  single mistake breaks the back button, reload, and sharing at once.

## The URL is part of the architecture

- **The reload test:** refresh the page — does the user land where they were? **The link test:** send
  the URL to someone else — do they see the same view?
- Filters, pagination, sort, active tab, selected record, and open detail panel all pass or fail those
  two tests. If they fail, that state is in the wrong place.
- **What never goes in the URL:** secrets, tokens, personal data, and genuinely ephemeral UI state
  (hover, a transient toast). URLs are logged, shared, and kept in history.
- **Back and forward are a contract with the user**, not an implementation detail of the router.

## Where data enters

- **Fetch at the route level by default.** A component that fetches its own data chains a round trip
  behind whatever rendered it — many small independent fetches become a serial waterfall, and the page
  is as slow as the deepest chain.
- **The cache key is the identity of the request.** Every parameter that changes the response belongs
  in it, or two different requests share one entry.
- **A mutation invalidates what it affects, explicitly.** "It'll refetch eventually" is a stale screen
  the user is about to act on.
- **An optimistic update needs its rollback written in the same change.** Without it you have a bug
  that looks like speed — the screen says it worked and the server disagreed.
- **Handle the request states where the data enters**, not in every consumer downstream.

## Server and client

- **The axis is what renders where, and why.** Whatever the framework calls it, the decision is the
  same: server-rendered by default, client interactivity added deliberately where it's needed.
- **Push the boundary as low and as late as possible.** An interactive leaf should not force its whole
  ancestry to become client code.
- **Secrets and heavy work stay on the server side of the boundary** — anything past it is public.
- **Hydration is a cost, not a free upgrade.** Every interactive island ships code and re-executes work
  the server already did; measuring that cost is `{{skill:performance-analysis}}`.

## The trust boundary

- **Client validation is UX; the server is enforcement.** Every rule that exists only in the interface
  does not exist. Duplicating a rule on both sides is fine and expected — treating the client copy as
  the guarantee is not.
- **Permission-driven rendering is convenience, not authorization.** Hiding a button hides nothing:
  the endpoint behind it is what must refuse.
- **No secret reaches client code**, including config variables that get inlined into the bundle. What
  ships to the browser is public by definition, and an environment value is never read to find out what
  it holds.
- **Data from elsewhere is input, including your own backend's.** Anything that becomes markup, a URL,
  or a redirect target is untrusted until it is treated as such.

## Forms

Where most real UI bugs live, and the cheapest ones to prevent.

- **One source of truth per field.** A controlled value plus a separate copy is a race with the user.
- **Validate on blur and on submit, not on every keystroke.** Errors that appear while someone is still
  typing their email teach them to ignore errors.
- **Guard the double submit at the handler**, not only by disabling the button — the second submit
  arrives from the keyboard, the slow network, or the impatient double-click regardless.
- **Errors go next to the field they belong to**, plus a summary when the form is long enough to scroll
  the first error off screen. Move focus to the first error.
- **A failed submit preserves everything the user typed.** Losing a filled form to a server error is
  the worst cheap bug there is.
- **Every field autofills correctly.** Each input carries the `autocomplete` token and a `name` browsers
  recognize; a field that is not personal data gets `autocomplete="off"`. A field with no standard token,
  like a house number or a neighborhood, is checked in the browser, never assumed.

## Async surfaces

- **Four states, always: loading, empty, error, and partial.** Empty is not loading — "no results for
  this filter" and "still fetching" look identical to the user unless you make them different.
- **An error state carries a way out** — retry, go back, or contact — never a bare message.
- **Failure is isolated.** One widget that fails to load must not blank the page around it; the rest of
  the screen stays usable.
- **A loading state occupies exactly the space of the final content.** The skeleton changes appearance,
  never size: the real component is its shell, and CLS stays at 0.

## Contract types

The types describing the backend's responses are either **generated from the backend's own schema** or
**hand-written and therefore a drift source**. There is no honest third option. Pick one deliberately
and record it — a hand-copied type that silently stopped matching is a runtime error wearing a
compile-time disguise.

## Accessibility (non-negotiable)

- **Semantic elements first; ARIA only to fill a real gap.** A native control brings keyboard, focus,
  and announcement for free — a rebuilt one owes all three.
- **Keyboard-navigable with visible focus**, and no focus trap that a keyboard cannot leave.
- **Manage focus on route change and on open/close** of any overlay. Focus left behind on a removed
  element strands a keyboard user with no position.
- **Every interactive element labeled; images have alt; color is never the only signal; contrast meets
  WCAG AA.**
- **Announce async results.** A screen that updated silently did not update for everyone.

## Verifying a screen

- **A delivery that changes a screen is verified in the browser before its commit**, on the session you
  logged in; I never type a password. The effect is checked in the database when there is one. Visual
  verification is never saved for the end of a phase.
- **Phone width is tested in the DevTools device toolbar you turn on**, in the tab you hand me, never
  simulated: no new tab, no window resize, no injected iframe, no CSS override. The phone is checked before
  the desktop.
- **Autofill is checked with a fictitious saved profile**, filling from the name field in Chrome, whose
  DevTools Autofill panel shows each field's predicted type. Safari on iOS goes on your list of visual
  checks.
- **Verification never changes real data.** A fictitious record is created, tested on, and deleted; a real
  record touched by accident is restored at once.

## Validation checklist

- [ ] Every piece of state has exactly one named owner among the five.
- [ ] Remote data lives in the server cache and is not copied elsewhere.
- [ ] The reload test and the link test pass for filters, tabs, pagination, and selection.
- [ ] Data enters at the route level; no accidental fetch waterfall; mutations invalidate explicitly.
- [ ] No rule enforced only on the client; no secret past the server/client boundary.
- [ ] Forms: one source per field, double-submit guarded, input preserved on failure.
- [ ] Every async surface has loading, empty, error, and partial — and failure is isolated.
- [ ] Keyboard, focus management, labels, and contrast covered.
- [ ] Contract types generated, or hand-written with the drift risk explicitly accepted.

## Never do

- Put data-fetching or business rules in a deep presentational component.
- Copy remote data into global client state, or keep two copies of one truth.
- Leave shareable view state out of the URL, or put a secret into it.
- Treat client-side validation, or a hidden control, as enforcement.
- Ship a secret, key, or token in anything the browser downloads.
- Ship an async surface missing empty or error, or an interactive UI that a keyboard can't use.
- Lose the user's input to a failed submit.
