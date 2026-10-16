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
    HandoffBodyIncomplete(Vec<&'static str>),
    HandoffUnremovable(PathBuf, io::Error),
    LintFailed,
    SyncUnfinished(PathBuf),
    SyncNoUpstream {
        vault: PathBuf,
        branch: String,
        remote: String,
    },
    SyncConflict(PathBuf, Vec<String>),
    SyncGitFailed(PathBuf, &'static str, String),
    StatusInputInvalid(String),
    StatusLineNotSet(&'static str),
    SettingsUnreadable(PathBuf, io::Error),
    SettingsInvalid(PathBuf),
    SettingsUnwritable(PathBuf, io::Error),
    StatusRecordInvalid(PathBuf),
    StatusRecordUnwritable(PathBuf, io::Error),
    OptionsInvalid(PathBuf),
    OptionsUnreadable(PathBuf, io::Error),
    ContractFileUnreadable(PathBuf, io::Error),
    HookOutputUnreadable(PathBuf, io::Error),
    HookOutputInvalid(PathBuf),
    Usage(String),
    OptionsIo(String),
    OptionsInvalidValue(String),
    PluginRootUnset,
    ContractPartNameInvalid(String),
    ContractPartMissing(PathBuf),
    ContractPartUnreadable(PathBuf, io::Error),
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
            Self::HandoffBodyIncomplete(missing) => write!(
                formatter,
                "the handoff's body has no {}; each of **Goal:**, **Done:**, **Next step:**, **Open decisions:**, and **Repo state:** starts a line of it",
                missing.join(", ")
            ),
            Self::HandoffUnremovable(path, error) => {
                write!(formatter, "cannot remove {}: {error}", path.display())
            }
            Self::LintFailed => write!(formatter, "the lint found problems in the vault"),
            Self::SyncUnfinished(vault) => write!(
                formatter,
                "a rebase or merge is under way in {vault}; finish it with `git -C {vault} rebase --continue` or `git -C {vault} merge --continue`, then run `tightship vault sync` again",
                vault = vault.display()
            ),
            Self::SyncNoUpstream {
                vault,
                branch,
                remote,
            } => write!(
                formatter,
                "the vault's branch {branch} tracks no remote branch; run `git -C {vault} push -u {remote} {branch}` once, then run `tightship vault sync` again",
                vault = vault.display()
            ),
            Self::SyncConflict(vault, paths) => write!(
                formatter,
                "these notes conflict with the remote: {paths}; resolve each, run `git -C {vault} add` on it and `git -C {vault} rebase --continue`, then run `tightship vault sync` again",
                paths = paths.join(", "),
                vault = vault.display()
            ),
            Self::SyncGitFailed(vault, step, message) => write!(
                formatter,
                "git {step} failed in {}: {message}",
                vault.display()
            ),
            Self::StatusInputInvalid(message) => write!(
                formatter,
                "the status line input is not a JSON object: {message}"
            ),
            Self::StatusLineNotSet(variable) => write!(
                formatter,
                "{variable} is not set; run the exact command the session start printed"
            ),
            Self::SettingsUnreadable(path, error) => write!(
                formatter,
                "cannot read the settings file {}: {error}",
                path.display()
            ),
            Self::SettingsInvalid(path) => write!(
                formatter,
                "the settings file {} is not a JSON object; nothing changed",
                path.display()
            ),
            Self::SettingsUnwritable(path, error) => write!(
                formatter,
                "cannot write the settings file {}: {error}; nothing changed",
                path.display()
            ),
            Self::StatusRecordInvalid(path) => write!(
                formatter,
                "the status line record {} is damaged; delete it and start a new session",
                path.display()
            ),
            Self::StatusRecordUnwritable(path, error) => write!(
                formatter,
                "cannot write the status line record {}: {error}",
                path.display()
            ),
            Self::OptionsInvalid(path) => {
                write!(formatter, "{} is not valid TOML", path.display())
            }
            Self::OptionsUnreadable(path, error) => {
                write!(formatter, "cannot read {}: {error}", path.display())
            }
            Self::ContractFileUnreadable(path, error) => {
                write!(formatter, "cannot read {}: {error}", path.display())
            }
            Self::HookOutputUnreadable(path, error) => {
                write!(formatter, "cannot read {}: {error}", path.display())
            }
            Self::HookOutputInvalid(path) => write!(
                formatter,
                "{} is not JSON with a \"{{text}}\" slot for the hook's output",
                path.display()
            ),
            Self::Usage(problem) => write!(formatter, "{problem}\n\n{}", crate::USAGE),
            Self::OptionsIo(message) => write!(formatter, "{message}"),
            Self::OptionsInvalidValue(name) => {
                write!(formatter, "{name} must be true or false")
            }
            Self::PluginRootUnset => write!(
                formatter,
                "CURSOR_PLUGIN_ROOT is not set; hooks run only inside the plugin"
            ),
            Self::ContractPartNameInvalid(name) => {
                write!(formatter, "contract part name {:?} is invalid", name)
            }
            Self::ContractPartMissing(path) => write!(
                formatter,
                "contract part {} is missing",
                path.display()
            ),
            Self::ContractPartUnreadable(path, error) => write!(
                formatter,
                "cannot read contract part {}: {error}",
                path.display()
            ),
        }
    }
}
