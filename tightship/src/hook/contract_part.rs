use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::options;

pub fn run(file: &str) -> Result<(), Failure> {
    if !options::cursor_boolean("contract")? {
        return Ok(());
    }
    if let Some(gate) = gate_for_file(file)
        && !options::cursor_boolean(&gate)?
    {
        return Ok(());
    }
    let root = plugin_root()?;
    let path = contract_file(&root, file)?;
    let text = fs::read_to_string(&path).map_err(|error| Failure::ContractPartUnreadable(path, error))?;
    let payload = serde_json::json!({ "additional_context": text });
    println!("{}", serde_json::to_string(&payload).map_err(|error| {
        Failure::OptionsIo(error.to_string())
    })?);
    Ok(())
}

fn gate_for_file(file: &str) -> Option<String> {
    if file.starts_with("part-") || !file.ends_with(".md") {
        return None;
    }
    Some(file.trim_end_matches(".md").to_owned())
}

fn plugin_root() -> Result<PathBuf, Failure> {
    for key in ["CURSOR_PLUGIN_ROOT", "CLAUDE_PLUGIN_ROOT"] {
        if let Ok(root) = env::var(key)
            && !root.is_empty()
        {
            return Ok(PathBuf::from(root));
        }
    }
    Err(Failure::PluginRootUnset)
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
