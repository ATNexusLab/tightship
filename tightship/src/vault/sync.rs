//! `tightship vault sync`: commits, rebases on the remote, renumbers colliding ids, expires notes, and pushes.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use jiff::Zoned;

use super::note::{self, Note};
use super::{expire, rules};
use crate::failure::Failure;
use crate::system::git;

/// Syncs the vault with its remote, or keeps it local when it has none.
///
/// # Errors
///
/// When a rebase is under way, there is no upstream, or a git step fails.
pub fn run() -> Result<(), Failure> {
    let vault = super::folder()?;
    if is_under_way(&vault)? {
        return Err(Failure::SyncUnfinished(vault));
    }
    println!("> Recording local changes");
    if !step(&vault, "commit", &["status", "--porcelain"])?
        .trim()
        .is_empty()
    {
        commit(&vault, &["-A"], "vault: sync")?;
    }
    let remotes = step(&vault, "fetch", &["remote"])?;
    let Some(remote) = remotes.lines().next() else {
        let expired = expire(&vault)?;
        update_rules(&vault)?;
        println!("kept local — the vault has no git remote; {expired} expired");
        return Ok(());
    };
    if git::run(&vault, &["rev-parse", "--abbrev-ref", "@{upstream}"]).is_err() {
        let branch = step(&vault, "fetch", &["branch", "--show-current"])?;
        return Err(Failure::SyncNoUpstream {
            branch: branch.trim().to_owned(),
            remote: remote.to_owned(),
            vault,
        });
    }
    println!("> Fetching the remote");
    step(&vault, "fetch", &["fetch", "--quiet"])?;
    println!("> Rebasing local commits on the remote");
    let (pulled, local) = rebase(&vault)?;
    let notes = note::read_all(&vault)?;
    let renames = colliding(&notes, &local, note::highest_id(&vault, &notes)?);
    if !renames.is_empty() {
        println!("> Renumbering colliding ids");
        renumber(&vault, &renames)?;
    }
    let expired = expire(&vault)?;
    update_rules(&vault)?;
    println!("> Pushing");
    let pushed = step(
        &vault,
        "push",
        &["rev-list", "--count", "@{upstream}..HEAD"],
    )?;
    step(&vault, "push", &["push", "--quiet"])?;
    println!(
        "synced: {} pushed, {} pulled, {expired} expired",
        pushed.trim(),
        pulled.trim()
    );
    Ok(())
}

fn rebase(vault: &Path) -> Result<(String, HashSet<PathBuf>), Failure> {
    let pulled = step(
        vault,
        "rebase",
        &["rev-list", "--count", "HEAD..@{upstream}"],
    )?;
    // The notes this clone added are the ones renumbered when their id collides.
    let added = step(
        vault,
        "rebase",
        &[
            "log",
            "--diff-filter=A",
            "--name-only",
            "--format=",
            "@{upstream}..HEAD",
            "--",
            "notes",
        ],
    )?;
    let local: HashSet<PathBuf> = added
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();
    if let Err(message) = git::run(vault, &["rebase", "--quiet", "@{upstream}"]) {
        let unmerged = step(vault, "rebase", &["diff", "--name-only", "--diff-filter=U"])?;
        let paths: Vec<String> = unmerged.lines().map(str::to_owned).collect();
        if paths.is_empty() {
            return Err(Failure::SyncGitFailed(
                vault.to_path_buf(),
                "rebase",
                message,
            ));
        }
        return Err(Failure::SyncConflict(vault.to_path_buf(), paths));
    }
    Ok((pulled, local))
}

fn expire(vault: &Path) -> Result<usize, Failure> {
    let expired = expire::run(vault, Zoned::now().date())?;
    if expired > 0 {
        commit(vault, &["-A", "notes"], "vault: expire notes")?;
    }
    Ok(expired)
}

fn update_rules(vault: &Path) -> Result<(), Failure> {
    if rules::run(vault)? {
        commit(vault, &["AGENTS.md"], "vault: update the rules")?;
    }
    Ok(())
}

fn commit(vault: &Path, paths: &[&str], message: &str) -> Result<(), Failure> {
    let mut add = vec!["add"];
    add.extend_from_slice(paths);
    step(vault, "commit", &add)?;
    step(vault, "commit", &["commit", "--quiet", "-m", message])?;
    Ok(())
}

fn step(vault: &Path, name: &'static str, arguments: &[&str]) -> Result<String, Failure> {
    git::run(vault, arguments)
        .map_err(|message| Failure::SyncGitFailed(vault.to_path_buf(), name, message))
}

fn is_under_way(vault: &Path) -> Result<bool, Failure> {
    for marker in ["rebase-merge", "rebase-apply", "MERGE_HEAD"] {
        let path = step(vault, "commit", &["rev-parse", "--git-path", marker])?;
        if vault.join(path.trim()).exists() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn colliding<'notes>(
    notes: &'notes [Note],
    local: &HashSet<PathBuf>,
    mut highest: u64,
) -> Vec<(&'notes Note, String)> {
    let mut taken: HashSet<&str> = notes
        .iter()
        .filter(|note| !local.contains(&note.path))
        .map(|note| note.field("id"))
        .collect();
    notes
        .iter()
        .filter(|note| local.contains(&note.path) && !note.field("id").is_empty())
        .filter(|note| !taken.insert(note.field("id")))
        .map(|note| {
            highest += 1;
            (note, note::id(highest))
        })
        .collect()
}

fn renumber(vault: &Path, renames: &[(&Note, String)]) -> Result<(), Failure> {
    let mut links = Vec::new();
    let mut renamed = Vec::new();
    for (note, new_id) in renames {
        let old_id = note.field("id");
        let old_stem = note.stem();
        let slug = old_stem
            .strip_prefix(old_id)
            .and_then(|rest| rest.strip_prefix('-'))
            .unwrap_or(&old_stem);
        let new_stem = format!("{new_id}-{slug}");
        let folder = note.path.parent().unwrap_or(Path::new(""));
        let new_path = PathBuf::from(format!("{}/{new_stem}.md", folder.display()));
        let from = note.path.to_string_lossy();
        let to = new_path.to_string_lossy();
        step(vault, "mv", &["mv", &from, &to])?;
        println!("{old_id} → {new_id} {}", new_path.display());
        links.push((old_stem.clone(), new_stem));
        renamed.push((new_path, old_id, new_id.as_str()));
    }
    note::rewrite_all(vault, |file, text| {
        let mut rewritten = match renamed.iter().find(|(new_path, _, _)| new_path == file) {
            Some((_, old_id, new_id)) => with_id(text, old_id, new_id),
            None => text.to_owned(),
        };
        for (old_stem, new_stem) in &links {
            for end in ["]]", "|", "#"] {
                rewritten =
                    rewritten.replace(&format!("[[{old_stem}{end}"), &format!("[[{new_stem}{end}"));
            }
        }
        rewritten
    })?;
    commit(vault, &["-A", "notes"], "vault: renumber colliding notes")
}

fn with_id(text: &str, old_id: &str, new_id: &str) -> String {
    let mut replaced = false;
    text.split_inclusive('\n')
        .map(|line| match line.split_once(':') {
            Some((key, value)) if !replaced && key.trim() == "id" && value.trim() == old_id => {
                replaced = true;
                format!("id: {new_id}\n")
            }
            _ => line.to_owned(),
        })
        .collect()
}
