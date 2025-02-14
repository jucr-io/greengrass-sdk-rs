use core::fmt::Display;

use crate::{Error, Result};

/// The type of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MessageType {
    /// An application message.
    Application = 0,
    /// An application error message.
    ApplicationError = 1,
    /// A ping message.
    Ping = 2,
    /// A pong message.
    Pong = 3,
    /// A connect message.
    Connect = 4,
    /// A connect acknowledgment message.
    ConnectAck = 5,
    /// A protocol error message.
    ProtocolError = 6,
    /// An internal server error message.
    InternalError = 7,
    /// The number of message types.
    Count = 8,
}

impl TryFrom<i32> for MessageType {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::Application),
            1 => Ok(Self::ApplicationError),
            2 => Ok(Self::Ping),
            3 => Ok(Self::Pong),
            4 => Ok(Self::Connect),
            5 => Ok(Self::ConnectAck),
            6 => Ok(Self::ProtocolError),
            7 => Ok(Self::InternalError),
            8 => Ok(Self::Count),
            _ => Err(Error::Protocol("Invalid message type".into())),
        }
    }
}

impl From<MessageType> for i32 {
    fn from(value: MessageType) -> Self {
        value as i32
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MessageType::Application => write!(f, "Application"),
            MessageType::ApplicationError => write!(f, "ApplicationError"),
            MessageType::Ping => write!(f, "Ping"),
            MessageType::Pong => write!(f, "Pong"),
            MessageType::Connect => write!(f, "Connect"),
            MessageType::ConnectAck => write!(f, "ConnectAck"),
            MessageType::ProtocolError => write!(f, "ProtocolError"),
            MessageType::InternalError => write!(f, "InternalError"),
            MessageType::Count => write!(f, "Count"),
        }
    }
}
