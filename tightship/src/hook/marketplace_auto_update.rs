//! The marketplace hook: keeps the harness's auto-update of the plugin in step with the option.

use serde_json::{Value, json};

use super::settings;
use crate::failure::Failure;
use crate::update::release::Manifest;

/// Sets `autoUpdate` on the plugin's marketplace in the user's settings, writing only when it changed.
///
/// # Errors
///
/// When the manifest or the settings are invalid, or the settings cannot be written.
pub fn run() -> Result<(), Failure> {
    let options = crate::options::load()?;
    let manifest = Manifest::read(&crate::env::variable("TIGHTSHIP_MANIFEST")?)?;
    let settings_path = crate::env::variable("TIGHTSHIP_SETTINGS")?;
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
