use std::collections::BTreeMap;
use std::path::Path;

use super::note::Note;

const CAP: usize = 9000;
const BRACKETS_RESERVE: usize = 200;

struct Selection<'notes> {
    handoff: Option<&'notes Note>,
    debts: Vec<&'notes Note>,
    pinned: Vec<&'notes Note>,
    unlisted: usize,
    other_scopes: BTreeMap<&'notes str, usize>,
}

struct Brackets {
    other_scopes: Option<String>,
}

impl Brackets {
    fn more(more: usize) -> String {
        format!("[{more} more notes in these scopes — `tightship vault search <terms>`]")
    }

    fn pinned_cut(cut: usize) -> String {
        format!(
            "[{cut} pinned notes cut at the {CAP}-character cap — `tightship vault lint` names the scope]"
        )
    }

    fn debts_cut(cut: usize) -> String {
        format!("[{cut} debts cut at the {CAP}-character cap — tightship vault search to-repo]")
    }
}

fn length(text: &str) -> usize {
    text.encode_utf16().count()
}

pub fn render(vault: &Path, scope: &str, notes: &[Note]) -> (String, usize) {
    let vault = vault.display();
    let mut text = format!(
        "# Vault — curated cross-harness memory\n\n\
         Scope `{scope}` and `all`. Before non-trivial work, search them for the subsystem and the task:\n\
         `tightship vault search <term> <term>` — one line per match, its `why` stating\n\
         what not knowing it costs. Open one at `{vault}/<path>`. Rules: `{vault}/AGENTS.md`.\n\
         Listed below, the project's debts and the pinned notes. A hint to verify against the repo, never truth.\n\
         Write with `tightship vault new --help`."
    );
    let selection = select(notes, scope);
    let brackets = Brackets {
        other_scopes: (!selection.other_scopes.is_empty()).then(|| {
            let counts: Vec<String> = selection
                .other_scopes
                .iter()
                .map(|(name, count)| format!("{name}({count})"))
                .collect();
            format!(
                "[other scopes: {} — `tightship vault search --everywhere <terms>`]",
                counts.join(" ")
            )
        }),
    };
    // The brackets close the text, so the budget keeps room for the widest they can be.
    let widest_brackets: usize = [
        Brackets::more(selection.unlisted),
        Brackets::pinned_cut(selection.pinned.len()),
        Brackets::debts_cut(selection.debts.len()),
    ]
    .iter()
    .chain(&brackets.other_scopes)
    .map(|line| length(line) + 1)
    .sum();
    let reserve = BRACKETS_RESERVE.max(widest_brackets + 2);
    let budget = CAP - reserve - 1;

    if let Some(handoff) = selection.handoff {
        text += &handoff_section(handoff, scope, &vault.to_string(), length(&text), budget);
    }
    let mut used = length(&text);
    let (listed_debts, debts_cut) = append_debts(&mut text, &mut used, &selection.debts, budget);
    let cut = append_pinned(
        &mut text,
        &mut used,
        &selection.pinned,
        debts_cut > 0,
        budget,
    );

    let mut lines = Vec::new();
    let more = selection.unlisted - listed_debts;
    if more > 0 {
        lines.push(Brackets::more(more));
    }
    lines.extend(brackets.other_scopes);
    if cut > 0 {
        lines.push(Brackets::pinned_cut(cut));
    }
    if debts_cut > 0 {
        lines.push(Brackets::debts_cut(debts_cut));
    }
    if !lines.is_empty() {
        text += "\n\n";
        text += &lines.join("\n");
    }
    (text + "\n", cut)
}

fn select<'notes>(notes: &'notes [Note], scope: &str) -> Selection<'notes> {
    let handoff = super::handoff::in_scope(notes, scope).into_iter().next();
    let is_handoff = |note: &&Note| handoff.is_some_and(|chosen| chosen.path == note.path);
    let is_debt = |note: &&Note| note.field("project") == scope && note.is_debt();
    let mut debts: Vec<&Note> = notes
        .iter()
        .filter(is_debt)
        .filter(|note| !is_handoff(note))
        .collect();
    debts.sort_by_key(|note| note.id_order());
    let mut pinned: Vec<&Note> = notes
        .iter()
        .filter(|note| note.is_in_reach(scope))
        .filter(|note| note.is_pinned())
        .filter(|note| !is_handoff(note) && !is_debt(note))
        .collect();
    pinned.sort_by_key(|note| (note.field("project") != scope, note.id_order()));
    let unlisted = notes.iter().filter(|note| note.is_in_reach(scope)).count()
        - pinned.len()
        - usize::from(handoff.is_some());
    let mut other_scopes: BTreeMap<&str, usize> = BTreeMap::new();
    for note in notes.iter().filter(|note| !note.is_in_reach(scope)) {
        *other_scopes.entry(note.field("project")).or_default() += 1;
    }
    Selection {
        handoff,
        debts,
        pinned,
        unlisted,
        other_scopes,
    }
}

fn handoff_section(handoff: &Note, scope: &str, vault: &str, used: usize, budget: usize) -> String {
    let heading = format!(
        "\n\n## {}\n\nClaim it with the handoff command once you pick the work up.\n\n",
        super::handoff::heading(handoff, scope)
    );
    if used + length(&heading) + length(&handoff.body) <= budget {
        heading + &handoff.body
    } else {
        heading
            + &format!(
                "The handoff is {} characters, more than the session start holds; read it at {vault}/{}.",
                length(&handoff.body),
                handoff.path.display()
            )
    }
}

fn append_debts(
    text: &mut String,
    used: &mut usize,
    debts: &[&Note],
    budget: usize,
) -> (usize, usize) {
    let mut listed = 0;
    let mut cut = 0;
    for debt in debts {
        let line = match debt.destination() {
            Some(destination) => format!("- [[{}]] → {destination}", debt.stem()),
            None => format!("- [[{}]] — {}", debt.stem(), debt.field("why")),
        };
        let addition = if listed == 0 {
            format!(
                "\n\n## Debts — move into this repo\n\n\
                 Move each into the file it names, in this work's commit, and delete the note.\n\n{line}"
            )
        } else {
            format!("\n{line}")
        };
        if cut > 0 || *used + length(&addition) > budget {
            cut += 1;
            continue;
        }
        *used += length(&addition);
        *text += &addition;
        listed += 1;
    }
    (listed, cut)
}

fn append_pinned(
    text: &mut String,
    used: &mut usize,
    pinned: &[&Note],
    debts_were_cut: bool,
    budget: usize,
) -> usize {
    let mut section = None;
    let mut cut = 0;
    for note in pinned {
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
        if debts_were_cut || cut > 0 || *used + length(&addition) > budget {
            cut += 1;
            continue;
        }
        *used += length(&addition);
        *text += &addition;
        section = Some(project);
    }
    cut
}
