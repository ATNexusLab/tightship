use std::fs;
use std::io;
use std::path::Path;

use crate::diagnostics::{Diagnostic, naming};
use crate::git;

pub fn check(root: &Path, diagnostics: &mut Vec<Diagnostic>) -> io::Result<()> {
    let tracked = git::run(root, &["ls-files", "-z"])?;
    for path in tracked.split('\0').filter(|path| !path.is_empty()) {
        let file = root.join(path);
        if !file.is_file() {
            continue;
        }
        let bytes = fs::read(&file).map_err(naming(&file))?;
        if is_vault_note(&String::from_utf8_lossy(&bytes)) {
            diagnostics.push(Diagnostic::general(format!(
                "{path} is a vault note; vault notes never enter this repository"
            )));
        }
    }
    Ok(())
}

fn is_vault_note(text: &str) -> bool {
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    if lines.next().map(str::trim_end) != Some("---") {
        return false;
    }
    lines
        .map(str::trim_end)
        .take_while(|line| *line != "---")
        .filter_map(|line| line.strip_prefix("id:"))
        .map(|value| value.trim().trim_matches(['"', '\'']))
        .filter_map(|value| value.strip_prefix("mem-"))
        .any(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
}
