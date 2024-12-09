use std::{
    fmt::{self, Display, Formatter},
    io,
};

use crate::protocol::headers::MessageType;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    Application(String),
    Protocol(String),
    InternalServer(String),
    UnexpectedMessageType { expected: MessageType, received: MessageType },
    MissingHeader(&'static str),
    BufferTooLarge { size: usize, max_size: usize },
    EnvVarNotSet(&'static str),
    ChecksumMismatch,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Protocol(e) => write!(f, "Protocol Error: {e}"),
            Self::Application(e) => write!(f, "Application Error: {e}"),
            Self::InternalServer(e) => write!(f, "Internal Server Error: {e}"),
            Self::UnexpectedMessageType { expected, received } => {
                write!(
                    f,
                    "message of unexpected type (`{received}`) received. Expected: `{expected}`"
                )
            }
            Self::MissingHeader(header) => write!(f, "Missing header: `{header}`"),
            Self::BufferTooLarge { size, max_size } => {
                write!(
                    f,
                    "Buffer or string size ({size}) exceeds maximum allowed size ({max_size})"
                )
            }
            Self::EnvVarNotSet(var) => write!(f, "Environment variable `{var}` not set"),
            Self::ChecksumMismatch => write!(f, "Checksum mismatch"),
        }
    }
}
