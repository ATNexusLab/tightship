# Security policy

English · [Português](SECURITY.pt-BR.md)

## Supported versions

tightship is in pre-release. Only the latest published release gets security fixes.

## Report a vulnerability

Report it privately through
[GitHub's private vulnerability reporting](https://github.com/ATNexusLab/tightship/security/advisories/new).
Do not open a public issue, pull request, or discussion about it.

Include:

- the version, harness, and operating system
- what an attacker can do and what they need to do it
- the steps to reproduce it, or a proof of concept

## What happens next

The maintainer confirms receipt within 7 days. Together you agree on the severity and a date for the fix.
The fix ships in a new release, and the advisory is published when the release is out, crediting you
unless you ask not to be named.

## Scope

The plugin runs hooks and a binary on your machine, reads your vault, and writes to your harness's
settings. Reports about any of these are in scope:

- code execution or file access beyond what the [install guide](docs/install/claude-code/README.md)
  describes
- your identity, your secrets, or your vault's notes leaking into a build or a release
- the release's binaries or catalog not matching this repository's source

Vulnerabilities in a companion plugin (ponytail, i-have-adhd, humanizer) or in a harness belong to its own
maintainers.
