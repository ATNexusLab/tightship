//! The checkpoint hook: asks the agent, before the context is compacted, whether anything belongs in the vault.

use crate::failure::Failure;

const CHECKPOINT: &str = "\
MEMORY CHECKPOINT — did anything this session become a durable fact?

Only these enter the vault:
1. A rule or preference the user stated — type `preference`.
2. A decision plus its why, one that outlives this task — type `decision`.
3. A trap that cost real time and will recur — type `gotcha`.
4. A project fact that lives in no repo, not the code, not git, not the docs — type `project`.

Everything else stays out: derivable from the repo, session state, task progress, whatever merely
seemed interesting. The test is the `why` field — if you cannot write in one line what not knowing
it would cost later, it does not qualify.

To write one (single step, then tell the user in one line what landed):
  tightship vault new [--scope <scope|all>] --type <preference|decision|gotcha|project> \\
    --title \"...\" --why \"...\" --tags a,b --source <user|session|repo> <<'EOF'
  <body>
  EOF

The note is then reached with `tightship vault search`, which reads the title, the `why`, the tags and the
body — so put in them the words someone hitting this again would type. Add `--pin` only for a
standing fact the work needs before it knows to search: pinned notes are listed at every session
start, a space that holds about thirty.

Nothing durable? Ignore this and carry on.";

/// Prints the checkpoint when memory is on and a vault folder is set.
///
/// # Errors
///
/// When the options cannot be read or the text cannot be emitted.
pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    if !options.memory {
        return Ok(());
    }
    if options.vault_folder.is_some() {
        super::emit_checkpoint(CHECKPOINT)?;
    }
    Ok(())
}
