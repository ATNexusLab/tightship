# Open questions

Undecided items. Each one leaves this file for a link to the spec or decision that settled it. A
question's number is never reused.

| ID | Question | Affects | Settled by |
| --- | --- | --- | --- |
| <a id="oq-01"></a>OQ-01 | What is the plugin's name? It becomes the prefix of every command in harnesses that namespace plugin commands. | [FR-SKL-02](functional/skills.md#fr-skl-02) | M1 spec |
| <a id="oq-02"></a>OQ-02 | What is the stack: the format of the canonical content, the language of the build, and the language of every script the plugin ships? The answer sets what an adopter must have installed. | [NFR-FLEX-05](non-functional/flexibility.md#nfr-flex-05), [NFR-MNT-01](non-functional/maintainability.md#nfr-mnt-01), [overview](overview.md#operating-environment) | M1 ADR |
| <a id="oq-03"></a>OQ-03 | Which network connections may the plugin make, and where may it write on the adopter's machine? The handoff of the baseline pulls and pushes the vault's git remote. | [FR-MEM-06](functional/memory.md#fr-mem-06), [FR-MEM-07](functional/memory.md#fr-mem-07), [FR-STL-02](functional/status-line.md#fr-stl-02) | M1 ADR |
| <a id="oq-04"></a>OQ-04 | What is the minimum supported version of each harness? | [NFR-FLEX-01](non-functional/flexibility.md#nfr-flex-01) to [NFR-FLEX-04](non-functional/flexibility.md#nfr-flex-04) | Each harness's spec |
| <a id="oq-05"></a>OQ-05 | How does a plugin install the companion plugins with itself in each harness, and where the harness cannot, what does the adopter see? | [FR-INS-02](functional/install.md#fr-ins-02) | Each harness's spec |
| <a id="oq-06"></a>OQ-06 | How does the whole contract reach the session in each harness, given the limits each one sets on always-on text? | [FR-CTR-01](functional/contract.md#fr-ctr-01) | Each harness's spec |
| <a id="oq-07"></a>OQ-07 | Does Cursor's command-line agent load the always-on text and the hooks of a plugin, as the editor does? | [NFR-FLEX-02](non-functional/flexibility.md#nfr-flex-02) | M2 spec |
| <a id="oq-08"></a>OQ-08 | Can Cursor, Antigravity CLI, and Codex show a status line a plugin provides? | [FR-STL-01](functional/status-line.md#fr-stl-01) | Each harness's spec |
| <a id="oq-09"></a>OQ-09 | How does a canonical piece name a capability native to one harness, such as plan mode or a built-in review command, and where is each harness's equivalent stated? | [NFR-FLEX-06](non-functional/flexibility.md#nfr-flex-06) | M1 ADR |
