//! The problems a command reports, and how a command fails.

use std::fmt;
use std::io;
use std::path::Path;

/// One problem, with the file and line it was found at when it has them.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Diagnostic {
    path: Option<String>,
    line: Option<usize>,
    message: String,
}

impl Diagnostic {
    /// A problem at a line of a file.
    pub fn at_line(path: &str, line: usize, message: String) -> Self {
        Self {
            path: Some(path.to_owned()),
            line: Some(line),
            message,
        }
    }

    /// A problem with a whole file.
    pub fn in_file(path: &str, message: String) -> Self {
        Self {
            path: Some(path.to_owned()),
            line: None,
            message,
        }
    }

    /// A problem that belongs to no file.
    pub fn general(message: String) -> Self {
        Self {
            path: None,
            line: None,
            message,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.path, self.line) {
            (Some(path), Some(line)) => write!(formatter, "{path}:{line}: {}", self.message),
            (Some(path), None) => write!(formatter, "{path}: {}", self.message),
            (None, _) => write!(formatter, "{}", self.message),
        }
    }
}

/// Prefixes an I/O error with the path it happened at.
pub fn with_path(path: &Path) -> impl FnOnce(io::Error) -> io::Error + '_ {
    move |error| io::Error::new(error.kind(), format!("{}: {error}", path.display()))
}

/// Why a command stopped: the checks found problems, or an I/O step failed.
pub enum Failure {
    Checks(Vec<Diagnostic>),
    Io(io::Error),
}

impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Stops the command with every diagnostic so far, sorted, when there is any.
///
/// # Errors
///
/// When `diagnostics` is not empty.
pub fn stop_on_errors(diagnostics: &mut Vec<Diagnostic>) -> Result<(), Failure> {
    if diagnostics.is_empty() {
        return Ok(());
    }
    diagnostics.sort();
    Err(Failure::Checks(std::mem::take(diagnostics)))
}
