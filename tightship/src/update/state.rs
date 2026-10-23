use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use jiff::Timestamp;
use serde_json::{Map, Value, json};

use crate::failure::Failure;

const STATE: &str = "update.json";
const LOCK: &str = "update.lock";
const CHECK_INTERVAL_SECONDS: i64 = 86_400;
const ABANDONED_LOCK: Duration = Duration::from_secs(3_600);

#[derive(Default)]
pub struct State {
    pub checked_at: Option<Timestamp>,
    pub notice: Option<Notice>,
    pub error: Option<String>,
}

pub struct Notice {
    pub refused: bool,
    pub from: String,
    pub to: String,
}

impl Notice {
    pub fn text(&self) -> String {
        let Self { from, to, .. } = self;
        if self.refused {
            format!(
                "# tightship update\n\nThe tightship {to} release did not match the sha256 it published, so it was not installed, and {from} stays. Tell the user, word for word: \"tightship refused the {to} update: its download did not match the sha256 the release published.\"\n"
            )
        } else {
            format!(
                "# tightship update\n\ntightship was updated from {from} to {to} since the last session. Tell the user, word for word: \"tightship was updated from {from} to {to}.\"\n"
            )
        }
    }
}

impl State {
    pub fn is_check_due(&self) -> bool {
        self.checked_at.is_none_or(|checked_at| {
            Timestamp::now().duration_since(checked_at).as_secs() > CHECK_INTERVAL_SECONDS
        })
    }
}

pub fn read(data: &Path) -> State {
    fs::read_to_string(data.join(STATE))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .and_then(|value| parse(&value))
        .unwrap_or_default()
}

fn parse(value: &Value) -> Option<State> {
    let fields = value.as_object()?;
    let mut state = State::default();
    for (name, field) in fields {
        match name.as_str() {
            "checked_at" => state.checked_at = Some(field.as_str()?.parse().ok()?),
            "error" => state.error = Some(field.as_str()?.to_owned()),
            "notice" => {
                let refused = match field.get("kind")?.as_str()? {
                    "updated" => false,
                    "refused" => true,
                    _ => return None,
                };
                let version = |name| field.get(name)?.as_str().map(str::to_owned);
                state.notice = Some(Notice {
                    refused,
                    from: version("from")?,
                    to: version("to")?,
                });
            }
            _ => return None,
        }
    }
    Some(state)
}

pub fn write(data: &Path, state: &State) -> Result<(), Failure> {
    let mut fields = Map::new();
    if let Some(checked_at) = state.checked_at {
        fields.insert("checked_at".to_owned(), checked_at.to_string().into());
    }
    if let Some(notice) = &state.notice {
        let kind = if notice.refused { "refused" } else { "updated" };
        fields.insert(
            "notice".to_owned(),
            json!({ "kind": kind, "from": notice.from, "to": notice.to }),
        );
    }
    if let Some(error) = &state.error {
        fields.insert("error".to_owned(), error.as_str().into());
    }
    let path = data.join(STATE);
    let text = serde_json::to_string_pretty(&fields)
        .map_err(|error| Failure::UpdateStateUnwritable(path.clone(), io::Error::other(error)))?;
    fs::create_dir_all(data)
        .and_then(|()| fs::write(&path, text + "\n"))
        .map_err(|error| Failure::UpdateStateUnwritable(path, error))
}

pub struct Lock(PathBuf);

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn lock(data: &Path) -> Result<Option<Lock>, Failure> {
    let path = data.join(LOCK);
    let unwritable = |error| Failure::UpdateStateUnwritable(path.clone(), error);
    fs::create_dir_all(data).map_err(unwritable)?;
    for _ in 0..2 {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => return Ok(Some(Lock(path))),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let abandoned = fs::metadata(&path)
                    .and_then(|metadata| metadata.modified())
                    .ok()
                    .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                    .is_some_and(|age| age > ABANDONED_LOCK);
                if !abandoned {
                    return Ok(None);
                }
                match fs::remove_file(&path) {
                    Err(error) if error.kind() != io::ErrorKind::NotFound => {
                        return Err(unwritable(error));
                    }
                    _ => {}
                }
            }
            Err(error) => return Err(unwritable(error)),
        }
    }
    Ok(None)
}
