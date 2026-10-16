use std::fs;
use std::path::Path;

use crate::failure::Failure;

pub fn run(gate: Option<&str>, file: &Path) -> Result<(), Failure> {
    let options = crate::options::load()?;
    if !options.contract {
        return Ok(());
    }
    if let Some(gate) = gate
        && !options.is_enabled(gate)?
    {
        return Ok(());
    }
    let content = fs::read_to_string(file)
        .map_err(|error| Failure::ContractFileUnreadable(file.to_path_buf(), error))?;
    if content.is_empty() {
        return Ok(());
    }
    super::emit(&content)
}
