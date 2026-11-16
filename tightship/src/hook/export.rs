//! The export hook: hands the vault folder to the harness's shell when memory is on.

use crate::failure::Failure;
use crate::vault;

/// Writes `TIGHTSHIP_VAULT` to the harness's environment file; a vault not set up yet is skipped.
///
/// # Errors
///
/// When the options are invalid, the vault folder cannot be read, or the file cannot be written.
pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    if !options.memory {
        return Ok(());
    }
    match vault::folder_from(&options) {
        Ok(folder) => super::export_vault(&folder),
        Err(Failure::VaultNotSet | Failure::VaultNotAbsolute(_) | Failure::VaultMissing(_)) => {
            Ok(())
        }
        Err(failure) => Err(failure),
    }
}
