use crate::failure::Failure;

pub fn run() -> Result<(), Failure> {
    if super::vault_is_set() {
        println!("{}", super::CHECKPOINT);
    }
    Ok(())
}
