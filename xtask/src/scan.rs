//! `cargo xtask scan`: no secret in the git history and no vault note in the working tree.

mod secrets;
mod vault_notes;

use std::path::Path;

use crate::diagnostics::{Failure, stop_on_errors};

/// Runs both scans and reports every finding at once.
///
/// # Errors
///
/// When a scan finds something, or git or gitleaks cannot run.
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
