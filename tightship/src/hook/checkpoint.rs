use crate::failure::Failure;

pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    if !options.memory {
        return Ok(());
    }
    if options.vault_folder.is_some() {
        super::emit(super::CHECKPOINT)?;
    }
    Ok(())
}
