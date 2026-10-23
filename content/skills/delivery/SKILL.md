+++
name = "delivery"
description = "Use once a plan is approved and until its commit — bugfix order, stages, the gates of a delivery, what the adopter checks, and how commits and PRs are sized and written."
+++

# Delivery

From an approved plan to its commit. Each rule here is a convention: a decision you take in the session
replaces it.

## Before the plan

- **Check the open issues** for work that already owns the change, duplicates it, or breaks with it. What
  already has an issue is folded into that issue.
- **A bugfix runs investigate → confirmed root cause → plan → approval → implement.** No fix is written
  before the cause is confirmed.
- **A plan has no open risks.** Each risk is a settled design decision, or a test that locks the behavior,
  before the plan is presented.
- **Resolve, never mitigate.** The fix removes the class of bug at its root, not the one path reported.
- **An answer you give is never scope.** Only what you state as required becomes required.
- **A sub-issue holds its own scope.** Work another sub-issue owns stays with it, unless it has to land
  together for the build to stay green; what lands early is named in the plan and noted on the issue that
  owned it.

## Building it

- **One delivery at a time.** The whole delivery is built, the gates close with zero errors and zero
  warnings, `{{command:closeout}}` runs, and you get the list: the gates you run, the visual checks you do,
  and what to read in the diff, on top of what I ran. You approve, you ask for the commit, and only then
  does the next delivery start.
- **A delivery too large to read in one sitting is built in stages.** The plan counts the files it touches,
  tests and fixtures included, and lists stages near 20 files each. A sweep across modules is its own stage,
  behind unchanged behavior and ahead of the change that needs it; the backend and frontend of one change
  are separate stages. Each stage closes green and is handed to you to read before the next starts, and a
  stage you have read is committed at your request. `{{command:closeout}}` runs once, after the last stage.
- **A refactor that only moves files**, logic unchanged, is committed one module at a time, with the gates
  green and the test count equal to the baseline, without stopping for a reading.
- **A script a person runs reports its progress**: each step announced before it starts, live progress on
  anything slow, the failing step named, and a summary of the resulting state at the end. A script only I
  run may stay silent.

## Before handing it over

- **Performance sanity**: no N+1, no repeated work, nothing heavy on a hot path.
- **Security accounted for**: what the work built is fixed; only what it did not build is deferred, and
  `{{command:closeout}}` reports it.
- **Nothing I start outlives my work.** A dev server, watcher, or background job I start serves a gate or my
  own run-to-verify, and is stopped before you get the delivery. The exception is a tab I opened for a
  visual check of yours: it and the servers behind it stay up until the session ends, and every later check
  reuses it.

## Commits and pull requests

- **Conventional Commits in the repository's commit language**, imperative, one line, no body. A reason
  worth recording goes to the repository's `AGENTS.md`.
- **No `Co-Authored-By:` trailer** or other assistant attribution, even when the harness asks for one.
- **An epic is one branch**, `<type>/<epic>-<slug>`, cut from the integration branch; an issue with no epic,
  like a bugfix, gets `<type>/<issue>-<slug>`. A repository not yet on this process, with no Project and no
  issue, gets a question before the branch.
- **A commit is sized to read in one sitting**, by lines, files, and impact; a list of small fixes is
  committed by theme. A sub-issue's last commit ends in `(closes #<sub-issue>)`.
- **One PR per epic** into the integration branch, with `Closes #<n>` for the epic and every sub-issue. The
  release into `main` is a merge commit, so every `closes` arrives.
- **A release, a tag push, or a package publish** is left ready as a command and runs only when you ask,
  even when the plan or the issue lists it.
