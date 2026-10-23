use super::content::split_front_matter;
use super::harness::Harness;
use super::options::OptionsFile;
use super::tokens;
use crate::diagnostics::Diagnostic;

pub struct Contract {
    pub path: String,
    pub text: String,
    outline: Option<Outline>,
}

struct Outline {
    title: String,
    title_line: usize,
    preamble: String,
    sections: Vec<Section>,
    gated: Vec<(String, String)>,
}

struct Section {
    heading: String,
    line: usize,
    text: String,
}

pub struct Part {
    pub file: String,
    pub text: String,
    pub gate: Option<String>,
}

pub fn read(
    path: &str,
    text: String,
    options: Option<&OptionsFile>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Contract {
    let found_before = diagnostics.len();
    let outline = outline(path, &text, options, diagnostics);
    Contract {
        path: path.to_owned(),
        outline: outline.filter(|_| diagnostics.len() == found_before),
        text,
    }
}

fn outline(
    path: &str,
    text: &str,
    options: Option<&OptionsFile>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Outline> {
    let (gated, body, body_line) = match split_front_matter(text) {
        Some((front_matter, body, body_line)) => {
            (gates(path, front_matter, diagnostics)?, body, body_line)
        }
        None if text.starts_with("+++\n") => {
            diagnostics.push(Diagnostic::in_file(
                path,
                "front matter: no closing +++".to_owned(),
            ));
            return None;
        }
        None => (Vec::new(), text, 1),
    };
    let mut title = None;
    let mut preamble = Vec::new();
    let mut sections: Vec<Section> = Vec::new();
    let mut fence: Option<&str> = None;
    for (index, line) in body.split('\n').enumerate() {
        let marker = ["```", "~~~"]
            .into_iter()
            .find(|marker| line.trim_start().starts_with(marker));
        if let Some(marker) = marker {
            fence = match fence {
                Some(open) if open == marker => None,
                None => Some(marker),
                open => open,
            };
        }
        let is_text = fence.is_none() && marker.is_none();
        if title.is_none() {
            if line.trim().is_empty() {
                continue;
            }
            let Some(heading) = line.strip_prefix("# ").filter(|_| is_text) else {
                diagnostics.push(Diagnostic::in_file(
                    path,
                    "the contract opens with no # title".to_owned(),
                ));
                return None;
            };
            title = Some((heading.trim().to_owned(), body_line + index));
            continue;
        }
        let heading = line.strip_prefix("## ").filter(|_| is_text);
        match (heading, sections.last_mut()) {
            (Some(heading), _) => sections.push(Section {
                heading: heading.trim().to_owned(),
                line: body_line + index,
                text: format!("{line}\n"),
            }),
            (None, Some(section)) => section.text.push_str(&format!("{line}\n")),
            (None, None) => preamble.push(line),
        }
    }
    let Some((title, title_line)) = title else {
        diagnostics.push(Diagnostic::in_file(
            path,
            "the contract opens with no # title".to_owned(),
        ));
        return None;
    };
    for section in &mut sections {
        section.text = section.text.trim_end().to_owned();
    }
    for (heading, option) in &gated {
        if !sections.iter().any(|section| section.heading == *heading) {
            diagnostics.push(Diagnostic::in_file(
                path,
                format!(
                    "front matter: [gated] names the section \"{heading}\", which the file does not have"
                ),
            ));
        }
        let is_declared = options.is_some_and(|file| {
            file.declared
                .iter()
                .any(|declared| declared.name == *option)
        });
        if !is_declared {
            diagnostics.push(Diagnostic::in_file(
                path,
                format!(
                    "front matter: the section \"{heading}\" is gated by \"{option}\", which content/options.toml does not declare"
                ),
            ));
        }
    }
    Some(Outline {
        title,
        title_line,
        preamble: preamble.join("\n").trim().to_owned(),
        sections,
        gated,
    })
}

fn gates(
    path: &str,
    front_matter: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Vec<(String, String)>> {
    let mut reject = |reason: String| {
        diagnostics.push(Diagnostic::in_file(path, format!("front matter: {reason}")));
    };
    let table: toml::Table = match front_matter.parse() {
        Ok(table) => table,
        Err(error) => {
            reject(error.message().to_owned());
            return None;
        }
    };
    let mut gated = Vec::new();
    let mut is_valid = true;
    for (key, value) in table {
        let entries = match (key.as_str(), value) {
            ("gated", toml::Value::Table(entries)) => entries,
            ("gated", _) => {
                reject("[gated] is not a table".to_owned());
                is_valid = false;
                continue;
            }
            _ => {
                reject(format!("unknown key \"{key}\""));
                is_valid = false;
                continue;
            }
        };
        for (heading, option) in entries {
            match option {
                toml::Value::String(option) => gated.push((heading, option)),
                _ => {
                    reject(format!(
                        "[gated] maps \"{heading}\" to something other than an option name"
                    ));
                    is_valid = false;
                }
            }
        }
    }
    is_valid.then_some(gated)
}

pub fn pack(
    contract: &Contract,
    harness: &Harness,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Part> {
    struct Block<'contract> {
        subject: String,
        line: usize,
        text: String,
        gate: Option<&'contract str>,
        heading: &'contract str,
    }
    let Some(outline) = &contract.outline else {
        return Vec::new();
    };
    let Some(table) = &harness.contract else {
        diagnostics.push(Diagnostic::in_file(
            &format!("harnesses/{}/harness.toml", harness.name),
            format!("{} exists, so [contract] limit is required", contract.path),
        ));
        return Vec::new();
    };
    let limit = table.limit;
    let render = |text: &str| tokens::render(text, harness).0;
    let length = |text: &str| text.encode_utf16().count();
    let compose = |title_line: &str, blocks: &[&Block]| {
        let texts: Vec<&str> = blocks.iter().map(|block| block.text.as_str()).collect();
        format!("{title_line}\n\n{}\n", texts.join("\n\n"))
    };
    let title = render(&outline.title);
    let preamble = Block {
        subject: "the preamble".to_owned(),
        line: outline.title_line,
        text: render(&outline.preamble),
        gate: None,
        heading: "",
    };
    let sections = outline.sections.iter().map(|section| Block {
        subject: format!("the section \"{}\"", section.heading),
        line: section.line,
        text: render(&section.text),
        gate: outline
            .gated
            .iter()
            .find(|(heading, _)| *heading == section.heading)
            .map(|(_, option)| option.as_str()),
        heading: &section.heading,
    });
    let blocks: Vec<Block> = std::iter::once(preamble)
        .filter(|block| !block.text.is_empty())
        .chain(sections)
        .collect();
    let ungated: Vec<&Block> = blocks.iter().filter(|block| block.gate.is_none()).collect();

    let title_line = |number: usize, total: usize| {
        let heading = format!("# {title} ({number}/{total})");
        match &table.compaction_notice {
            Some(notice) if number == 1 => format!("{heading}\n\n{notice}"),
            _ => heading,
        }
    };
    let pack_with = |total: usize| {
        let mut groups: Vec<Vec<&Block>> = Vec::new();
        for block in &ungated {
            let title_line = title_line(groups.len().max(1), total);
            let joins = groups.last().is_some_and(|group| {
                let candidate: Vec<&Block> = group.iter().copied().chain([*block]).collect();
                length(&compose(&title_line, &candidate)) <= limit
            });
            match groups.last_mut() {
                Some(group) if joins => group.push(block),
                _ => groups.push(vec![block]),
            }
        }
        groups
    };
    let mut total = 1;
    let mut groups = pack_with(total);
    while groups.len() != total {
        total = groups.len();
        groups = pack_with(total);
    }

    let mut parts = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        let number = index + 1;
        let text = compose(&title_line(number, total), group);
        let count = length(&text);
        if count > limit {
            let block = group[0];
            diagnostics.push(Diagnostic::at_line(
                &contract.path,
                block.line,
                format!(
                    "{} renders to {count} characters with its title; {} takes at most {limit} per hook",
                    block.subject, harness.name
                ),
            ));
        }
        parts.push(Part {
            file: format!("part-{number}.md"),
            text,
            gate: None,
        });
    }

    let mut options: Vec<&str> = Vec::new();
    for (_, option) in &outline.gated {
        if !options.contains(&option.as_str()) {
            options.push(option);
        }
    }
    for option in options {
        let group: Vec<&Block> = blocks
            .iter()
            .filter(|block| block.gate == Some(option))
            .collect();
        let headings: Vec<String> = group.iter().map(|block| render(block.heading)).collect();
        let text = compose(&format!("# {title} ({})", headings.join(", ")), &group);
        let count = length(&text);
        if count > limit {
            diagnostics.push(Diagnostic::in_file(
                &contract.path,
                format!(
                    "the sections gated by \"{option}\" render to {count} characters with their title; {} takes at most {limit} per hook",
                    harness.name
                ),
            ));
        }
        parts.push(Part {
            file: format!("{option}.md"),
            text,
            gate: Some(option.to_owned()),
        });
    }
    const MOST_CHARACTERS: usize = 8000;
    let count: usize = parts.iter().map(|part| length(&part.text)).sum();
    if count > MOST_CHARACTERS {
        diagnostics.push(Diagnostic::in_file(
            &contract.path,
            format!(
                "the contract renders to {count} characters with every option on; NFR-PERF-03 allows {MOST_CHARACTERS}"
            ),
        ));
    }
    parts
}
