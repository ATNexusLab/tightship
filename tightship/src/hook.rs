pub mod checkpoint;
pub mod contract;
pub mod export;
pub mod prompt;
pub mod start;
pub mod status_line;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

const CHECKPOINT: &str = "\
MEMORY CHECKPOINT — did anything this session become a durable fact?

Only these enter the vault:
1. A rule or preference the user stated — type `preference`.
2. A decision plus its why, one that outlives this task — type `decision`.
3. A trap that cost real time and will recur — type `gotcha`.
4. A project fact that lives in no repo, not the code, not git, not the docs — type `project`.

Everything else stays out: derivable from the repo, session state, task progress, whatever merely
seemed interesting. The test is the `why` field — if you cannot write in one line what not knowing
it would cost later, it does not qualify.

To write one (single step, then tell the user in one line what landed):
  tightship vault new --scope <project|all> --type <preference|decision|gotcha|project> \\
    --title \"...\" --why \"...\" --tags a,b --source <user|session|repo> <<'EOF'
  <body>
  EOF

The note is then reached with `tightship vault search`, which reads the title, the `why`, the tags and the
body — so put in them the words someone hitting this again would type. Add `--pin` only for a
standing fact the work needs before it knows to search: pinned notes are listed at every session
start, a space that holds about thirty.

Nothing durable? Ignore this and carry on.";

pub fn vault_is_set() -> bool {
    crate::options::load().is_ok_and(|options| options.memory && options.vault_folder.is_some())
}

pub fn emit(text: &str) -> Result<(), Failure> {
    if text.is_empty() {
        return Ok(());
    }
    if env::var_os("TIGHTSHIP_HOOK_FORMAT").is_some_and(|value| value == "inject-steps") {
        let payload = serde_json::json!({
            "injectSteps": [{
                "systemMessage": {
                    "systemMessage": text
                }
            }]
        });
        println!("{}", serde_json::to_string(&payload).unwrap_or_default());
    } else if text.ends_with('\n') {
        print!("{text}");
    } else {
        println!("{text}");
    }
    Ok(())
}

fn export_vault(folder: &Path) -> Result<(), Failure> {
    let Some(file) = env::var_os("TIGHTSHIP_ENV_FILE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    else {
        return Ok(());
    };
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .and_then(|mut opened| writeln!(opened, "export TIGHTSHIP_VAULT={}", quoted(folder)))
        .map_err(|error| Failure::EnvUnwritable(file, error))
}

fn quoted(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}
