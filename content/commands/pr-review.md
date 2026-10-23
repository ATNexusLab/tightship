+++
name = "pr-review"
description = "Review someone else's PR — why and how, what changed, verified concerns, and conflicts with other PRs"
argument-hint = "[PR number or URL] [--only sec|perf|arch|maint]"
+++

Review the PR: {{capability:arguments}}

This command **explains the PR before judging it** and **never publishes anything on GitHub**. The output
is a briefing in the chat, in the chat's language, for me to read and then comment on myself.
`gh pr review`, `gh pr comment`, `gh pr merge`, `gh pr edit` and `git push` are forbidden here, even if the
output seems to call for them.

## Phase 0 — prepare

1. Dirty tree (`git status --porcelain` not empty) → **stop** and say what is pending. Never stash it,
   never discard it.
2. `git fetch origin --prune`.
3. `gh pr view <n> --json number,title,body,author,baseRefName,headRefName,isDraft,commits,files,closingIssuesReferences,mergeable,mergeStateStatus,statusCheckRollup,url`.
4. Signs of an experiment — draft, days idle, no linked issue, automated author, a new isolated folder →
   ask in one line whether the PR is for real before auditing.
5. `gh pr checkout <n>`.
6. `BASE=origin/<baseRefName>` — the base is **the one the PR declares**, never an assumed `main`.
   `MERGE_BASE=$(git merge-base $BASE HEAD)`.
7. The review diff is **`git diff $MERGE_BASE...HEAD`** and nothing else. Nothing the base moved
   afterwards counts as the PR's authorship.

At the end of the command I am on the PR's branch — say so in one line, so I can run the application
right away. Any later work on the PR happens on that branch, in the repo directory, never on a scratch
branch or in a worktree in a temp folder; it lands as a commit on top plus a review comment, never as a
rewritten PR description or a force push to the branch under review.

## Phase 1 — understand

Read the description, the linked issues (`closingIssuesReferences`, read with
`gh issue view --json title,body,state,closedByPullRequestsReferences`) and the commit messages. This
feeds sections 1 and 2 of the output. A deduction from the diff is never presented as the author's
stated intent.

## Phase 2 — conflicts and overlap

1. **Merge state** — `mergeable` and `mergeStateStatus` from Phase 0.
2. **Base drift** — `git log --oneline $MERGE_BASE..$BASE -- <the PR's files>`: the files the base
   changed after the starting point.
3. **Open PRs on the same files** — `gh pr list --state open --json number,title,headRefName,files`,
   crossed by file with the PR's files. For each that crosses, read only the overlapping hunks of
   `gh pr diff <m>`.
4. **Already solved elsewhere** — a linked issue already closed by another PR
   (`closedByPullRequestsReferences`), or a commit in the drift that attacks the same goal.

This feeds section 4 of the output and the Phase 4 filter.

## Phase 3 — findings

Read this workflow's `{{command:audit}}` and dispatch its subagents — the same axes, lenses, conditional
lenses, intent reading and admission gate — **in parallel, in one message**, all with the same target:
`git diff $MERGE_BASE...HEAD`. `--only <axis>` restricts the run. Consolidation and verification are
Phases 4 and 5 below, not `{{command:audit}}`'s.

## Phase 4 — time filter

**No finding reaches the report without passing all three tests.** Time spent discussing what was
already handled is time lost.

1. **Already fixed on the current base** — did the finding's lines change between `$MERGE_BASE` and
   `$BASE`? `git log -L<start>,<end>:<file> $MERGE_BASE..$BASE`, with
   `git log -S'<snippet>' $MERGE_BASE..$BASE` as the net for code that moved. Read the change: if it
   covers the finding, discard it as *fixed in `<sha>` (PR #N)*.
2. **Already being handled in an open PR** — from Phase 2's crossing: if an open PR attacks the same
   point, discard it as *being handled in #M*.
3. **It is real** — each Critical, High and Medium finding that passed the first two tests goes to its
   own `{{agent:adversarial-verifier}}`, all in parallel in one message. What it refutes is discarded.

## Phase 5 — output

In the chat's language, short, no jargon the scenario does not explain. Four sections:

**1. Why and how** — up to 5 lines: the problem the PR solves and how it solves it. A claim deduced from
the diff, not stated by the author, is marked `[inferred]`.

**2. What changed** — one line per module or area, never a list of loose files. Then one line of risk
profile: files, layers crossed, and the sensitive surfaces touched (auth, authorization, migration,
money, env/secrets, upload, the server/client boundary).

**3. Concerns**

- CI: each failing or warning check from `statusCheckRollup`, by name — or "CI green".
- The `{{command:audit}}` triage table for the surviving findings, each tagged `in scope` (the PR caused
  it or made it reachable) or `adjacent` (pre-existing, only touched — it becomes an issue if I say so).
- One line: `Decisions to review: N · Low: N · Discarded: N`. Their detail comes only when I ask.

**4. Conflicts and overlap** — merge state, the base drift on the PR's files, the open PRs touching the
same files with one line on what each does there, and whether the goal was already solved elsewhere.
Nothing found → "no conflicts".

Close with a **recommended verdict** in one line: approve · request changes · talk first. The action on
GitHub is always mine.

Then walk the findings **one at a time**, like `{{command:audit}}`: **Finding 1 of N** — what happens · why it
matters in this PR · where (`path:line` with the snippet) · the fix. Stop for my call before the next.
A ready-to-paste review comment, in the language the PR's repo uses for its reviews, is written only
when I ask for it.
