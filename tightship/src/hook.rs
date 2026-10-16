pub mod checkpoint;
pub mod contract;
pub mod contract_part;
pub mod export;
pub mod prompt;
pub mod start;
pub mod status_line;

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::ops::BitOr;
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

pub fn emit(text: &str) -> Result<(), Failure> {
    emit_shaped(text, template("TIGHTSHIP_HOOK_OUTPUT"))
}

pub fn emit_checkpoint(text: &str) -> Result<(), Failure> {
    let shape =
        template("TIGHTSHIP_CHECKPOINT_OUTPUT").or_else(|| template("TIGHTSHIP_HOOK_OUTPUT"));
    emit_shaped(text, shape)
}

fn template(variable: &str) -> Option<PathBuf> {
    env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn emit_shaped(text: &str, template: Option<PathBuf>) -> Result<(), Failure> {
    if text.is_empty() {
        return Ok(());
    }
    let Some(template) = template else {
        if text.ends_with('\n') {
            print!("{text}");
        } else {
            println!("{text}");
        }
        return Ok(());
    };
    let shape = fs::read_to_string(&template)
        .map_err(|error| Failure::HookOutputUnreadable(template.clone(), error))?;
    let mut payload: serde_json::Value =
        serde_json::from_str(&shape).map_err(|_| Failure::HookOutputInvalid(template.clone()))?;
    if !fill(&mut payload, text) {
        return Err(Failure::HookOutputInvalid(template));
    }
    println!("{payload}");
    Ok(())
}

fn fill(value: &mut serde_json::Value, text: &str) -> bool {
    match value {
        serde_json::Value::String(slot) if slot == "{text}" => {
            *slot = text.to_owned();
            true
        }
        serde_json::Value::Array(items) => items
            .iter_mut()
            .map(|item| fill(item, text))
            .fold(false, BitOr::bitor),
        serde_json::Value::Object(fields) => fields
            .values_mut()
            .map(|field| fill(field, text))
            .fold(false, BitOr::bitor),
        _ => false,
    }
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
