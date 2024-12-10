use endi::{ReadBytes, WriteBytes};
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
    headers: HashMap<Cow<'h, str>, Value<'h>>,
}

impl<'h> Headers<'h> {
    pub fn new(stream_id: i32, message_type: MessageType, message_flags: MessageFlags) -> Self {
        let mut headers = HashMap::new();
        headers.insert(":stream-id".into(), Value::Int32(stream_id));
        headers.insert(":message-type".into(), Value::Int32(message_type.into()));
        headers.insert(":message-flags".into(), Value::Int32(message_flags as i32));

        Self { headers }
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
        let mut headers = Self { headers: HashMap::new() };

        while !bytes.is_empty() {
            let name = read_header_name_from_bytes(bytes)?;
            let value = Value::from_bytes(bytes)?;

            headers.headers.insert(Cow::Borrowed(name), value);
        }

        // Ensure all mandatory headers are present.
        for (header, convert_fn) in [
            (":stream-id", (&|_| Some(())) as &dyn Fn(_) -> Option<()>),
            (
                ":message-type",
                (&|i| MessageType::try_from(i).ok().map(|_| ())) as &dyn Fn(_) -> Option<()>,
            ),
            (
                ":message-flags",
                (&|i| MessageFlags::try_from(i).ok().map(|_| ())) as &dyn Fn(_) -> Option<()>,
            ),
        ] {
            if headers
                .headers
                .get(header)
                .and_then(|v| match v {
                    Value::Int32(i) => convert_fn(*i),
                    _ => None,
                })
                .is_none()
            {
                return Err(Error::MissingHeader(header));
            }
        }

        Ok(headers)
    }

    pub fn to_owned(&self) -> Headers<'static> {
        let headers =
            self.headers.iter().map(|(k, v)| (Cow::Owned(k.to_string()), v.to_owned())).collect();

        Headers { headers }
    }

    // # SAFETY
    //
    // These getters of the mandatory headers assume that the headers are present and correct as our
    // constructors ensure that.
    //

    pub fn stream_id(&self) -> i32 {
        match self.headers.get(":stream-id").unwrap() {
            Value::Int32(id) => *id,
            _ => unreachable!(),
        }
    }

    pub fn message_type(&self) -> MessageType {
        match self.headers.get(":message-type").unwrap() {
            Value::Int32(t) => MessageType::try_from(*t).unwrap(),
            _ => unreachable!(),
        }
    }

    pub fn message_flags(&self) -> MessageFlags {
        match self.headers.get(":message-flags").unwrap() {
            Value::Int32(f) => MessageFlags::try_from(*f).unwrap(),
            _ => unreachable!(),
        }
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

    std::str::from_utf8(name_bytes)
        .map_err(|_| Error::Protocol("Invalid header name: invalid UTF-8".into()))
}
