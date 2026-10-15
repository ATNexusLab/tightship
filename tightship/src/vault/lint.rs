use std::collections::{BTreeSet, HashMap, HashSet};

use super::context;
use super::note::{self, Note};
use crate::failure::Failure;

pub fn run() -> Result<(), Failure> {
    let vault = super::folder()?;
    let notes = note::read_all(&vault)?;
    let stems: HashSet<String> = note::markdown_files(&vault, &vault)?
        .iter()
        .filter_map(|file| file.file_stem())
        .map(|stem| stem.to_string_lossy().into_owned())
        .collect();
    let mut first_with_id: HashMap<&str, &Note> = HashMap::new();
    let mut with_problems = 0;
    for note in &notes {
        let id = note.field("id");
        let found = problems(note, first_with_id.get(id).copied(), &stems);
        if !id.is_empty() {
            first_with_id.entry(id).or_insert(note);
        }
        if found.is_empty() {
            continue;
        }
        with_problems += 1;
        println!("{}", note.path.display());
        for problem in found {
            println!("  - {problem}");
        }
    }
    let scopes: BTreeSet<String> = notes.iter().map(folder_of).collect();
    let mut over = 0;
    for scope in &scopes {
        let (_, cut) = context::render(&vault, scope, &notes);
        if cut > 0 {
            over += 1;
            println!(
                "scope `{scope}`: {cut} pinned notes do not fit the session start — unpin them, move them to the repo, or delete them"
            );
        }
    }
    println!(
        "{} notes, {with_problems} with problems, {over} scopes over the session start",
        notes.len()
    );
    if with_problems > 0 || over > 0 {
        return Err(Failure::LintFailed);
    }
    Ok(())
}

fn problems(note: &Note, earlier: Option<&Note>, stems: &HashSet<String>) -> Vec<String> {
    let mut found = Vec::new();
    for field in [
        "id", "type", "project", "why", "source", "created", "updated",
    ] {
        if note.field(field).is_empty() {
            found.push(format!("missing `{field}`"));
        }
    }
    let kind = note.field("type");
    let kinds = ["project", "gotcha", "decision", "preference", "handoff"];
    if !kind.is_empty() && !kinds.contains(&kind) {
        found.push(format!(
            "type `{kind}` is not one of project, gotcha, decision, preference, handoff"
        ));
    }
    let source = note.field("source");
    if !source.is_empty() && !["user", "session", "repo"].contains(&source) {
        found.push(format!(
            "source `{source}` is not one of user, session, repo"
        ));
    }
    let id = note.field("id");
    let is_valid_id = id.strip_prefix("mem-").is_some_and(|digits| {
        digits.len() >= 4 && digits.bytes().all(|byte| byte.is_ascii_digit())
    });
    if !id.is_empty() && !is_valid_id {
        found.push(format!("id `{id}` is not mem- and four or more digits"));
    }
    if let Some(earlier) = earlier {
        let file_name = earlier
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        found.push(format!("id `{id}` already used by {file_name}"));
    }
    let project = note.field("project");
    let folder = folder_of(note);
    if !project.is_empty() && project != folder {
        found.push(format!(
            "project `{project}` disagrees with folder `{folder}`"
        ));
    }
    let why = note.field("why");
    if !why.is_empty() && why.split_whitespace().count() < 5 {
        found.push("why names a topic instead of stating what ignoring it costs".to_owned());
    }
    for target in links(&outside_code(&note.body)) {
        if !stems.contains(target) {
            found.push(format!("[[{target}]] points at no note"));
        }
    }
    if let Some((_, pinned)) = note.fields.iter().find(|(key, _)| key == "pinned")
        && pinned != "true"
    {
        found.push(format!("pinned `{pinned}` is neither `true` nor absent"));
    }
    found
}

fn folder_of(note: &Note) -> String {
    let folder = note.path.parent().and_then(|parent| parent.file_name());
    folder.unwrap_or_default().to_string_lossy().into_owned()
}

fn outside_code(body: &str) -> String {
    let mut kept = String::new();
    let mut rest = body;
    while let Some(start) = rest.find('`') {
        kept += &rest[..start];
        let fence = if rest[start..].starts_with("```") {
            "```"
        } else {
            "`"
        };
        let after = &rest[start + fence.len()..];
        rest = match after.find(fence) {
            Some(end) => &after[end + fence.len()..],
            None => after,
        };
    }
    kept + rest
}

fn links(text: &str) -> Vec<&str> {
    text.split("[[")
        .skip(1)
        .filter_map(|piece| piece.split_once("]]"))
        .filter_map(|(inside, _)| inside.split(['|', '#']).next())
        .map(str::trim)
        .collect()
}
