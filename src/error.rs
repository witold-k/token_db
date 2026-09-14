use std::fmt;
use std::io;

/// Errors returned by `token_db`.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A token count would exceed `u64::MAX`.
    CountOverflow,
    /// The database contains more tokens than can be represented by `TokenId`.
    TooManyTokens,
    /// Persisted data does not use the expected file format.
    InvalidFormat(&'static str),
    /// Persisted data contains the same token more than once.
    DuplicateToken(String),
    /// An I/O operation failed.
    Io(io::Error),
}

/// Result type used by `token_db`.
pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOverflow => write!(f, "token count overflow"),
            Self::TooManyTokens => write!(f, "too many unique tokens"),
            Self::InvalidFormat(message) => write!(f, "invalid token database: {message}"),
            Self::DuplicateToken(token) => {
                write!(f, "invalid token database: duplicate token {token:?}")
            }
            Self::Io(error) => write!(f, "I/O error: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

