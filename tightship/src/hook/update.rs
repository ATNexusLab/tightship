//! The update hook: shows the notice of the last update and starts the next check in the background.

use std::env;
use std::process::{Command, Stdio};

use crate::failure::Failure;
use crate::update::state;

/// Prints a pending update notice once, and starts a detached `tightship update` when a check is due.
///
/// # Errors
///
/// When the options are invalid, the state cannot be written, or the update cannot start.
pub fn run() -> Result<(), Failure> {
    let data = crate::system::env::variable("TIGHTSHIP_DATA")?;
    let mut pending = state::read(&data);
    if let Some(notice) = pending.notice.take() {
        super::emit(&notice.text())?;
        state::write(&data, &pending)?;
    }
    if !crate::options::load()?.auto_update || !pending.is_check_due() {
        return Ok(());
    }
    let binary = env::current_exe().map_err(Failure::UpdateStartFailed)?;
    let mut command = Command::new(binary);
    command
        .arg("update")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    command.spawn().map_err(Failure::UpdateStartFailed)?;
    Ok(())
}
