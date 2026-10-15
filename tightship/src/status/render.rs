use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use jiff::Timestamp;
use jiff::tz::TimeZone;
use serde_json::Value;
use unicode_width::UnicodeWidthStr;

use crate::failure::Failure;

struct Segment {
    plain: String,
    colored: String,
    priority: u8,
}

impl Segment {
    fn painted(code: &str, text: String, priority: u8) -> Self {
        Self {
            colored: paint(code, &text),
            plain: text,
            priority,
        }
    }
}

pub fn run() -> Result<(), Failure> {
    let input = io::read_to_string(io::stdin())
        .map_err(|error| Failure::StatusInputInvalid(error.to_string()))?;
    let session: Value = serde_json::from_str(&input)
        .map_err(|error| Failure::StatusInputInvalid(error.to_string()))?;
    if !session.is_object() {
        return Err(Failure::StatusInputInvalid("expected an object".to_owned()));
    }
    let branch =
        text(&session, "/workspace/current_dir").and_then(|folder| read_branch(Path::new(folder)));
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|columns| columns.parse::<usize>().ok())
        .filter(|columns| *columns > 0)
        .map(|columns| columns.saturating_sub(4));
    for row in rows(&session, branch.as_deref(), width) {
        println!("{row}");
    }
    Ok(())
}

fn read_branch(folder: &Path) -> Option<String> {
    let repository = folder
        .ancestors()
        .find(|ancestor| ancestor.join(".git").exists())?;
    let mut git_folder: PathBuf = repository.join(".git");
    if git_folder.is_file() {
        let pointer = fs::read_to_string(&git_folder).ok()?;
        let target = pointer.trim();
        git_folder = repository.join(target.strip_prefix("gitdir: ").unwrap_or(target));
    }
    let head = fs::read_to_string(git_folder.join("HEAD")).ok()?;
    head.trim()
        .strip_prefix("ref: refs/heads/")
        .map(str::to_owned)
}

fn rows(session: &Value, branch: Option<&str>, width: Option<usize>) -> Vec<String> {
    let mut identity = identity_row(session, branch);
    if let (Some(branch), Some(width)) = (branch, width)
        && branch.chars().count() > 20
        && row_width(&identity) > width
    {
        let cut: String = branch.chars().take(19).chain(['…']).collect();
        identity = identity_row(session, Some(&cut));
    }
    let separator = format!(" {} ", paint("2", "·"));
    [identity, usage_row(session)]
        .into_iter()
        .map(|row| fit(row, width))
        .filter(|row| !row.is_empty())
        .map(|row| {
            let colored: Vec<&str> = row.iter().map(|segment| segment.colored.as_str()).collect();
            colored.join(&separator)
        })
        .collect()
}

fn fit(mut row: Vec<Segment>, width: Option<usize>) -> Vec<Segment> {
    let Some(width) = width else {
        return row;
    };
    while row.len() > 1 && row_width(&row) > width {
        let Some(least_important) = (0..row.len()).max_by_key(|index| row[*index].priority) else {
            break;
        };
        row.remove(least_important);
    }
    row
}

fn row_width(row: &[Segment]) -> usize {
    let plain: Vec<&str> = row.iter().map(|segment| segment.plain.as_str()).collect();
    plain.join(" · ").width()
}

