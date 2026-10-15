+++
name = "sync"
description = "Sync the memory vault with its git remote, and resolve what git hands back"
+++

Sync the memory vault. `<vault>` below is what `tightship vault path` prints.

1. Run `tightship vault sync`. On exit 0, tell the adopter its last line, and stop.
2. When it fails with `these notes conflict with the remote` or `a rebase or merge is under way`, list the
   conflicting notes with `git -C <vault> diff --name-only --diff-filter=U`. Open each one and keep the facts
   of both sides, under the vault's `AGENTS.md`; when the two sides state the same fact, keep one. Remove the
   conflict markers, run `git -C <vault> add` on the note, then
   `git -c core.editor=true -C <vault> rebase --continue`, then `tightship vault lint`, and go back to step 1.
3. When it fails with `tracks no remote branch`, run the `git push -u` its message names, once, and go back to
   step 1.
4. On any other failure, show the adopter git's message and what it needs from them, and stop. Credentials,
   the network, and the git identity are the adopter's.
5. Never force-push, reset, stash, or delete a note to make a conflict go away. When the same note conflicts
   again on a second pass, stop and show it to the adopter.
