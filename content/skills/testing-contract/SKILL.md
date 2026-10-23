+++
name = "testing-contract"
description = "Use before writing or changing a test — whether the change earns one, which existing test absorbs it, and how to write it so it locks real behavior and never flakes."
+++

# Testing Contract

Whether a test exists, which ones the suite holds, and how to write them. The runner, assertion style, and
file layout come from the repo's own `AGENTS.md` and its existing tests — match what is there.

## What earns a test

- **A test exists only when the change alters behavior someone observes and that can break silently** —
  logic, a business rule, data flow, authorization, a bugfix. Then it comes before the code. Color, style,
  layout, copy, renames, config, docs, wiring, and dependency bumps get no test, and no justification.
- **The suite holds the happy path of each operation, the denial on a protected one, and one test per
  real regression.** A speculative edge, a redundant case, or a generic guard re-tested per route: none.
- **Revise before adding.** The existing test that covers the behavior is modified; what is neither happy
  path nor regression is deleted. A new test only when no existing one can absorb the case.
- **Nothing outside the repo's runner**, which is the one its config sets, never invented. A repo without
  one gets no test file. Throwaway checks — mutants, verification scripts, self-checks — run in a scratch
  folder and never stay in the tree.

## Method

1. **Does the change earn a test?** No observable behavior that can break silently → no test, and move
   on. Nothing to state, nothing to justify.
2. **Find the test that already covers the behavior.** Modify it to catch the new case. While in that
   file, delete what is neither a happy path nor a real regression.
3. **Only if nothing can absorb the case, add one** — at the paths the repo's tests already use, with a
   name that states the behavior.
4. **See it red for the right reason** before writing the code.
5. Implement until green.

## A test executes the behavior

- **Assert the observable outcome, never how it was produced.** Internals change in a refactor; the
  contract does not.
- **Reading the source is not a test.** Matching a guard's name, a SQL fragment, or a string in a file
  survives any mutation that keeps the word. Call the code — a fake client, an extracted pure function.
  When only the text exists (a migration with no database in CI), match the whole predicate and count
  its occurrences.
- **A fixture comes from the real constructor, never an object literal.** A literal freezes a state the
  application may no longer produce — a field a form stopped writing keeps a rule satisfied in the test
  while every real entity is blocked.
- **A loop asserts its length first.** A `for` over zero items passes without testing anything.
- **Never assert on a mock.** If the fake satisfies the assertion with the real thing broken, it tests
  the setup.
- **A regression test is proven by the bug.** Reintroduce the historical form of the bug and watch the
  test die. A regression test never seen red locks nothing. Revert the mutant with an edit, never with
  `git checkout`, which also discards uncommitted work.

## Determinism — a flaky test is worse than no test

- **Time is an input.** Clock, time zone, and "now" are injected, never read inside the test.
- **Randomness is an input.** Seed it or inject it.
- **Order is not a fixture.** Each test sets up what it needs and leaves nothing behind.
- **No sleeping to synchronize.** Wait on the condition, not on a duration.
- **Fake what you do not own; use the real thing where it is the point.** The clock, an outbound call,
  a payment gateway are faked at the port. The database in a query test is the thing under test.

A flaky test is quarantined with a deadline or deleted, never re-run until green.
