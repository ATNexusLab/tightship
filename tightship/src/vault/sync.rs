use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::note::{self, Note};
use crate::failure::Failure;
use crate::git;

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
        step(&vault, "commit", &["add", "-A"])?;
        step(
            &vault,
            "commit",
            &["commit", "--quiet", "-m", "vault: sync"],
        )?;
    }
    let remotes = step(&vault, "fetch", &["remote"])?;
    let Some(remote) = remotes.lines().next() else {
        println!("kept local — the vault has no git remote");
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
    let pulled = step(
        &vault,
        "rebase",
        &["rev-list", "--count", "HEAD..@{upstream}"],
    )?;
    let added = step(
        &vault,
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
    if let Err(message) = git::run(&vault, &["rebase", "--quiet", "@{upstream}"]) {
        let unmerged = step(
            &vault,
            "rebase",
            &["diff", "--name-only", "--diff-filter=U"],
        )?;
        let paths: Vec<String> = unmerged.lines().map(str::to_owned).collect();
        if paths.is_empty() {
            return Err(Failure::SyncGitFailed(vault, "rebase", message));
        }
        return Err(Failure::SyncConflict(vault, paths));
    }
    let notes = note::read_all(&vault)?;
    let renames = colliding(&notes, &local, note::highest_id(&vault, &notes)?);
    if !renames.is_empty() {
        println!("> Renumbering colliding ids");
        renumber(&vault, &renames)?;
    }
    println!("> Pushing");
    let pushed = step(
        &vault,
        "push",
        &["rev-list", "--count", "@{upstream}..HEAD"],
    )?;
    step(&vault, "push", &["push", "--quiet"])?;
    println!("synced: {} pushed, {} pulled", pushed.trim(), pulled.trim());
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
            (note, format!("mem-{highest:04}"))
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
    for file in note::markdown_files(vault, &vault.join("notes"))? {
        let path = vault.join(&file);
        let text = fs::read_to_string(&path)
            .map_err(|error| Failure::NoteUnreadable(file.clone(), error))?;
        let mut rewritten = match renamed.iter().find(|(new_path, _, _)| *new_path == file) {
            Some((_, old_id, new_id)) => with_id(&text, old_id, new_id),
            None => text.clone(),
        };
        for (old_stem, new_stem) in &links {
            for end in ["]]", "|", "#"] {
                rewritten =
                    rewritten.replace(&format!("[[{old_stem}{end}"), &format!("[[{new_stem}{end}"));
            }
        }
        if rewritten != text {
            fs::write(&path, rewritten).map_err(|error| Failure::NoteUnwritable(file, error))?;
        }
    }
    step(vault, "commit", &["add", "-A", "notes"])?;
    step(
        vault,
        "commit",
        &["commit", "--quiet", "-m", "vault: renumber colliding notes"],
    )?;
    Ok(())
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
