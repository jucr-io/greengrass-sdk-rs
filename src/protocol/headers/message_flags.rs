use enumflags2::{bitflags, BitFlags};

#[bitflags]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageFlags {
    ConnectionAccepted = 0b01,
    TerminateStream = 0b10,
}

impl MessageFlags {
    pub fn none() -> BitFlags<Self> {
        BitFlags::default()
    }

    pub fn both() -> BitFlags<Self> {
        Self::ConnectionAccepted | Self::TerminateStream
    }
}
