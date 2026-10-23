use serde_json::{Value, json};

use crate::failure::Failure;
use crate::status::settings;
use crate::update::release::Manifest;

pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    let manifest = Manifest::read(&crate::variable("TIGHTSHIP_MANIFEST")?)?;
    let settings_path = crate::variable("TIGHTSHIP_SETTINGS")?;
    let read = settings::read_settings(&settings_path)?;
    let mut user_settings = read.clone();
    let invalid = || Failure::SettingsInvalid(settings_path.clone());
    let entry = user_settings
        .entry("extraKnownMarketplaces")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(invalid)?
        .entry(manifest.name)
        .or_insert_with(|| json!({ "source": source_of(&manifest.repository) }))
        .as_object_mut()
        .ok_or_else(invalid)?;
    entry.insert("autoUpdate".to_owned(), Value::Bool(options.auto_update));
    if user_settings == read {
        return Ok(());
    }
    settings::write_settings(&settings_path, &user_settings)
}

fn source_of(repository: &str) -> Value {
    match repository.strip_prefix("https://github.com/") {
        Some(repo) => json!({ "source": "github", "repo": repo.trim_end_matches(".git") }),
        None => json!({ "source": "git", "url": repository }),
    }
}
