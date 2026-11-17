# Without the contract

English · [Português](without-the-contract.pt-BR.md)

With the `contract` option off, the session no longer loads the workflow's rules. Every skill and command
still loads and works. Each one holds the rules of its own step, so it does the same with the contract off.
The session loses the rules every task needs: the hard lines, how a convention yields to your decision, the
rules for code and for done, and the memory rules. The agent then goes by your repository's `AGENTS.md` and
its own judgement.

One piece still points to the contract instead of repeating it:

| Piece | What it refers to | What it does with the contract off |
| --- | --- | --- |
| `bootstrap` | The whole contract: the `AGENTS.md` it writes adds to the contract and never repeats it | Writes the same `AGENTS.md`, which leaves out everything the contract covers. Those rules then exist nowhere unless you add them to the repository |
