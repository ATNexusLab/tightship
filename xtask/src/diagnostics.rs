use std::fmt;
use std::io;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Diagnostic {
    path: Option<String>,
    line: Option<usize>,
    message: String,
}

impl Diagnostic {
    pub fn at_line(path: &str, line: usize, message: String) -> Self {
        Self {
            path: Some(path.to_owned()),
            line: Some(line),
            message,
        }
    }

    pub fn in_file(path: &str, message: String) -> Self {
        Self {
            path: Some(path.to_owned()),
            line: None,
            message,
        }
    }

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

pub fn naming(path: &Path) -> impl FnOnce(io::Error) -> io::Error + '_ {
    move |error| io::Error::new(error.kind(), format!("{}: {error}", path.display()))
}
