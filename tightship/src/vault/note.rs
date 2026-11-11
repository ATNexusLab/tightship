//! A vault note: its front matter, its id, and reading and writing the notes of a vault.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use jiff::civil::Date;

use crate::failure::Failure;
use crate::system::git;

/// The types a note written with `tightship vault new` may have.
pub const TYPES: [&str; 4] = ["preference", "decision", "gotcha", "project"];
/// The type of a handoff note.
pub const HANDOFF_TYPE: &str = "handoff";
/// Where a note's fact came from.
pub const SOURCES: [&str; 3] = ["user", "session", "repo"];
/// The fewest words a note's `why` may have.
pub const WHY_MIN_WORDS: usize = 5;
const ID_PREFIX: &str = "mem-";

/// A note read from the vault: its path under the vault, its front matter fields, and its body.
pub struct Note {
    pub path: PathBuf,
    pub fields: Vec<(String, String)>,
    pub body: String,
}

impl Note {
    /// The value of the front matter field, empty when it is absent.
    pub fn field(&self, name: &str) -> &str {
        self.fields
            .iter()
            .find(|(key, _)| key == name)
            .map_or("", |(_, value)| value.as_str())
    }

    /// Whether the note belongs to the scope or to `all`.
    pub fn is_in_reach(&self, scope: &str) -> bool {
        [scope, "all"].contains(&self.field("project"))
    }

    /// Whether the note is pinned.
    pub fn is_pinned(&self) -> bool {
        self.field("pinned") == "true"
    }

    /// Whether the note is tagged `to-repo`, a debt to move into a repository.
    pub fn is_debt(&self) -> bool {
        self.field("tags")
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .any(|tag| tag.trim() == "to-repo")
    }

    /// Where a debt moves to, from its last `**Moves to:**` line.
    pub fn destination(&self) -> Option<&str> {
        self.body
            .lines()
            .rev()
            .find_map(|line| line.strip_prefix("**Moves to:**"))
            .map(str::trim)
    }

    /// The `updated` date, when it is a valid `YYYY-MM-DD`.
    pub fn updated(&self) -> Option<Date> {
        let text = self.field("updated");
        text.parse::<Date>()
            .ok()
            .filter(|date| date.to_string() == text)
    }

    /// The note's `# ` heading, or its file name when it has none.
    pub fn title(&self) -> String {
        self.body
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .map_or_else(|| self.stem(), |title| title.trim().to_owned())
    }

    /// The note's file name without `.md`.
    pub fn stem(&self) -> String {
        self.path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }

    /// The created date, followed by the updated one when it differs.
    pub fn age(&self) -> String {
        let created = self.field("created");
        let updated = self.field("updated");
        if updated.is_empty() || updated == created {
            created.to_owned()
        } else {
            format!("{created} → {updated}")
        }
    }

    /// The key that orders notes by id number, with an invalid id last.
    pub fn id_order(&self) -> (u64, &str) {
        let id = self.field("id");
        let number = id
            .strip_prefix(ID_PREFIX)
            .and_then(|digits| digits.parse().ok())
            .unwrap_or(u64::MAX);
        (number, id)
    }

    fn parse(path: PathBuf, text: &str) -> Option<Self> {
        let mut lines = text.split_inclusive('\n');
        if lines.next()?.trim_end() != "---" {
            return None;
        }
        let mut fields = Vec::new();
        let mut consumed = text.find('\n')? + 1;
        for line in lines {
            consumed += line.len();
            let line = line.trim_end();
            if line == "---" {
                let body = text[consumed..].trim().to_owned();
                return Some(Self { path, fields, body });
            }
            if let Some((key, value)) = line.split_once(':') {
                fields.push((key.trim().to_owned(), value.trim().to_owned()));
            }
        }
        None
    }
}

/// A note about to be written.
pub struct NewNote<'arguments> {
    pub scope: &'arguments str,
    pub kind: &'arguments str,
    pub title: &'arguments str,
    pub why: &'arguments str,
    pub tags: Vec<&'arguments str>,
    pub pinned: bool,
    pub source: &'arguments str,
    pub created: String,
    pub updated: String,
    pub body: &'arguments str,
}

