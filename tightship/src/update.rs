//! `tightship update`: checks GitHub Releases for a newer plugin, verifies it, and swaps it in.

mod apply;
mod commands;
pub mod release;
pub mod state;

use std::fs;
use std::path::{Path, PathBuf};

use jiff::Timestamp;

use crate::failure::Failure;
use crate::files::remove_folder_if_present;
use release::{Manifest, Release, Version};
use state::Notice;

/// Installs a newer release when there is one, recording the check and a notice for the next session.
///
/// # Errors
///
/// When the manifest is invalid, or a download, the checksum, the extraction, or the swap fails.
pub fn run() -> Result<(), Failure> {
    if !crate::options::load()?.auto_update {
        return Ok(());
    }
    let data = crate::env::variable("TIGHTSHIP_DATA")?;
    let root = crate::env::variable("TIGHTSHIP_ROOT")?;
    let manifest_path = crate::env::variable("TIGHTSHIP_MANIFEST")?;
    let harness = crate::env::variable("TIGHTSHIP_HARNESS")?;
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
    if let Some(notice) = notice_for(&outcome, ended.notice.as_ref(), &manifest) {
        ended.notice = Some(notice);
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

fn notice_for(
    outcome: &Result<Option<Version>, Failure>,
    previous: Option<&Notice>,
    manifest: &Manifest,
) -> Option<Notice> {
    // An update that lands on top of an unread one keeps the version the user last saw.
    let from = match previous {
        Some(notice) if !notice.refused => notice.from.clone(),
        _ => manifest.version.to_string(),
    };
    match outcome {
        Ok(Some(version)) => Some(Notice {
            refused: false,
            from,
            to: version.to_string(),
        }),
        Err(Failure::UpdateIntegrity { tag, .. }) => Some(Notice {
            refused: true,
            from: manifest.version.to_string(),
            to: tag.trim_start_matches('v').to_owned(),
        }),
        _ => None,
    }
}

fn check(
    data: &Path,
    root: &Path,
    manifest: &Manifest,
    harness: &str,
) -> Result<Option<Version>, Failure> {
    let work = data.join("update");
    let unwritable = |error| Failure::UpdateStateUnwritable(work.clone(), error);
    remove_folder_if_present(&work).map_err(unwritable)?;
    fs::create_dir_all(&work).map_err(unwritable)?;
    let Some(release) = newer_release(&work, manifest, harness)? else {
        // The work folder is scratch, emptied again at the start of the next check.
        let _ = fs::remove_dir_all(&work);
        return Ok(None);
    };
    let archive = download_verified(&work, manifest, &release)?;
    let staging = root.join(apply::STAGING);
    remove_folder_if_present(&staging)
        .map_err(|error| Failure::UpdateApplyFailed(staging.clone(), error))?;
    let applied = install(root, &staging, &archive, &release.archive);
    // The work folder is scratch, emptied again at the start of the next check.
    let _ = fs::remove_dir_all(&work);
    applied?;
    Ok(Some(release.version))
}

fn newer_release(
    work: &Path,
    manifest: &Manifest,
    harness: &str,
) -> Result<Option<Release>, Failure> {
    let latest_url = format!(
        "{}/releases/latest/download/latest.json",
        manifest.repository
    );
    let latest = work.join("latest.json");
    commands::download(&latest_url, &latest)?;
    let text = fs::read_to_string(&latest)
        .map_err(|_| Failure::UpdateReleaseInvalid(latest_url.clone()))?;
    let release = Release::parse(&text, &latest_url, harness)?;
    Ok((release.version > manifest.version).then_some(release))
}

fn download_verified(
    work: &Path,
    manifest: &Manifest,
    release: &Release,
) -> Result<PathBuf, Failure> {
    let archive = work.join(&release.archive);
    commands::download(
        &format!(
            "{}/releases/download/{}/{}",
            manifest.repository, release.tag, release.archive
        ),
        &archive,
    )?;
    let actual = commands::sha256(&archive, &release.archive)?;
    if actual != release.sha256 {
        // The integrity failure is what matters; the work folder is emptied at the next check.
        let _ = fs::remove_dir_all(work);
        return Err(Failure::UpdateIntegrity {
            archive: release.archive.clone(),
            actual,
            tag: release.tag.clone(),
            expected: release.sha256.clone(),
        });
    }
    Ok(archive)
}

fn install(root: &Path, staging: &Path, archive: &Path, archive_name: &str) -> Result<(), Failure> {
    let applied = commands::extract(archive, staging, archive_name)
        .and_then(|()| apply::check_plain(staging, archive_name))
        .and_then(|()| apply::place_windows_hooks(staging))
        .and_then(|()| apply::swap(root, staging));
    // The staging folder is removed again before the next install.
    let _ = fs::remove_dir_all(staging);
    applied
}