fn identity_row(session: &Value, branch: Option<&str>) -> Vec<Segment> {
    let mut row = Vec::new();
    if let Some(folder) = text(session, "/workspace/current_dir") {
        let name = Path::new(folder)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        row.push(Segment::painted("36", name.unwrap_or_default(), 1));
    }
    if let (Some(owner), Some(name)) = (
        text(session, "/workspace/repo/owner"),
        text(session, "/workspace/repo/name"),
    ) {
        row.push(Segment::painted("2;37", format!("{owner}/{name}"), 9));
    }
    if let Some(branch) = branch {
        row.push(Segment::painted("33", branch.to_owned(), 2));
    }
    if let Some(number) = count(session, "/pr/number") {
        let state = text(session, "/pr/review_state").unwrap_or("open");
        let code = match state {
            "approved" => "32",
            "changes_requested" => "31",
            _ => "33",
        };
        row.push(Segment::painted(code, format!("PR #{number} ({state})"), 7));
    }
    if let Some(worktree) = text(session, "/worktree/name") {
        row.push(Segment::painted("2;36", format!("wt:{worktree}"), 10));
    }
    if let Some(agent) = text(session, "/agent/name") {
        row.push(Segment::painted("35", format!("agent:{agent}"), 11));
    }
    if let Some(model) = text(session, "/model/display_name") {
        let model = model.strip_prefix("Claude ").unwrap_or(model);
        let short_name = model.find(" (").map_or(model, |end| &model[..end]);
        row.push(Segment::painted("1;37", short_name.to_owned(), 4));
    }
    if let Some(effort) = text(session, "/effort/level") {
        let code = match effort {
            "low" => "2",
            "medium" => "33",
            "high" => "36",
            "xhigh" | "max" => "31",
            _ => "37",
        };
        row.push(Segment::painted(code, effort.to_owned(), 8));
    }
    row
}

fn usage_row(session: &Value) -> Vec<Segment> {
    let mut row = Vec::new();
    if let Some(percentage) = number(session, "/context_window/used_percentage") {
        let tokens = match (
            count(session, "/context_window/total_input_tokens"),
            count(session, "/context_window/context_window_size"),
        ) {
            (Some(input), Some(size)) => format!("{}/{}", compact(input), compact(size)),
            _ => String::new(),
        };
        row.push(gauge("ctx", percentage, tokens, 3));
    }
    if let Some(percentage) = number(session, "/rate_limits/five_hour/used_percentage") {
        let reset = reset(session, "/rate_limits/five_hour/resets_at", "%H:%M");
        row.push(gauge("5h", percentage, reset, 5));
    }
    if let Some(percentage) = number(session, "/rate_limits/seven_day/used_percentage") {
        let reset = reset(session, "/rate_limits/seven_day/resets_at", "%d/%m %H:%M");
        row.push(gauge("7d", percentage, reset, 6));
    }
    row
}

fn gauge(label: &str, percentage: f64, detail: String, priority: u8) -> Segment {
    let rounded = percentage.round();
    let code = match rounded {
        value if value < 50.0 => "32",
        value if value < 80.0 => "33",
        _ => "31",
    };
    let filled = (rounded * 6.0 / 100.0).round().clamp(0.0, 6.0) as usize;
    let reading = format!(
        "{label} {}{} {rounded}%",
        "█".repeat(filled),
        "░".repeat(6 - filled)
    );
    if detail.is_empty() {
        return Segment::painted(code, reading, priority);
    }
    Segment {
        colored: format!("{} {}", paint(code, &reading), paint("2", &detail)),
        plain: format!("{reading} {detail}"),
        priority,
    }
}

fn compact(count: u64) -> String {
    match count {
        1_000_000.. => format!("{}M", (count as f64 / 100_000.0).round() / 10.0),
        1_000.. => format!("{}k", (count as f64 / 1_000.0).round()),
        _ => count.to_string(),
    }
}

fn reset(session: &Value, pointer: &str, format: &str) -> String {
    let Some(timestamp) = session
        .pointer(pointer)
        .and_then(Value::as_i64)
        .and_then(|second| Timestamp::from_second(second).ok())
    else {
        return String::new();
    };
    timestamp
        .to_zoned(TimeZone::system())
        .strftime(format)
        .to_string()
}

fn paint(code: &str, text: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

fn text<'session>(session: &'session Value, pointer: &str) -> Option<&'session str> {
    session.pointer(pointer).and_then(Value::as_str)
}

fn number(session: &Value, pointer: &str) -> Option<f64> {
    session.pointer(pointer).and_then(Value::as_f64)
}

fn count(session: &Value, pointer: &str) -> Option<u64> {
    session.pointer(pointer).and_then(Value::as_u64)
}