impl NewNote<'_> {
    /// The note's file text, with the id given.
    pub fn text(&self, id: &str) -> String {
        let tags = if self.tags.is_empty() {
            String::new()
        } else {
            format!("[{}]", self.tags.join(", "))
        };
        let pinned = if self.pinned { "true" } else { "" };
        let fields = [
            ("id", id),
            ("type", self.kind),
            ("project", self.scope),
            ("why", self.why),
            ("tags", &tags),
            ("pinned", pinned),
            ("source", self.source),
            ("created", &self.created),
            ("updated", &self.updated),
        ];
        let mut text = String::from("---\n");
        for (key, value) in fields.iter().filter(|(_, value)| !value.is_empty()) {
            text += &format!("{key}: {value}\n");
        }
        text + &format!("---\n\n# {}\n\n{}\n", self.title, self.body)
    }
}

/// The id with the number, padded to four digits.
pub fn id(number: u64) -> String {
    format!("{ID_PREFIX}{number:04}")
}

/// Whether the text is an id: the prefix and at least four digits.
pub fn is_valid_id(id: &str) -> bool {
    id.strip_prefix(ID_PREFIX)
        .is_some_and(|digits| digits.len() >= 4 && digits.bytes().all(|byte| byte.is_ascii_digit()))
}

/// The text with every run of whitespace turned into one space.
pub fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The title on one line.
///
/// # Errors
///
/// When it is empty or longer than 120 characters.
pub fn title(raw: &str) -> Result<String, Failure> {
    let title = one_line(raw);
    if !(1..=120).contains(&title.chars().count()) {
        return Err(Failure::NoteTitleInvalid);
    }
    Ok(title)
}

/// The body piped on stdin, trimmed.
///
/// # Errors
///
/// When nothing is piped or the body is empty.
pub fn read_body() -> Result<String, Failure> {
    let mut body = String::new();
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        stdin
            .lock()
            .read_to_string(&mut body)
            .map_err(|_| Failure::NoteBodyMissing)?;
    }
    let body = body.trim();
    if body.is_empty() {
        return Err(Failure::NoteBodyMissing);
    }
    Ok(body.to_owned())
}

/// Writes the note with the next free id, under a lock, and returns its path under the vault.
///
/// # Errors
///
/// When the vault cannot be read, the lock cannot be taken, or the file exists or cannot be written.
pub fn write(vault: &Path, note: &NewNote) -> Result<PathBuf, Failure> {
    let _lock = lock(vault)?;
    let id = id(highest_id(vault, &read_all(vault)?)? + 1);
    let slug = slug(note.title);
    let file_name = if slug.is_empty() {
        format!("{id}.md")
    } else {
        format!("{id}-{slug}.md")
    };
    let relative = PathBuf::from(format!("notes/{}/{file_name}", note.scope));
    let unwritable = |error| Failure::NoteUnwritable(relative.clone(), error);
    fs::create_dir_all(vault.join("notes").join(note.scope)).map_err(unwritable)?;
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(vault.join(&relative))
    {
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(Failure::NoteExists(relative));
        }
        opened => opened.map_err(unwritable)?,
    };
    file.write_all(note.text(&id).as_bytes())
        .map_err(unwritable)?;
    Ok(relative)
}

fn slug(title: &str) -> String {
    let folded = deunicode::deunicode(title).to_lowercase();
    let words: Vec<&str> = folded
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    words.join("-").chars().take(60).collect()
}

fn lock(vault: &Path) -> Result<File, Failure> {
    let name = "tightship-new.lock";
    let path = match git::run(vault, &["rev-parse", "--git-path", name]) {
        Ok(path) => vault.join(path.trim()),
        // A vault that is not a git repository yet still needs the lock, as a hidden file at its root.
        Err(_) => vault.join(format!(".{name}")),
    };
    let unwritable = |error| Failure::NoteUnwritable(path.clone(), error);
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(unwritable)?;
    file.lock().map_err(unwritable)?;
    Ok(file)
}

