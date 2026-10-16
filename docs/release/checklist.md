# Release checklist

One row per acceptance criterion of every functional requirement of M1, the macOS signature, and
[NFR-FLEX-02.1](../requirements/non-functional/flexibility.md#nfr-flex-02) Scenario 6 for Cursor's
command-line agent. Run it on macOS, Linux, and Windows for every release, and write ✅ or ❌ with the date
in each system's column. A release is promoted only when every row passes on all three ([README](README.md)).
The Claude rows use the preparation below; the Cursor row uses the Cursor preparation as well.

## Prepare each system

1. Check that `claude --version` is 2.1.295 or later.
2. Add the companions' three catalogs and the pre-release's own catalog, then install. The pre-release's
   catalog goes in a folder of its own, at `.claude-plugin/marketplace.json`, since Claude Code clones every
   `https://github.com/` URL as a git repository:

   ```sh
   claude plugin marketplace add DietrichGebert/ponytail
   claude plugin marketplace add ayghri/i-have-adhd
   claude plugin marketplace add blader/humanizer
   mkdir -p ~/.cache/tightship-catalog/.claude-plugin
   curl -fsSL -o ~/.cache/tightship-catalog/.claude-plugin/marketplace.json \
     https://github.com/ATNexusLab/tightship/releases/download/v<version>/marketplace.json
   claude plugin marketplace add ~/.cache/tightship-catalog
   claude plugin install tightship@tightship
   ```

3. Create a scratch vault with `tightship vault init <folder>` from a session, and a scratch git repository
   to open the sessions in.
4. Run `cargo test --workspace` from the tagged commit. Every row marked **Test** passes when the named test
   passes on that system.

On Windows, run every command in Git Bash.

### Cursor

On the same three systems, after the steps above:

1. Check that the Cursor editor is 3.25.6 or later and `agent --version` reports 2026.10.14 or later.
2. From the tagged commit, run `cargo xtask build` in the clone so `dist/cursor/` exists.
3. Add the three companion catalogs per `docs/install/cursor/companions.md` (marketplace add only; no
   tightship install). Reuse the scratch vault and repository from Claude preparation step 3 when a later row
   needs them; the NFR-FLEX-02.1 row below needs only step 2.

On Windows, run every command in Git Bash.

## Install

| Criterion | How it is checked | macOS | Linux | Windows |
| --- | --- | --- | --- | --- |
| FR-INS-01.1 | Manual: after step 2, `claude plugin list` shows `tightship@tightship` at the release's version with no error, and a session lists its skills and commands | | | |
| FR-INS-02.1 | Manual: after step 2, `claude plugin list` shows `ponytail`, `i-have-adhd`, and `humanizer` | | | |
| FR-INS-02.2 | n/a: Claude Code installs dependencies, so `install.md` adds the companions' catalogs first | n/a | n/a | n/a |
| FR-INS-03.1 | Manual: reinstall with `--config memory=false --config status_line=false`; the next session has no vault context and no status line offer | | | |
| FR-INS-03.2 | Manual: install with no `--config`; the next session loads the contract and the vault's pinned notes | | | |
| FR-INS-04.1 | Manual: `echo '{"memory":"false"}' \| claude plugin configure tightship@tightship --values-stdin`; the next session has no vault context | | | |
| FR-INS-05.1 | Manual: with `memory` off, run `claude plugin update tightship@tightship` against the next pre-release; `claude plugin configure tightship@tightship` still shows `memory` off. Skipped on the first release, with nothing older to update from | | | |
| FR-INS-06.1 | Manual: follow `docs/install/claude-code/uninstall.md`; `~/.claude` holds no tightship file or option, and `git -C <vault> status` is clean | | | |
| FR-INS-07.1 | Manual: on a configuration with the baseline merged in, follow `migration.md`; `/hooks` shows no `vault.py` hook, and each skill and command appears once | | | |
| FR-INS-07.2 | Manual: after the same migration, `diff -r` against the backup shows only the files the migration lists, and the vault's `git status` is clean | | | |
| FR-INS-08.1 | Manual: `docs/install/claude-code/README.md` links install, companions, options, update, uninstall, and migration | | | |

## Skills, commands, and verifier

| Criterion | How it is checked | macOS | Linux | Windows |
| --- | --- | --- | --- | --- |
| FR-SKL-01.1 | Manual: in a session, the skill list holds the 13 skills of `content/skills/` under `tightship:` | | | |
| FR-SKL-01.2 | Manual: ask for `tightship:project-docs`; the agent reads `docs-theme.css` and `image-viewer.js` from the skill's folder | | | |
| FR-SKL-02.1 | Manual: run `/tightship:handoff claim` and `/tightship:sprint plan`; each answers from its own instructions with the argument given | | | |
| FR-SKL-03.1 | Test: `xtask/tests/build.rs` `references_render_under_the_plugin_names` and `a_token_naming_a_missing_piece_fails_and_leaves_no_plugin` | | | |
| FR-VER-01.1 | Manual: ask the agent to dispatch `tightship:adversarial-verifier` on "`README.md` exists"; it returns a verdict | | | |
| FR-VER-02.1 | Manual: dispatch the verifier with the instruction to edit `README.md`; the edit is refused | | | |
| FR-VER-02.2 | Manual: after the verifier returns, `git status` in the scratch repository is as it was before | | | |

## Contract

| Criterion | How it is checked | macOS | Linux | Windows |
| --- | --- | --- | --- | --- |
| FR-CTR-01.1 | Manual: ask the agent for the contract's first and last rule; it quotes both. Test: `xtask/tests/contract.rs` `the_contract_renders_into_parts_gated_files_and_session_start_hooks` | | | |
| FR-CTR-01.2 | Manual: run `/compact`, then ask again for the first and last rule; it quotes both | | | |
| FR-CTR-02.1 | Manual: with `contract` off, a new session cannot quote any rule of the contract | | | |
| FR-CTR-02.2 | Manual: with `contract` off, `/tightship:closeout` runs | | | |
| FR-CTR-03.1 | Manual: with `memory` off, the contract in the session holds no rule that names the vault. Test: `xtask/tests/contract.rs` `the_hooks_honor_the_options` | | | |
| FR-CTR-03.2 | Manual: with both on, the contract holds its **Memory** section | | | |
| FR-CTR-04.1 | Manual: every rule of the baseline's contract is in `content/`, or in the epic #4 spec's list of personal and changed rules | | | |
| FR-CTR-05.1 | Manual: `without-the-contract.md` names every skill and command whose text in `content/` names the contract or one of its sections | | | |

## Memory

| Criterion | How it is checked | macOS | Linux | Windows |
| --- | --- | --- | --- | --- |
| FR-MEM-01.1 | Manual: a pinned note in the project's scope is listed at session start. Test: `tightship/tests/hook.rs` `start_lists_the_pinned_notes_of_the_project_then_all` | | | |
| FR-MEM-01.2 | Manual: after `/tightship:handoff release`, a new session shows the handoff's body. Test: `tightship/tests/hook.rs` `start_shows_the_handoff_apart_from_the_pinned_notes` | | | |
| FR-MEM-01.3 | Test: `tightship/tests/hook.rs` `start_takes_the_scope_whose_name_prefixes_the_project_folder` | | | |
| FR-MEM-02.1 | Manual: the 10th prompt of a session shows the checkpoint. Test: `tightship/tests/hook.rs` `prompt_prints_the_checkpoint_every_10_prompts_and_sweeps_stale_counters` | | | |
| FR-MEM-02.2 | Manual: right after `/compact`, the context holds the checkpoint. Test: `tightship/tests/hook.rs` `checkpoint_prints_the_checkpoint_only_with_a_vault` | | | |
| FR-MEM-03.1 | Test: `tightship/tests/vault.rs` `search_lists_the_project_scope_first` | | | |
| FR-MEM-03.2 | Test: `tightship/tests/vault.rs` `search_lists_the_project_scope_first` | | | |
| FR-MEM-04.1 | Manual: the agent writes a note with `tightship vault new`, and it lands in its scope's folder with a new id. Test: `tightship/tests/vault.rs` `new_writes_the_note_and_rejects_a_short_why_a_bad_scope_and_no_body` | | | |
| FR-MEM-05.1 | Test: `tightship/tests/vault.rs` `lint_passes_a_clean_vault_and_names_each_problem` | | | |
| FR-MEM-05.2 | Manual: `tightship vault lint` on the scratch vault exits 0. Test: `tightship/tests/vault.rs` `lint_passes_a_clean_vault_and_names_each_problem` | | | |
| FR-MEM-06.1 | Manual: `/tightship:handoff release` twice; the vault holds one handoff for the scope, with its five parts. Test: `tightship/tests/handoff.rs` `release_replaces_the_scope_handoff` | | | |
| FR-MEM-06.2 | Manual: after the release, `git log`, `git stash list`, and the branch of the scratch repository are unchanged. Test: `tightship/tests/handoff.rs` `release_leaves_the_project_repository_as_it_was` | | | |
| FR-MEM-07.1 | Manual: `/tightship:handoff claim` shows the next step and the open decisions, and the handoff is gone from the vault. Test: `tightship/tests/handoff.rs` `claim_prints_the_handoff_and_removes_it` | | | |
| FR-MEM-07.2 | Manual: release a handoff, `git reset --hard HEAD~1` in the scratch repository, then claim; the agent names the missing commit | | | |
| FR-MEM-07.3 | Manual: claim with no handoff; the agent says there is none. Test: `tightship/tests/handoff.rs` `claim_with_no_handoff_changes_nothing` | | | |
| FR-MEM-08.1 | Manual: step 3's vault is a git repository with the template's `AGENTS.md` and no note. Test: `tightship/tests/vault.rs` `init_creates_a_vault_from_the_template` | | | |
| FR-MEM-08.2 | Test: `tightship/tests/vault.rs` `init_into_a_folder_that_holds_files_changes_nothing` | | | |
| FR-MEM-09.1 | Manual: with `vault_folder` set to step 3's folder, `tightship vault path` prints it, and a new note lands there. Test: `tightship/tests/vault.rs` `path_prints_the_folder_the_adopter_set` | | | |
| FR-MEM-10.1 | Manual: with `memory` off, the session start lists no note and the 10th prompt shows no checkpoint | | | |
| FR-MEM-10.2 | Manual: with `memory` off, `/tightship:handoff release` says memory is off, and the vault is unchanged | | | |
| FR-MEM-11.1 | Manual: `tightship vault sync` against a bare remote with a note on each side; both hold both. Test: `tightship/tests/sync.rs` `sync_takes_the_remote_notes_and_pushes_the_local_ones` | | | |
| FR-MEM-11.2 | Test: `tightship/tests/sync.rs` `a_conflict_stops_in_the_rebase_and_the_resolution_is_pushed` | | | |
| FR-MEM-11.3 | Test: `tightship/tests/sync.rs` `an_id_taken_on_both_machines_is_renumbered_with_its_links` | | | |

## Status line

| Criterion | How it is checked | macOS | Linux | Windows |
| --- | --- | --- | --- | --- |
| FR-STL-01.1 | Manual: say yes to the offer; the status line shows the same segments as the baseline's in the same repository. Test: `tightship/tests/status.rs` `render_prints_both_rows_in_their_colors` | | | |
| FR-STL-02.1 | Manual: say no to the offer; `~/.claude/settings.json` is unchanged. Test: `tightship/tests/status.rs` `no_is_kept_until_the_option_is_turned_off_and_on` | | | |
| FR-STL-03.1 | Manual: with a `statusLine` of your own, say yes, then turn `status_line` off; the next session puts yours back byte for byte. Test: `tightship/tests/status.rs` `yes_saves_the_previous_status_line_and_off_puts_it_back_in_place` | | | |
| FR-STL-03.2 | Test: `tightship/tests/status.rs` `yes_with_no_status_line_before_removes_the_key_when_off` | | | |

## Cursor command-line agent

| Criterion | How it is checked | macOS | Linux | Windows |
| --- | --- | --- | --- | --- |
| NFR-FLEX-02.1 | Manual: after Cursor preparation step 2, start an interactive `agent --plugin-dir dist/cursor` session (not `agent -p`), ask which slash commands are available; the answer lists `/spec` and the other workflow commands from `content/commands/` | | | |

## The macOS binary

| Check | How it is checked | macOS | Linux | Windows |
| --- | --- | --- | --- | --- |
| Signature | With `memory` on, a session starts with no Gatekeeper prompt and no kill, and `codesign -dv` on the installed `bin/macos/tightship` reports an ad-hoc signature | | n/a | n/a |
