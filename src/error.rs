use core::fmt::{self, Display, Formatter};
use std::io;

use crate::protocol::headers::MessageType;

/// Error type for the AWS Greengrass Nucleus IPC client.
#[derive(Debug)]
pub enum Error {
    /// An IO error occurred.
    Io(io::Error),
    /// A JSON error occurred.
    Json(serde_json::Error),
    /// An application error occurred.
    Application(String),
    /// A protocol error occurred.
    Protocol(String),
    /// An internal server error occurred.
    InternalServer(String),
    /// An error occurred because a message of an unexpected type was received.
    UnexpectedMessageType {
        /// The expected message type.
        expected: MessageType,
        /// The received message type.
        received: MessageType,
    },
    /// Missing header from a message.
    MissingHeader(&'static str),
    /// Buffer or string size exceeds maximum allowed size.
    BufferTooLarge {
        /// The size of the buffer or string.
        size: usize,
        /// The maximum allowed size.
        max_size: usize,
    },
    /// An environment variable was not set.
    EnvVarNotSet(&'static str),
    /// Checksum mismatch.
    ChecksumMismatch,
    /// Connection refused by the server.
    ConnectionRefused,
}

/// Result type for the AWS Greengrass Nucleus IPC client.
pub type Result<T> = core::result::Result<T, Error>;

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
            Self::ConnectionRefused => write!(f, "Connection refused by the server"),
        }
    }
}
