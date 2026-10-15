use std::fmt;
use std::io;
use std::path::PathBuf;

pub enum Failure {
    VaultNotSet,
    VaultNotAbsolute(PathBuf),
    VaultMissing(PathBuf),
    VaultUnreadable(PathBuf, io::Error),
    NoteUnreadable(PathBuf, io::Error),
    EnvUnwritable(PathBuf, io::Error),
    DataUnwritable(PathBuf, io::Error),
    InitNoGit,
    InitNotEmpty(PathBuf),
    InitUnwritable(PathBuf, io::Error),
    InitGitFailed(PathBuf, String),
    SearchNoTerm,
    NoteScopeInvalid,
    NoteTypeInvalid,
    NoteTitleInvalid,
    NoteWhyShort(usize),
    NoteSourceInvalid,
    NoteCreatedInvalid,
    NoteBodyMissing,
    NoteExists(PathBuf),
    NoteUnwritable(PathBuf, io::Error),
    LintFailed,
    Usage(String),
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VaultNotSet => write!(
                formatter,
                "no vault is set for this session; turn memory on, set the plugin's vault folder, and start a new session"
            ),
            Self::VaultNotAbsolute(value) => write!(
                formatter,
                "the vault folder {} is not an absolute path",
                value.display()
            ),
            Self::VaultMissing(folder) => write!(
                formatter,
                "the vault folder {folder} does not exist; create it with `tightship vault init {folder}`",
                folder = folder.display()
            ),
            Self::VaultUnreadable(folder, error) => write!(
                formatter,
                "cannot read the vault folder {}: {error}",
                folder.display()
            ),
            Self::NoteUnreadable(path, error) => {
                write!(formatter, "cannot read {}: {error}", path.display())
            }
            Self::EnvUnwritable(file, error) => write!(
                formatter,
                "cannot write the session's environment file {}: {error}",
                file.display()
            ),
            Self::DataUnwritable(counter, error) => write!(
                formatter,
                "cannot write the checkpoint counter {}: {error}",
                counter.display()
            ),
            Self::InitNoGit => write!(
                formatter,
                "git is not installed; a vault is a git repository, so nothing was created"
            ),
            Self::InitNotEmpty(folder) => write!(
                formatter,
                "{} already exists and is not an empty folder; nothing changed",
                folder.display()
            ),
            Self::InitUnwritable(folder, error) => write!(
                formatter,
                "cannot write the vault at {}: {error}",
                folder.display()
            ),
            Self::InitGitFailed(folder, message) => write!(
                formatter,
                "git init failed in {}: {message}",
                folder.display()
            ),
            Self::SearchNoTerm => write!(formatter, "search needs at least one term"),
            Self::NoteScopeInvalid => write!(
                formatter,
                "--scope must be 1 to 100 characters, with no / or \\ and no leading dot"
            ),
            Self::NoteTypeInvalid => write!(
                formatter,
                "--type must be one of preference, decision, gotcha, project"
            ),
            Self::NoteTitleInvalid => write!(formatter, "--title must be 1 to 120 characters"),
            Self::NoteWhyShort(words) => write!(
                formatter,
                "--why must state in a full line what not knowing this would cost later, not name a topic (got {words} words)"
            ),
            Self::NoteSourceInvalid => {
                write!(formatter, "--source must be one of user, session, repo")
            }
            Self::NoteCreatedInvalid => write!(formatter, "--created must be a date as YYYY-MM-DD"),
            Self::NoteBodyMissing => write!(formatter, "the note's body must arrive on stdin"),
            Self::NoteExists(path) => write!(formatter, "{} already exists", path.display()),
            Self::NoteUnwritable(path, error) => {
                write!(formatter, "cannot write {}: {error}", path.display())
            }
            Self::LintFailed => write!(formatter, "the lint found problems in the vault"),
            Self::Usage(problem) => write!(formatter, "{problem}\n\n{}", crate::USAGE),
        }
    }
}
