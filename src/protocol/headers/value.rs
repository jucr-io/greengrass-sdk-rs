use endi::{ReadBytes, WriteBytes};
use std::{borrow::Cow, io::Write};

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'v> {
    Bool(bool),
    Byte(u8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    ByteBuffer(Cow<'v, [u8]>),
    String(Cow<'v, str>),
    Timestamp(i64),
    Uuid(uuid::Uuid),
}

impl Value<'_> {
    pub const fn r#type(&self) -> u8 {
        match self {
            Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            Value::Byte(_) => 2,
            Value::Int16(_) => 3,
            Value::Int32(_) => 4,
            Value::Int64(_) => 5,
            Value::ByteBuffer(_) => 6,
            Value::String(_) => 7,
            Value::Timestamp(_) => 8,
            Value::Uuid(_) => 9,
        }
    }

    pub fn write_as_bytes(&self, writer: &mut impl Write) -> Result<usize> {
        // The type of the header value.
        writer.write_u8(endi::Endian::Big, self.r#type())?;
        let mut bytes_written = 1;

        // The header value.
        match self {
            // No field value for booleans. The type already covers it as there are separate types
            // for true and false.
            Value::Bool(_) => (),
            Value::Byte(b) => {
                writer.write_u8(endi::Endian::Big, *b)?;
                bytes_written += 1;
            }
            Value::Int16(i) => {
                writer.write_i16(endi::Endian::Big, *i)?;
                bytes_written += 2;
            }
            Value::Int32(i) => {
                writer.write_i32(endi::Endian::Big, *i)?;
                bytes_written += 4;
            }
            Value::Int64(i) => {
                writer.write_i64(endi::Endian::Big, *i)?;
                bytes_written += 8;
            }
            Value::ByteBuffer(bytes) => {
                let len = u16::try_from(bytes.len()).map_err(|_| Error::BufferTooLarge {
                    size: bytes.len(),
                    max_size: u16::MAX as usize,
                })?;
                writer.write_u16(endi::Endian::Big, len)?;
                bytes_written += 2;

                bytes_written += writer.write(bytes)?;
            }
            Value::String(s) => {
                let len = u16::try_from(s.len()).map_err(|_| Error::BufferTooLarge {
                    size: s.len(),
                    max_size: u16::MAX as usize,
                })?;
                writer.write_u16(endi::Endian::Big, len)?;
                bytes_written += 2;

                bytes_written += writer.write(s.as_bytes())?;
            }
            Value::Timestamp(ts) => {
                writer.write_i64(endi::Endian::Big, *ts)?;
                bytes_written += 8;
            }
            Value::Uuid(uuid) => {
                writer.write(uuid.as_bytes())?;
                bytes_written += 16;
            }
        }

        Ok(bytes_written)
    }

    pub fn size_in_bytes(&self) -> Result<u32> {
        // All values have a type byte so that's why 5 bytes for i32 for example.
        Ok(match self {
            Value::Bool(_) => 0,
            Value::Byte(_) => 2,
            Value::Int16(_) => 3,
            Value::Int32(_) => 5,
            Value::Int64(_) | Value::Timestamp(_) => 9,
            Value::ByteBuffer(bytes) => bytes
                .len()
                .try_into()
                .map(|len: u32| len + 3)
                .map_err(|_| Error::Protocol("buffer length too large".into()))?,
            Value::String(s) => s
                .len()
                .try_into()
                .map(|len: u32| len + 3)
                .map_err(|_| Error::Protocol("string length too large".into()))?,
            Value::Uuid(_) => 17,
        })
    }

    pub fn to_owned(&self) -> Value<'static> {
        match self {
            Value::Bool(b) => Value::Bool(*b),
            Value::Byte(b) => Value::Byte(*b),
            Value::Int16(i) => Value::Int16(*i),
            Value::Int32(i) => Value::Int32(*i),
            Value::Int64(i) => Value::Int64(*i),
            Value::ByteBuffer(bytes) => Value::ByteBuffer(Cow::Owned(bytes.to_vec())),
            Value::String(s) => Value::String(Cow::Owned(s.to_string())),
            Value::Timestamp(ts) => Value::Timestamp(*ts),
            Value::Uuid(uuid) => Value::Uuid(*uuid),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_byte(&self) -> Option<u8> {
        match self {
            Value::Byte(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int16(&self) -> Option<i16> {
        match self {
            Value::Int16(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_int32(&self) -> Option<i32> {
        match self {
            Value::Int32(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_int64(&self) -> Option<i64> {
        match self {
            Value::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_byte_buffer(&self) -> Option<&[u8]> {
        match self {
            Value::ByteBuffer(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_timestamp(&self) -> Option<i64> {
        match self {
            Value::Timestamp(ts) => Some(*ts),
            _ => None,
        }
    }

    pub fn as_uuid(&self) -> Option<uuid::Uuid> {
        match self {
            Value::Uuid(uuid) => Some(*uuid),
            _ => None,
        }
    }
}

impl<'v> Value<'v> {
    pub fn from_bytes(bytes: &mut &'v [u8]) -> Result<Self> {
        let r#type = bytes
            .get(0)
            .copied()
            .ok_or(Error::Protocol("Invalid header value: missing type".into()))?;
        *bytes = &bytes[1..];

        match r#type {
            0 => Ok(Value::Bool(false)),
            1 => Ok(Value::Bool(true)),
            2 => bytes
                .read_u8(endi::Endian::Big)
                .map(Value::Byte)
                .map_err(|_| Error::Protocol("Invalid header value: missing byte".into())),
            3 => bytes
                .read_i16(endi::Endian::Big)
                .map(Value::Int16)
                .map_err(|_| Error::Protocol("Invalid header value: missing int16".into())),
            4 => bytes
                .read_i32(endi::Endian::Big)
                .map(Value::Int32)
                .map_err(|_| Error::Protocol("Invalid header value: missing int32".into())),
            5 => bytes
                .read_i64(endi::Endian::Big)
                .map(Value::Int64)
                .map_err(|_| Error::Protocol("Invalid header value: missing int64".into())),
            6 => {
                let len = bytes.read_u16(endi::Endian::Big).map_err(|_| {
                    Error::Protocol("Invalid header value: missing byte buffer length".into())
                })? as usize;

                let value = bytes
                    .get(..len)
                    .ok_or(Error::Protocol("Invalid header value: missing byte buffer".into()))
                    .map(Cow::Borrowed)
                    .map(Value::ByteBuffer)?;
                *bytes = &bytes[len..];
                Ok(value)
            }
            7 => {
                let len = bytes.read_u16(endi::Endian::Big).map_err(|_| {
                    Error::Protocol("Invalid header value: missing string length".into())
                })? as usize;

                let value = bytes
                    .get(..len)
                    .ok_or(Error::Protocol("Invalid header value: missing string".into()))
                    .map(|slice| {
                        let string = core::str::from_utf8(slice).map_err(|_| {
                            Error::Protocol("Invalid header value: invalid UTF-8".into())
                        })?;
                        Ok(Value::String(Cow::Borrowed(string)))
                    })?;
                *bytes = &bytes[len..];
                value
            }
            8 => bytes
                .read_i64(endi::Endian::Big)
                .map(Value::Timestamp)
                .map_err(|_| Error::Protocol("Invalid header value: missing timestamp".into())),
            9 => {
                let array = bytes
                    .get(..16)
                    .ok_or(Error::Protocol("Invalid header value: missing UUID".into()))
                    .map(|slice| slice.try_into().unwrap())?;
                let value = uuid::Uuid::from_slice(array)
                    .map(Value::Uuid)
                    .map_err(|_| Error::Protocol("Invalid header value: invalid UUID".into()))?;
                *bytes = &bytes[16..];
                Ok(value)
            }
            _ => Err(Error::Protocol("Invalid header value: unknown type".into())),
        }
    }
}
