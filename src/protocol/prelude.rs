use crc::{Crc, CRC_32_ISO_HDLC};
use endi::{ReadBytes, WriteBytes};
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prelude {
    headers_len: usize,
    total_len: usize,
    crc_checksum: u32,
}

impl Prelude {
    pub fn new(total_len: usize, headers_len: usize) -> io::Result<Self> {
        let total_len = total_len
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid length"))?;
        let headers_len = headers_len
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid length"))?;
        let mut bytes = [0; 8];
        let mut bytes_buf = &mut bytes[..];
        bytes_buf.write_u32(endi::Endian::Big, total_len).unwrap();
        bytes_buf.write_u32(endi::Endian::Big, headers_len).unwrap();
        let crc32 = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let crc_checksum = crc32.checksum(&bytes);

        Ok(Self {
            total_len: total_len as usize,
            headers_len: headers_len as usize,
            crc_checksum,
        })
    }

    pub fn from_bytes(bytes: &mut &[u8]) -> io::Result<Self> {
        let crc32 = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let prelude_checksum = crc32.checksum(&bytes[..8]);
        let total_len = bytes.read_u32(endi::Endian::Big)?;
        let headers_len = bytes.read_u32(endi::Endian::Big)?;
        let crc_checksum = bytes.read_u32(endi::Endian::Big)?;

        if headers_len > total_len {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length"));
        }

        if prelude_checksum != crc_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid prelude checksum",
            ));
        }

        Ok(Self {
            total_len: total_len as usize,
            headers_len: headers_len as usize,
            crc_checksum,
        })
    }

    pub fn write_as_bytes(&self, writer: &mut impl Write) -> io::Result<()> {
        // Safe to cast because our constructor ensures that the values are within u32 range.
        writer.write_u32(endi::Endian::Big, self.total_len as u32)?;
        writer.write_u32(endi::Endian::Big, self.headers_len as u32)?;
        writer.write_u32(endi::Endian::Big, self.crc_checksum)?;

        Ok(())
    }

    pub fn headers_len(&self) -> usize {
        self.headers_len
    }

    pub fn total_len(&self) -> usize {
        self.total_len
    }

    pub fn crc_checksum(&self) -> u32 {
        self.crc_checksum
    }
}

pub const PRELUDE_SIZE: usize = 12;
