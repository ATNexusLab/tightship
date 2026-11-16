+++
name = "production-artifact"
description = "Use when writing or changing anything that builds or ships a production artifact — a container image, a serverless bundle, a release binary, a package — so it carries only what runs in production."
+++

# Production Artifact

What a deploy runs carries what runs in production, nothing more. The commands that apply each rule come
from the stack the repository uses.

## Rules

1. The artifact carries the build output, the runtime dependencies, and the files the program reads at
   run time, and nothing else.
2. Development dependencies, build tools, tests, and docs stay out. Building and running are separate
   steps, and the running step receives only the build's output.
3. Whatever selects the files that enter the build, an ignore file or an include list, starts from
   nothing and admits what the build needs.
4. The agent applies each rule with the commands of the stack it finds, and checks the result by listing
   what the built artifact holds.

## Auditing

Loaded by the audit, each case is one finding under the title below, in the chat's language and with
nothing else changed:

| Case | Title |
| --- | --- |
| A development dependency reaches the artifact | "Development dependency in the production artifact: \<dependency\> (\<file\>)" |
| No ignore file or include list selects the build's files | "No ignore file: the build receives the whole repository (\<file\>)" |
| The ignore file excludes paths instead of admitting them | "Open ignore file: \<file\> excludes paths instead of admitting them" |
| One step both builds and runs | "One step builds and runs the artifact (\<file\>)" |
