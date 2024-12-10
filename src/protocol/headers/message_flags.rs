use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum MessageFlags {
    #[default]
    None = 0b00,
    ConnectionAccepted = 0b01,
    TerminateStream = 0b10,
    Both = 0b11,
}

impl TryFrom<i32> for MessageFlags {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::ConnectionAccepted),
            2 => Ok(Self::TerminateStream),
            3 => Ok(Self::Both),
            _ => Err(Error::Protocol("Invalid message flags".into())),
        }
    }
}
