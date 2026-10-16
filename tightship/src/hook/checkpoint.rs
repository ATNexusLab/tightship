use crate::failure::Failure;

pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    if !options.memory {
        return Ok(());
    }
    if super::vault_is_set() {
        super::emit(super::CHECKPOINT)?;
    }
    Ok(())
}
