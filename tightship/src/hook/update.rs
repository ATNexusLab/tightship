use std::env;
use std::process::{Command, Stdio};

use crate::failure::Failure;
use crate::update::state;

pub fn run() -> Result<(), Failure> {
    let data = crate::variable("TIGHTSHIP_DATA")?;
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
