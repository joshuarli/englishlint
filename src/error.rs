use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ConfigError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    InvalidSyntax {
        path: PathBuf,
        line: usize,
        message: String,
    },
    InvalidValue {
        path: PathBuf,
        line: usize,
        key: String,
        value: String,
        message: String,
    },
    UnknownKey {
        path: PathBuf,
        line: usize,
        section: String,
        key: String,
    },
    UnknownSection {
        path: PathBuf,
        line: usize,
        section: String,
    },
    UnknownRule {
        path: PathBuf,
        line: usize,
        value: String,
    },
    UnknownGlossaryConcept {
        path: PathBuf,
        line: usize,
        concept: String,
    },
    FileTooLarge {
        path: PathBuf,
        bytes: u64,
        limit: usize,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::InvalidSyntax {
                path,
                line,
                message,
            } => write!(f, "{}:{}: {message}", path.display(), line),
            Self::InvalidValue {
                path,
                line,
                key,
                value,
                message,
            } => write!(
                f,
                "{}:{}: invalid {key} '{value}': {message}",
                path.display(),
                line
            ),
            Self::UnknownKey {
                path,
                line,
                section,
                key,
            } => write!(
                f,
                "{}:{}: unknown [{section}] key '{key}'",
                path.display(),
                line
            ),
            Self::UnknownSection {
                path,
                line,
                section,
            } => write!(
                f,
                "{}:{}: unknown section [{section}]",
                path.display(),
                line
            ),
            Self::UnknownRule { path, line, value } => {
                write!(f, "{}:{}: unknown rule '{value}'", path.display(), line)
            }
            Self::UnknownGlossaryConcept {
                path,
                line,
                concept,
            } => write!(
                f,
                "{}:{}: unknown glossary concept '{concept}'",
                path.display(),
                line
            ),
            Self::FileTooLarge { path, bytes, limit } => write!(
                f,
                "{}: file is {bytes} bytes; maximum is {limit}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug)]
pub enum LintError {
    Io { path: PathBuf, source: io::Error },
    Walk(String),
    Config(ConfigError),
}

impl fmt::Display for LintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::Walk(message) => write!(f, "walk error: {message}"),
            Self::Config(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for LintError {}
impl From<ConfigError> for LintError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}
