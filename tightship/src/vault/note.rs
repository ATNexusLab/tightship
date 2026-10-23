use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use jiff::civil::Date;

use crate::failure::Failure;
use crate::git;

pub struct Note {
    pub path: PathBuf,
    pub fields: Vec<(String, String)>,
    pub body: String,
}

impl Note {
    pub fn field(&self, name: &str) -> &str {
        self.fields
            .iter()
            .find(|(key, _)| key == name)
            .map_or("", |(_, value)| value.as_str())
    }

    pub fn is_pinned(&self) -> bool {
        self.field("pinned") == "true"
    }

    pub fn is_debt(&self) -> bool {
        self.field("tags")
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .any(|tag| tag.trim() == "to-repo")
    }

    pub fn destination(&self) -> Option<&str> {
        self.body
            .lines()
            .rev()
            .find_map(|line| line.strip_prefix("**Moves to:**"))
            .map(str::trim)
    }

    pub fn updated(&self) -> Option<Date> {
        let text = self.field("updated");
        text.parse::<Date>()
            .ok()
            .filter(|date| date.to_string() == text)
    }

    pub fn title(&self) -> String {
        self.body
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .map_or_else(|| self.stem(), |title| title.trim().to_owned())
    }

    pub fn stem(&self) -> String {
        self.path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }

    pub fn age(&self) -> String {
        let created = self.field("created");
        let updated = self.field("updated");
        if updated.is_empty() || updated == created {
            created.to_owned()
        } else {
            format!("{created} → {updated}")
        }
    }

    pub fn id_order(&self) -> (u64, &str) {
        let id = self.field("id");
        let number = id
            .strip_prefix("mem-")
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

pub fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn title(raw: &str) -> Result<String, Failure> {
    let title = one_line(raw);
    if !(1..=120).contains(&title.chars().count()) {
        return Err(Failure::NoteTitleInvalid);
    }
    Ok(title)
}

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

pub fn write(vault: &Path, note: &NewNote) -> Result<PathBuf, Failure> {
    let _lock = lock(vault)?;
    let id = format!("mem-{:04}", highest_id(vault, &read_all(vault)?)? + 1);
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

pub fn slug(title: &str) -> String {
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

pub fn highest_id(vault: &Path, notes: &[Note]) -> Result<u64, Failure> {
    let files = markdown_files(vault, &vault.join("notes"))?;
    let from_names = files
        .iter()
        .filter_map(|file| file.file_stem())
        .map(|stem| stem.to_string_lossy().into_owned());
    let from_fields = notes.iter().map(|note| note.field("id").to_owned());
    Ok(from_names
        .chain(from_fields)
        .filter_map(|name| name.strip_prefix("mem-")?.split('-').next()?.parse().ok())
        .max()
        .unwrap_or(0))
}

pub fn read_all(vault: &Path) -> Result<Vec<Note>, Failure> {
    let mut notes = Vec::new();
    for (relative, text) in texts(vault)? {
        notes.extend(Note::parse(relative, &text));
    }
    Ok(notes)
}

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

fn texts(vault: &Path) -> Result<Vec<(PathBuf, String)>, Failure> {
    let mut texts = Vec::new();
    for relative in markdown_files(vault, &vault.join("notes"))? {
        let text = fs::read_to_string(vault.join(&relative))
            .map_err(|error| Failure::NoteUnreadable(relative.clone(), error))?;
        texts.push((relative, text));
    }
    Ok(texts)
}

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
