//! `tightship docs affected`: names the docs whose `covers` match the files a diff changes.

use std::collections::BTreeSet;

use super::{Repository, glob};
use crate::failure::Failure;
use crate::system::git;

/// Prints each doc a changed file falls under, with the files that matched it.
///
/// The diff is the working tree, staged changes included, against `base` (`HEAD` when unset), plus the
/// files git does not track yet.
///
/// # Errors
///
/// When the repository or a doc cannot be read, or git cannot diff against `base`.
pub fn run(base: Option<&str>) -> Result<(), Failure> {
    let repository = Repository::open()?;
    let base = base.unwrap_or("HEAD");
    let diff = git::run(&repository.root, &["diff", "--name-only", base, "--"])
        .map_err(Failure::DocsGitFailed)?;
    let untracked = git::run(
        &repository.root,
        &["ls-files", "--others", "--exclude-standard"],
    )
    .map_err(Failure::DocsGitFailed)?;
    let changed: BTreeSet<&str> = diff.lines().chain(untracked.lines()).collect();
    let mut any = false;
    for doc in repository.docs()? {
        let matched: Vec<&str> = changed
            .iter()
            .copied()
            .filter(|file| {
                doc.covers
                    .iter()
                    .any(|cover| glob::matches(&cover.pattern, file))
            })
            .collect();
        if !matched.is_empty() {
            any = true;
            println!("{} ← {}", doc.path, matched.join(", "));
        }
    }
    if !any {
        println!("no doc covers the changed files");
    }
    Ok(())
}
