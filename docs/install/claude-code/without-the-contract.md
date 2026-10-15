# Without the contract

With the `contract` option off, the session no longer loads the workflow's rules. Every skill and command
still loads and works. The ones below point to a rule of the contract instead of repeating it, so with the
contract off that rule is not in the session. Each falls back to what your repository's `AGENTS.md`
says, or to the agent's own judgement.

| Piece | Contract rule it refers to | What it does with the contract off |
| --- | --- | --- |
| `audit` | **Code I write**, for the maintainability axis; **Security**, for where a deliberate trade-off is recorded; the dev loop, for how a fix is made | Judges maintainability by the repository's `AGENTS.md` and common practice; names no place to record a trade-off; a fix follows no set order |
| `bootstrap` | The whole contract: the `AGENTS.md` it writes adds to the contract and never repeats it | Writes the same `AGENTS.md`, which leaves out everything the contract covered. Those rules then exist nowhere unless you add them to the repository |
| `closeout` | **Process**, for the chain from requirement to PR and for what may be deferred; **Git**, for branches, commits, and PRs | Checks traceability and security debt against the repository's `AGENTS.md` only; nothing defines the chain or what counts as debt |
| `epic` | **Git**, for the epic's branch, commits, and PR | Creates the epic and its sub-issues the same way; branch and commit naming come from the repository or the agent's default |
| `testing-contract` | **Testing**, which decides whether a test exists and what the suite holds | Still says how to write a test; nothing decides which tests to write |
| `security-audit` | **Security**, for where a deliberate trade-off is recorded | Still sets a written-down trade-off apart from a finding; names no place to record it |
| `frontend-architecture` | **Security**, for the environment variables shipped to the browser; **UI**, for the size of a loading state | Leaves out both checks, which it does not repeat |
| `performance-analysis` | **UI**, for the size of a loading state | Still treats layout shift as a bug; the rule that a skeleton keeps the final content's size is not in the session |
