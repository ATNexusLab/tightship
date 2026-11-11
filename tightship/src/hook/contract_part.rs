//! The contract-part hook: prints one file of the working contract when its option is on.

use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

/// Prints the contract file, unless the contract or the file's own option is off.
///
/// # Errors
///
/// When the name is not a plain file name, the file is missing or unreadable, or the options are invalid.
pub fn run(file: &str) -> Result<(), Failure> {
    let options = crate::options::load()?;
    if !options.contract {
        return Ok(());
    }
    if let Some(gate) = gate_for_file(file)
        && !options.is_enabled(&gate)?
    {
        return Ok(());
    }
    let root = crate::env::variable("TIGHTSHIP_ROOT")?;
    let path = contract_file(&root, file)?;
    let text =
        fs::read_to_string(&path).map_err(|error| Failure::ContractPartUnreadable(path, error))?;
    super::emit(&text)
}

fn gate_for_file(file: &str) -> Option<String> {
    if file.starts_with("part-") || !file.ends_with(".md") {
        return None;
    }
    Some(file.trim_end_matches(".md").to_owned())
}

fn contract_file(root: &Path, file: &str) -> Result<PathBuf, Failure> {
    if file.is_empty() || file.contains('/') || file.contains('\\') || file.contains("..") {
        return Err(Failure::ContractPartNameInvalid(file.to_owned()));
    }
    let joined = root.join("contract").join(file);
    if !joined.is_file() {
        return Err(Failure::ContractPartMissing(joined));
    }
    Ok(joined)
}
