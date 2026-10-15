use std::collections::BTreeMap;
use std::path::Path;

use super::note::Note;

const CAP: usize = 9000;
const BRACKETS_RESERVE: usize = 200;

pub fn render(vault: &Path, scope: &str, notes: &[Note]) -> (String, usize) {
    let vault = vault.display();
    let length = |text: &str| text.encode_utf16().count();
    let in_reach = |note: &&Note| [scope, "all"].contains(&note.field("project"));
    let mut text = format!(
        "# Vault — curated cross-harness memory\n\n\
         Scope `{scope}` and `all`. Before non-trivial work, search them for the subsystem and the task:\n\
         `tightship vault search <term> <term>` — one line per match, its `why` stating\n\
         what not knowing it costs. Open one at `{vault}/<path>`. Rules: `{vault}/AGENTS.md`.\n\
         Listed below, only the pinned notes. A hint to verify against the repo, never truth.\n\
         Write with `tightship vault new --help`."
    );

    let handoff = notes
        .iter()
        .filter(in_reach)
        .filter(|note| note.field("type") == "handoff" && note.field("project") == scope)
        .max_by(|first, second| {
            (first.field("updated"), first.id_order())
                .cmp(&(second.field("updated"), second.id_order()))
        });
    let mut pinned: Vec<&Note> = notes
        .iter()
        .filter(in_reach)
        .filter(|note| note.is_pinned())
        .filter(|note| handoff.is_none_or(|chosen| chosen.path != note.path))
        .collect();
    pinned.sort_by_key(|note| (note.field("project") != scope, note.id_order()));
    let more =
        notes.iter().filter(in_reach).count() - pinned.len() - usize::from(handoff.is_some());
    let mut other_scopes: BTreeMap<&str, usize> = BTreeMap::new();
    for note in notes.iter().filter(|note| !in_reach(note)) {
        *other_scopes.entry(note.field("project")).or_default() += 1;
    }

    let mut brackets = Vec::new();
    if more > 0 {
        brackets.push(format!(
            "[{more} more notes in these scopes — `tightship vault search <terms>`]"
        ));
    }
    if !other_scopes.is_empty() {
        let counts: Vec<String> = other_scopes
            .iter()
            .map(|(name, count)| format!("{name}({count})"))
            .collect();
        brackets.push(format!(
            "[other scopes: {} — `tightship vault search --everywhere <terms>`]",
            counts.join(" ")
        ));
    }
    let cut_line = |cut: usize| {
        format!(
            "[{cut} pinned notes cut at the {CAP}-character cap — `tightship vault lint` names the scope]"
        )
    };
    let widest_brackets: usize = brackets
        .iter()
        .chain([&cut_line(pinned.len())])
        .map(|line| length(line) + 1)
        .sum();
    let reserve = BRACKETS_RESERVE.max(widest_brackets + 2);
    let budget = CAP - reserve - 1;

    if let Some(handoff) = handoff {
        let heading = format!(
            "\n\n## Handoff — {scope} · {} · {}\n\n",
            handoff.field("id"),
            handoff.age()
        );
        if length(&text) + length(&heading) + length(&handoff.body) <= budget {
            text += &heading;
            text += &handoff.body;
        } else {
            text += &heading;
            text += &format!(
                "The handoff is {} characters, more than the session start holds; read it at {vault}/{}.",
                length(&handoff.body),
                handoff.path.display()
            );
        }
    }

    let mut used = length(&text);
    let mut section = None;
    let mut cut = 0;
    for note in &pinned {
        let project = note.field("project");
        let line = format!(
            "- [[{}]] ({}) — {}",
            note.stem(),
            note.age(),
            note.field("why")
        );
        let addition = if section == Some(project) {
            format!("\n{line}")
        } else {
            format!("\n\n## Scope — {project}\n\n{line}")
        };
        if cut > 0 || used + length(&addition) > budget {
            cut += 1;
            continue;
        }
        used += length(&addition);
        text += &addition;
        section = Some(project);
    }

    if cut > 0 {
        brackets.push(cut_line(cut));
    }
    if !brackets.is_empty() {
        text += "\n\n";
        text += &brackets.join("\n");
    }
    (text + "\n", cut)
}
