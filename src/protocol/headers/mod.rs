use endi::{ReadBytes, WriteBytes};
use enumflags2::{BitFlag, BitFlags};
use std::{borrow::Cow, collections::HashMap, io::Write};

use crate::{Error, Result};

mod value;
pub use value::Value;
mod message_flags;
pub use message_flags::MessageFlags;
mod message_type;
pub use message_type::MessageType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Headers<'h> {
    stream_id: i32,
    message_type: MessageType,
    message_flags: BitFlags<MessageFlags>,
    headers: HashMap<Cow<'h, str>, Value<'h>>,
}

impl<'h> Headers<'h> {
    pub fn new(
        stream_id: i32,
        message_type: MessageType,
        message_flags: BitFlags<MessageFlags>,
    ) -> Self {
        let mut headers = HashMap::new();
        headers.insert(":stream-id".into(), Value::Int32(stream_id));
        headers.insert(":message-type".into(), Value::Int32(message_type.into()));
        headers.insert(":message-flags".into(), Value::Int32(message_flags.bits() as i32));

        Self { headers, stream_id, message_type, message_flags }
    }

    pub fn insert<N>(&mut self, name: N, value: Value<'h>)
    where
        N: Into<Cow<'h, str>>,
    {
        self.headers.insert(name.into(), value);
    }

    pub fn get(&self, name: &'static str) -> Option<&Value> {
        self.headers.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.headers.iter().map(|(k, v)| (k.as_ref(), v))
    }

    /// Write into the given writer the headers in the IPC wire format.
    ///
    /// Returns the number of bytes written on success, or `Err(())` on failure.
    pub fn write_as_bytes(&self, writer: &mut impl Write) -> Result<usize> {
        let mut bytes_written = 0;

        for (name, value) in self.headers.iter() {
            bytes_written += write_header_as_bytes(name, value, writer)?;
        }

        Ok(bytes_written)
    }

    /// The size in bytes of the headers in the IPC wire format.
    pub fn size_in_bytes(&self) -> Result<u32> {
        self.headers.iter().try_fold(0, |acc, (name, value)| {
            name.len()
                .try_into()
                .map_err(|_| Error::Protocol("Header name too long".into()))
                .map(|len: u32| acc + 1 + len)
                .and_then(|len| value.size_in_bytes().map(|v_len| len + v_len))
        })
    }

    pub fn from_bytes(bytes: &mut &'h [u8]) -> Result<Self> {
        let mut headers = HashMap::new();

        while !bytes.is_empty() {
            let name = read_header_name_from_bytes(bytes)?;
            let value = Value::from_bytes(bytes)?;

            headers.insert(Cow::Borrowed(name), value);
        }

        // Ensure all mandatory headers are present.
        let stream_id = headers
            .get(":stream-id")
            .and_then(Value::as_int32)
            .ok_or_else(|| Error::MissingHeader(":stream-id"))?;
        let message_type = headers
            .get(":message-type")
            .and_then(Value::as_int32)
            .ok_or_else(|| Error::MissingHeader(":message-type"))
            .and_then(TryInto::try_into)?;
        let message_flags = headers
            .get(":message-flags")
            .and_then(Value::as_int32)
            .ok_or_else(|| Error::MissingHeader(":message-flags"))
            .and_then(|i| {
                MessageFlags::from_bits(i as u32).map_err(|e| Error::Protocol(e.to_string()))
            })?;

        Ok(Self { headers, stream_id, message_type, message_flags })
    }

    pub fn to_owned(&self) -> Headers<'static> {
        let headers =
            self.headers.iter().map(|(k, v)| (Cow::Owned(k.to_string()), v.to_owned())).collect();

        Headers {
            stream_id: self.stream_id,
            message_type: self.message_type,
            message_flags: self.message_flags,
            headers,
        }
    }

    pub fn stream_id(&self) -> i32 {
        self.stream_id
    }

    pub fn message_type(&self) -> MessageType {
        self.message_type
    }

    pub fn message_flags(&self) -> BitFlags<MessageFlags> {
        self.message_flags
    }
}

fn write_header_as_bytes(name: &str, value: &Value<'_>, writer: &mut impl Write) -> Result<usize> {
    let mut bytes_written = 0;

    bytes_written += write_header_name_as_bytes(name, writer)?;
    bytes_written += value.write_as_bytes(writer)?;

    Ok(bytes_written)
}

fn write_header_name_as_bytes(name: &str, writer: &mut impl Write) -> Result<usize> {
    let mut bytes_written = 0;

    // The length of the header name.
    let len =
        u8::try_from(name.len()).map_err(|_| Error::Protocol("header name too large".into()))?;
    writer.write_u8(endi::Endian::Big, len)?;
    bytes_written += 1;

    // The header name.
    bytes_written += writer.write(name.as_bytes())?;

    Ok(bytes_written)
}

fn read_header_name_from_bytes<'n>(bytes: &mut &'n [u8]) -> Result<&'n str> {
    let len = bytes
        .read_u8(endi::Endian::Big)
        .map_err(|_| Error::Protocol("Invalid header name: missing length".into()))?
        as usize;
    let name_bytes =
        bytes.get(..len).ok_or(Error::Protocol("Invalid header name: missing name".into()))?;
    *bytes = &bytes[len..];

    core::str::from_utf8(name_bytes)
        .map_err(|_| Error::Protocol("Invalid header name: invalid UTF-8".into()))
}
