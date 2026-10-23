pub mod apply;
pub mod release;
pub mod state;
pub mod tools;

use std::fs;
use std::io;
use std::path::Path;

use jiff::Timestamp;

use crate::failure::Failure;
use release::{Manifest, Release, Version};
use state::Notice;

pub fn run() -> Result<(), Failure> {
    if !crate::options::load()?.auto_update {
        return Ok(());
    }
    let data = crate::variable("TIGHTSHIP_DATA")?;
    let root = crate::variable("TIGHTSHIP_ROOT")?;
    let manifest_path = crate::variable("TIGHTSHIP_MANIFEST")?;
    let harness = crate::variable("TIGHTSHIP_HARNESS")?;
    let Some(_lock) = state::lock(&data)? else {
        println!("Another tightship update is running.");
        return Ok(());
    };
    let mut checked = state::read(&data);
    checked.checked_at = Some(Timestamp::now());
    let manifest = Manifest::read(&manifest_path);
    checked.error = manifest.as_ref().err().map(ToString::to_string);
    state::write(&data, &checked)?;
    let manifest = manifest?;
    let outcome = check(&data, &root, &manifest, &harness.to_string_lossy());
    let mut ended = state::read(&data);
    ended.error = outcome.as_ref().err().map(ToString::to_string);
    let from = match &ended.notice {
        Some(notice) if !notice.refused => notice.from.clone(),
        _ => manifest.version.to_string(),
    };
    match &outcome {
        Ok(Some(version)) => {
            ended.notice = Some(Notice {
                refused: false,
                from,
                to: version.to_string(),
            });
        }
        Err(Failure::UpdateIntegrity { tag, .. }) => {
            ended.notice = Some(Notice {
                refused: true,
                from: manifest.version.to_string(),
                to: tag.trim_start_matches('v').to_owned(),
            });
        }
        _ => {}
    }
    state::write(&data, &ended)?;
    match outcome? {
        Some(version) => println!(
            "Updated tightship from {} to {version}. The next session loads it.",
            manifest.version
        ),
        None => println!("tightship {} is the latest release.", manifest.version),
    }
    Ok(())
}

fn check(
    data: &Path,
    root: &Path,
    manifest: &Manifest,
    harness: &str,
) -> Result<Option<Version>, Failure> {
    let work = data.join("update");
    let unwritable = |error| Failure::UpdateStateUnwritable(work.clone(), error);
    match fs::remove_dir_all(&work) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(unwritable(error)),
        _ => {}
    }
    fs::create_dir_all(&work).map_err(unwritable)?;

    let latest_url = format!(
        "{}/releases/latest/download/latest.json",
        manifest.repository
    );
    let latest = work.join("latest.json");
    tools::download(&latest_url, &latest)?;
    let text = fs::read_to_string(&latest)
        .map_err(|_| Failure::UpdateReleaseInvalid(latest_url.clone()))?;
    let release = Release::parse(&text, &latest_url, harness)?;
    if release.version <= manifest.version {
        let _ = fs::remove_dir_all(&work);
        return Ok(None);
    }

    let archive = work.join(&release.archive);
    tools::download(
        &format!(
            "{}/releases/download/{}/{}",
            manifest.repository, release.tag, release.archive
        ),
        &archive,
    )?;
    let actual = tools::sha256(&archive, &release.archive)?;
    if actual != release.sha256 {
        let _ = fs::remove_dir_all(&work);
        return Err(Failure::UpdateIntegrity {
            archive: release.archive,
            actual,
            tag: release.tag,
            expected: release.sha256,
        });
    }

    let staging = root.join(apply::STAGING);
    match fs::remove_dir_all(&staging) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            return Err(Failure::UpdateApplyFailed(staging, error));
        }
        _ => {}
    }
    let applied = tools::extract(&archive, &staging, &release.archive)
        .and_then(|()| apply::check_plain(&staging, &release.archive))
        .and_then(|()| apply::place_windows_hooks(&staging))
        .and_then(|()| apply::swap(root, &staging));
    let _ = fs::remove_dir_all(&staging);
    let _ = fs::remove_dir_all(&work);
    applied?;
    Ok(Some(release.version))
}
