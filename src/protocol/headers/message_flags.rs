use enumflags2::{bitflags, BitFlags};

/// Flags that can be set on a message.
#[bitflags]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageFlags {
    /// The connection has been accepted by the server.
    ConnectionAccepted = 0b01,
    /// The stream should be terminated after this message.
    TerminateStream = 0b10,
}

impl MessageFlags {
    /// Returns a `BitFlags` with no flags set.
    pub fn none() -> BitFlags<Self> {
        BitFlags::default()
    }

    /// Returns a `BitFlags` with both flags set.
    pub fn both() -> BitFlags<Self> {
        Self::ConnectionAccepted | Self::TerminateStream
    }
}