/// The highest id number among the notes' file names and fields, or 0.
///
/// # Errors
///
/// When the `notes/` folder cannot be read.
pub fn highest_id(vault: &Path, notes: &[Note]) -> Result<u64, Failure> {
    let files = markdown_files(vault, &vault.join("notes"))?;
    let from_names = files
        .iter()
        .filter_map(|file| file.file_stem())
        .map(|stem| stem.to_string_lossy().into_owned());
    let from_fields = notes.iter().map(|note| note.field("id").to_owned());
    Ok(from_names
        .chain(from_fields)
        .filter_map(|name| {
            name.strip_prefix(ID_PREFIX)?
                .split('-')
                .next()?
                .parse()
                .ok()
        })
        .max()
        .unwrap_or(0))
}

/// Every note under `notes/` whose front matter parses.
///
/// # Errors
///
/// When a file cannot be read.
pub fn read_all(vault: &Path) -> Result<Vec<Note>, Failure> {
    let mut notes = Vec::new();
    for (relative, text) in texts(vault)? {
        notes.extend(Note::parse(relative, &text));
    }
    Ok(notes)
}

/// The files whose front matter opens and never closes.
///
/// # Errors
///
/// When a file cannot be read.
pub fn unclosed(vault: &Path) -> Result<Vec<PathBuf>, Failure> {
    let mut files = Vec::new();
    for (relative, text) in texts(vault)? {
        let opens = text
            .lines()
            .next()
            .is_some_and(|line| line.trim_end() == "---");
        if opens && Note::parse(relative.clone(), &text).is_none() {
            files.push(relative);
        }
    }
    Ok(files)
}

/// Rewrites each note's text with the function, writing only the ones it changed.
///
/// # Errors
///
/// When a note cannot be read or written.
pub fn rewrite_all(vault: &Path, rewrite: impl Fn(&Path, &str) -> String) -> Result<(), Failure> {
    for file in markdown_files(vault, &vault.join("notes"))? {
        let path = vault.join(&file);
        let text = fs::read_to_string(&path)
            .map_err(|error| Failure::NoteUnreadable(file.clone(), error))?;
        let rewritten = rewrite(&file, &text);
        if rewritten != text {
            fs::write(&path, rewritten).map_err(|error| Failure::NoteUnwritable(file, error))?;
        }
    }
    Ok(())
}

fn texts(vault: &Path) -> Result<Vec<(PathBuf, String)>, Failure> {
    let mut texts = Vec::new();
    for relative in markdown_files(vault, &vault.join("notes"))? {
        let text = fs::read_to_string(vault.join(&relative))
            .map_err(|error| Failure::NoteUnreadable(relative.clone(), error))?;
        texts.push((relative, text));
    }
    Ok(texts)
}

/// The `.md` files under the folder, as sorted paths under the vault with `/` between parts.
///
/// # Errors
///
/// When a folder cannot be read.
pub fn markdown_files(vault: &Path, folder: &Path) -> Result<Vec<PathBuf>, Failure> {
    let mut files = Vec::new();
    collect_markdown(vault, folder, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_markdown(vault: &Path, folder: &Path, files: &mut Vec<PathBuf>) -> Result<(), Failure> {
    let unreadable = |error| Failure::VaultUnreadable(vault.to_path_buf(), error);
    let entries = match fs::read_dir(folder) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        entries => entries.map_err(unreadable)?,
    };
    for entry in entries {
        let entry = entry.map_err(unreadable)?;
        let path = entry.path();
        if entry.file_type().map_err(unreadable)?.is_dir() {
            if entry.file_name() == ".git" {
                continue;
            }
            collect_markdown(vault, &path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "md") {
            let mut relative = OsString::new();
            for (index, part) in path.strip_prefix(vault).unwrap_or(&path).iter().enumerate() {
                if index > 0 {
                    relative.push("/");
                }
                relative.push(part);
            }
            files.push(PathBuf::from(relative));
        }
    }
    Ok(())
}
