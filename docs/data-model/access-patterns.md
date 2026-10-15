# Access patterns

Every read and write the requirements call for. Volume: a vault of 1,000 notes
([NFR-PERF-01](../requirements/non-functional/performance.md#nfr-perf-01)).

| Operation | Reads / writes | Filter and order | When |
| --- | --- | --- | --- |
| Session start ([FR-MEM-01](../requirements/functional/memory.md#fr-mem-01)) | Reads pinned notes and the handoff | Pinned in the project's scope or `all`; the handoff of the project's scope | Every session start, within 500 ms at p95 |
| Checkpoint ([FR-MEM-02](../requirements/functional/memory.md#fr-mem-02)) | Reads nothing | — | Every N prompts and before compaction, within 100 ms per prompt at p95 |
| Search ([FR-MEM-03](../requirements/functional/memory.md#fr-mem-03)) | Reads notes | Terms in the note; the project's scope first, then `all` | On demand |
| Write a note ([FR-MEM-04](../requirements/functional/memory.md#fr-mem-04)) | Writes one note | Into its scope's folder | On demand |
| Lint ([FR-MEM-05](../requirements/functional/memory.md#fr-mem-05)) | Reads every note | All | On demand |
| Release a handoff ([FR-MEM-06](../requirements/functional/memory.md#fr-mem-06)) | Replaces one handoff | The project's scope | On demand |
| Claim a handoff ([FR-MEM-07](../requirements/functional/memory.md#fr-mem-07)) | Reads and removes one handoff | The project's scope | On demand |
| Read the options ([FR-INS-03](../requirements/functional/install.md#fr-ins-03)) | A hook reads its option's state from the environment Claude Code sets | By name; absent is on | Every session start |
| Change an option ([FR-INS-04](../requirements/functional/install.md#fr-ins-04)) | Claude Code writes one option's state in the user settings | By name | On demand |
