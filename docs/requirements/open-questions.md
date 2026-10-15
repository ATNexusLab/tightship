# Open questions

Undecided items. Each one leaves this file for a link to the spec or decision that settled it. A
question's number is never reused.

| ID | Question | Affects | Settled by |
| --- | --- | --- | --- |
| <a id="oq-04"></a>OQ-04 | What is the minimum supported version of each harness? | [NFR-FLEX-01](non-functional/flexibility.md#nfr-flex-01) to [NFR-FLEX-04](non-functional/flexibility.md#nfr-flex-04) | Each harness's spec |
| <a id="oq-05"></a>OQ-05 | How does a plugin install the companion plugins with itself in Cursor, Antigravity CLI, and Codex, and where the harness cannot, what does the adopter see? | [FR-INS-02](functional/install.md#fr-ins-02) | Each harness's spec |
| <a id="oq-06"></a>OQ-06 | How does the whole contract reach the session in Cursor, Antigravity CLI, and Codex, given the limits each one sets on always-on text? | [FR-CTR-01](functional/contract.md#fr-ctr-01) | Each harness's spec |
| <a id="oq-07"></a>OQ-07 | Does Cursor's command-line agent load the always-on text and the hooks of a plugin, as the editor does? | [NFR-FLEX-02](non-functional/flexibility.md#nfr-flex-02) | M2 spec |
| <a id="oq-08"></a>OQ-08 | Can Cursor, Antigravity CLI, and Codex show a status line a plugin provides? | [FR-STL-01](functional/status-line.md#fr-stl-01) | Each harness's spec |
