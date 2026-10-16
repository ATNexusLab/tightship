use crate::failure::Failure;
use crate::project;
use crate::vault::{self, context, note};

pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    if !options.memory {
        return Ok(());
    }
    let folder = match vault::folder() {
        Ok(folder) => folder,
        Err(Failure::VaultNotSet) => {
            return super::emit(
                "# Vault\n\nMemory is on and no vault folder is set, so this session has no vault. When the user asks\nabout memory, tell them to create one with `tightship vault init <folder>`, set that folder as the\nplugin's vault folder, and start a new session, or to turn the memory option off.\n",
            );
        }
        Err(Failure::VaultNotAbsolute(value)) => {
            return super::emit(&format!(
                "# Vault\n\nMemory is on and the vault folder `{}` is not an absolute path, so this session has no\nvault. When the user asks about memory, tell them to set the plugin's vault folder to an absolute path.\n",
                value.display()
            ));
        }
        Err(Failure::VaultMissing(folder)) => {
            return super::emit(&format!(
                "# Vault\n\nMemory is on and the vault folder `{folder}` does not exist, so this session has no vault.\nWhen the user asks about memory, tell them to create it with `tightship vault init {folder}`, or to set\nanother folder as the plugin's vault folder.\n",
                folder = folder.display()
            ));
        }
        Err(failure) => return Err(failure),
    };
    let notes = note::read_all(&folder)?;
    let scope = project::scope(&folder)?;
    let (memory_context, _) = context::render(&folder, &scope, &notes);
    super::export_vault(&folder)?;
    super::emit(&memory_context)
}
