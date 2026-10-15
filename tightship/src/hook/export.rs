use crate::failure::Failure;
use crate::vault;

pub fn run() -> Result<(), Failure> {
    match vault::folder() {
        Ok(folder) => super::export_vault(&folder),
        Err(Failure::VaultNotSet | Failure::VaultNotAbsolute(_) | Failure::VaultMissing(_)) => {
            Ok(())
        }
        Err(failure) => Err(failure),
    }
}
