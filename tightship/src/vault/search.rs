use std::cmp::Reverse;

use super::note::{self, Note};
use crate::failure::Failure;
use crate::project;

pub fn run(terms: &[String], everywhere: bool) -> Result<(), Failure> {
    let terms: Vec<&str> = terms
        .iter()
        .map(|term| term.trim())
        .filter(|term| !term.is_empty())
        .collect();
    if terms.is_empty() {
        return Err(Failure::SearchNoTerm);
    }
    let vault = super::folder()?;
    let notes = note::read_all(&vault)?;
    let scope = project::scope(&vault)?;
    let found = rank(&scope, &notes, &terms, everywhere);
    if found.is_empty() {
        let hint = if everywhere {
            ""
        } else {
            ", or add --everywhere for the other scopes"
        };
        println!(
            "no note matches {} — try other terms{hint}",
            terms.join(" · ")
        );
        return Ok(());
    }
    let shown = found.len().min(10);
    println!(
        "{shown} of {} matching notes, paths under `{}/`:",
        found.len(),
        vault.display()
    );
    for note in &found[..shown] {
        println!("- {} — {}", note.path.display(), note.field("why"));
    }
    Ok(())
}

fn rank<'notes>(
    scope: &str,
    notes: &'notes [Note],
    terms: &[&str],
    everywhere: bool,
) -> Vec<&'notes Note> {
    let fold = |text: &str| {
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let terms: Vec<String> = terms.iter().map(|term| fold(term)).collect();
    let mut matches: Vec<(usize, usize, usize, &Note)> = notes
        .iter()
        .filter(|note| everywhere || [scope, "all"].contains(&note.field("project")))
        .filter_map(|note| {
            let texts = [note.field("why"), note.field("tags"), &note.body].map(fold);
            let occurrences: Vec<usize> = terms
                .iter()
                .map(|term| {
                    texts
                        .iter()
                        .map(|text| text.matches(term.as_str()).count())
                        .sum()
                })
                .collect();
            let matched = occurrences.iter().filter(|count| **count > 0).count();
            let reach = match note.field("project") {
                project if project == scope => 0,
                "all" => 1,
                _ => 2,
            };
            (matched > 0).then(|| (reach, matched, occurrences.iter().sum(), note))
        })
        .collect();
    let order = |(reach, matched, occurrences, note): &(usize, usize, usize, &'notes Note)| {
        (
            *reach,
            Reverse(*matched),
            Reverse(*occurrences),
            note.id_order(),
        )
    };
    matches.sort_by(|first, second| order(first).cmp(&order(second)));
    matches.into_iter().map(|(.., note)| note).collect()
}
