+++
name = "handoff"
description = "Release this session's work as a handoff, or claim the one waiting — to switch machine, OS or harness mid-task"
argument-hint = "release | claim"
+++

Run the handoff step for this project: {{capability:arguments}}

A handoff is the one vault note that holds a session's state: one per project scope, replaced on release
and removed on claim. `tightship vault handoff` writes and removes it, `tightship vault sync` carries it
between machines, and this command writes its text and reads the repository.

Memory is `{{option:memory}}` for this plugin. When that reads exactly `false`, tell the user "Memory is off,
so this project has no vault to hand off through. Turn the memory option on and start a new session to use
it." and stop. When the argument is neither `release` nor `claim`, tell the user "Run the handoff command with
`release` or `claim`." and stop.

The project's repository is only read, never changed: no commit, push, stash, or checkout. Its approval
stays with the user.

## release

1. Read the repository: `git branch --show-current`, `git status -sb`, and the commits not on the remote,
   `git log --oneline @{upstream}..`, or every commit of the branch when it tracks no remote branch.
2. Write the body from the session, in English like every vault note:

   ```markdown
   **Goal:** <what the work is for, with the issue or plan it follows>
   **Done:** <what is finished and verified>
   **Next step:** <the one action the next session starts with>
   **Open decisions:** <what waits for the user, or none>
   **Repo state:** branch `<name>` · <clean, or the dirty files> · <each unpushed commit as its short hash and subject>
   ```

   Anything only on this machine — a dirty file, an unpushed commit, a running server — is named in
   **Repo state** with the warning that the other machine does not have it.
3. From the project's directory, run `tightship vault handoff release --title "<what the work is, and where
   it stopped>"` with the body on stdin. When it fails, show its message and stop.
4. Run `tightship vault sync`. When it fails, follow `{{command:sync}}` from its step 2.
5. Tell the user, in one line: "Handoff released: \<the path release printed\> — \<the sync's last line\>."

## claim

1. Run `tightship vault sync`. When it fails, follow `{{command:sync}}` from its step 2; when that ends
   without a sync, stop and claim nothing, since the remote may hold a newer handoff.
2. From the project's directory, run `tightship vault handoff claim`. When it prints
   `no handoff for <scope>`, tell the user, word for word, "No handoff waits for \<scope\>; nothing changed."
   and stop.
3. Run `tightship vault sync` again, as in step 1, so the other machines see the handoff is gone.
4. Compare each handoff with the repository: run `git fetch`, then check every commit of **Repo state** with
   `git cat-file -e <hash>^{commit}`, and the branch and the working tree against it. Name each difference
   with its sentence, word for word:
   - "Commit \<hash\> is not in this repository; it stayed on the machine that released the handoff."
   - "The handoff was released on branch \<name\>, and this checkout is on \<name\>."
   - "The handoff names uncommitted changes to \<files\>; they stayed on the other machine."
5. Show the next step and the open decisions of each handoff, and wait for the user before acting on them.
