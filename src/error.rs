use std::fmt;
use std::io;

#[derive(Debug)]
pub enum BlendFileError {
    IoError(String),
    InvalidFormat(String),
    UnsupportedVersion(String),
    DnaError(String),
    BlockError(String),
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, BlendFileError>;

impl fmt::Display for BlendFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlendFileError::IoError(msg) => write!(f, "IO error: {msg}"),
            BlendFileError::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            BlendFileError::UnsupportedVersion(msg) => write!(f, "Unsupported version: {msg}"),
            BlendFileError::DnaError(msg) => write!(f, "DNA error: {msg}"),
            BlendFileError::BlockError(msg) => write!(f, "Block error: {msg}"),
            BlendFileError::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for BlendFileError {}

impl From<io::Error> for BlendFileError {
    fn from(err: io::Error) -> Self {
        BlendFileError::IoError(err.to_string())
    }
}

impl From<std::str::Utf8Error> for BlendFileError {
    fn from(err: std::str::Utf8Error) -> Self {
        BlendFileError::ParseError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for BlendFileError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        BlendFileError::ParseError(err.to_string())
    }
}
