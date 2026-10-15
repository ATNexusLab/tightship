mod secrets;
mod vault_notes;

use std::path::Path;

use crate::{Failure, stop_on_errors};

pub fn run(root: &Path) -> Result<String, Failure> {
    println!(
        "> Scanning the git history for secrets (gitleaks {})",
        secrets::GITLEAKS_VERSION
    );
    let mut diagnostics = Vec::new();
    let commits = secrets::check(root, &mut diagnostics)?;
    println!("> Looking for vault notes in the working tree");
    vault_notes::check(root, &mut diagnostics)?;
    stop_on_errors(&mut diagnostics)?;
    Ok(format!("No secret in {commits} commits. No vault note."))
}
