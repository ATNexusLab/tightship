//! `tightship vault path`: prints the vault folder.

use crate::failure::Failure;

/// Prints the vault folder.
///
/// # Errors
///
/// When the vault folder is not set up.
pub fn run() -> Result<(), Failure> {
    println!("{}", super::folder()?.display());
    Ok(())
}
