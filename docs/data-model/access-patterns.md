# Access patterns

Every read and write the requirements call for. Volume: a vault of 1,000 notes
([NFR-PERF-01](../requirements/non-functional/performance.md#nfr-perf-01)).

| Operation | Reads / writes | Filter and order | When |
| --- | --- | --- | --- |
| Session start ([FR-MEM-01](../requirements/functional/memory.md#fr-mem-01)) | Reads pinned notes, the handoff, and the debts; appends one line to the session's environment file | Pinned in the project's scope or `all`, by `id`; the handoff of the project's scope; notes of the project's scope tagged `to-repo`, by `id` (M4, #118) | Every session start, `/clear`, and compaction, within 500 ms at p95 |
| Checkpoint ([FR-MEM-02](../requirements/functional/memory.md#fr-mem-02)) | Reads nothing | — | After each compaction; on every 10th prompt too until M4 (#118) |
| Search ([FR-MEM-03](../requirements/functional/memory.md#fr-mem-03)) | Reads notes | Terms in the `why`, tags, and body; the project's scope first, then `all`, then the rest with `--everywhere`; 10 at most | On demand |
| Write a note ([FR-MEM-04](../requirements/functional/memory.md#fr-mem-04)) | Writes one note | Into its scope's folder | On demand |
| Lint ([FR-MEM-05](../requirements/functional/memory.md#fr-mem-05)) | Reads every note | All | On demand |
| Sync ([FR-MEM-11](../requirements/functional/memory.md#fr-mem-11)) | Commits, rebases on the remote, renames and edits colliding notes, removes expired notes and rewrites the links to them, writes `AGENTS.md`, pushes | Notes added by unpushed commits whose `id` another note has; every note, for its lifetime (M4, #118) | On demand |
| Release a handoff ([FR-MEM-06](../requirements/functional/memory.md#fr-mem-06)) | Writes one handoff and removes the scope's others, then syncs | The project's scope | On demand |
| Claim a handoff ([FR-MEM-07](../requirements/functional/memory.md#fr-mem-07)) | Reads and removes every handoff of the scope, between two syncs | The project's scope; the latest `updated` first, then the highest `id` | On demand |
| Read the options ([FR-INS-03](../requirements/functional/install.md#fr-ins-03)) | A hook reads its option's state from the environment (Claude Code) or `${TIGHTSHIP_DATA}/options.toml` (Antigravity CLI, M3, #59) | By name; absent is on | Every session start |
| Turn on or put back the status line ([FR-STL-02](../requirements/functional/status-line.md#fr-stl-02), [FR-STL-03](../requirements/functional/status-line.md#fr-stl-03)) | Reads the user settings' `statusLine` and `status-line.json`; writes either on a change | The one key; the one record | Every `startup`, within 50 ms at p95 |
| Check for an update ([FR-INS-10](../requirements/functional/install.md#fr-ins-10)) | Reads `update.json`; writes `checked_at`, then the notice or the error, under `update.lock` | The one file | At most once a day, from the session start |
| Announce an update ([FR-INS-10](../requirements/functional/install.md#fr-ins-10)) | Reads and clears `update.json`'s `notice` | The one file | Every session start |
| Render the status line ([FR-STL-01](../requirements/functional/status-line.md#fr-stl-01)) | Reads the session's JSON on stdin and the repository's `HEAD` | — | On the harness's status line events |
| Change an option ([FR-INS-04](../requirements/functional/install.md#fr-ins-04)) | Claude Code writes one option's state in the user settings; in Antigravity CLI, the installer or the adopter updates `${PLUGIN_DATA}/options.toml` (M3, #59) | By name | On demand |
