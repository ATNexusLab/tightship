//! Checking that each rendered skill's body fits what its harness loads.

use std::fs;
use std::io;
use std::path::Path;

use crate::build::check::content::Content;
use crate::build::harness::Harness;
use crate::diagnostics::{Diagnostic, with_path};

/// Reports each `skills/<name>/SKILL.md` in a staging folder whose body passes the harness's
/// `[skills] body-limit`.
///
/// # Errors
///
/// When the staging folder's skills cannot be listed or read.
pub fn check_folder(
    staging: &Path,
    harness: &Harness,
    content: &Content,
    diagnostics: &mut Vec<Diagnostic>,
) -> io::Result<()> {
    let Some(limit) = harness.skills.as_ref().map(|table| table.body_limit) else {
        return Ok(());
    };
    let folder = staging.join("skills");
    if !folder.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&folder).map_err(with_path(&folder))? {
        let skill = entry.map_err(with_path(&folder))?.path();
        let file = skill.join("SKILL.md");
        if !file.is_file() {
            continue;
        }
        let text = fs::read_to_string(&file).map_err(with_path(&file))?;
        let size = body(&text).len();
        if size <= limit {
            continue;
        }
        let name = skill
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let source = content
            .pieces
            .iter()
            .find(|piece| piece.name == name)
            .map_or_else(
                || format!("dist/{}/skills/{name}/SKILL.md", harness.name),
                |piece| piece.path.clone(),
            );
        diagnostics.push(Diagnostic::in_file(
            &source,
            format!(
                "its body renders to {size} bytes for {}, which loads only the first {limit}",
                harness.name
            ),
        ));
    }
    Ok(())
}

fn body(text: &str) -> &str {
    let after_front_matter = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.find("\n---\n").map(|end| &rest[end + 5..]))
        .unwrap_or(text);
    // The harness counts from the first line of text, past the blank line the render puts after the front matter.
    after_front_matter.trim_start_matches('\n')
}
