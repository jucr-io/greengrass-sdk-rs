use endi::{ReadBytes, WriteBytes};
use std::{borrow::Cow, collections::HashMap, io, io::Write};

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
    pub fn write_as_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
        let mut bytes_written = 0;

        for (name, value) in self.headers.iter() {
            bytes_written += write_header_as_bytes(name, value, writer)?;
        }

        Ok(bytes_written)
    }

    /// The size in bytes of the headers in the IPC wire format.
    pub fn size_in_bytes(&self) -> io::Result<u32> {
        self.headers.iter().try_fold(0, |acc, (name, value)| {
            name.len()
                .try_into()
                .map_err(|_| io::ErrorKind::InvalidInput.into())
                .map(|len: u32| acc + 1 + len)
                .and_then(|len| value.size_in_bytes().map(|v_len| len + v_len))
        })
    }

    pub fn from_bytes(bytes: &mut &'h [u8], headers_len: usize) -> Result<Self, &'static str> {
        let mut headers = Self {
            headers: HashMap::new(),
        };

        while (headers.size_in_bytes().map_err(|_| "too large header")? as usize) < headers_len {
            let name = read_header_name_from_bytes(bytes)?;
            let value = Value::from_bytes(bytes)?;

            headers.headers.insert(Cow::Borrowed(name), value);
        }

        Ok(headers)
    }
}

fn write_header_as_bytes(
    name: &str,
    value: &Value<'_>,
    writer: &mut impl Write,
) -> io::Result<usize> {
    let mut bytes_written = 0;

    bytes_written += write_header_name_as_bytes(name, writer)?;
    bytes_written += value.write_as_bytes(writer)?;

    Ok(bytes_written)
}

fn write_header_name_as_bytes(name: &str, writer: &mut impl Write) -> io::Result<usize> {
    let mut bytes_written = 0;

    // The length of the header name.
    let len = u8::try_from(name.len()).map_err(|_| io::ErrorKind::InvalidInput)?;
    writer.write_u8(endi::Endian::Big, len)?;
    bytes_written += 1;

    // The header name.
    bytes_written += writer.write(name.as_bytes())?;

    Ok(bytes_written)
}

fn read_header_name_from_bytes<'n>(bytes: &mut &'n [u8]) -> Result<&'n str, &'static str> {
    let len = bytes
        .read_u16(endi::Endian::Big)
        .map_err(|_| "Invalid header name: missing length")? as usize;

    let name_bytes = bytes
        .get(..len)
        .ok_or("Invalid header name: missing name")?;
    *bytes = &bytes[len..];

    std::str::from_utf8(name_bytes).map_err(|_| "Invalid header name: invalid UTF-8")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MessageType {
    Application = 0,
    ApplicationError = 1,
    Ping = 2,
    Pong = 3,
    Connect = 4,
    ConnectAck = 5,
    ProtocolError = 6,
    InternalError = 7,
    Count = 8,
}

impl TryFrom<i32> for MessageType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
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
            _ => Err("Invalid message type"),
        }
    }
}

impl From<MessageType> for i32 {
    fn from(value: MessageType) -> Self {
        value as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum MessageFlags {
    #[default]
    None = 0b00,
    ConnectionAccepted = 0b01,
    TerminateStream = 0b10,
    Both = 0b11,
}

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

    fn write_as_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
        let mut bytes_written = 0;

        // The type of the header value.
        writer.write_u8(endi::Endian::Big, self.r#type())?;

        // The header value.
        match self {
            // No field value for booleans. The type already covers it as there are separate types for
            // true and false.
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
                let len = u16::try_from(bytes.len())
                    .map_err(|_| io::ErrorKind::InvalidInput)
                    .map_err(io::Error::from)?;
                writer.write_u16(endi::Endian::Big, len)?;
                bytes_written += 2;

                bytes_written += writer.write(bytes)?;
            }
            Value::String(s) => {
                let len = u16::try_from(s.len())
                    .map_err(|_| io::ErrorKind::InvalidInput)
                    .map_err(io::Error::from)?;
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

    pub fn size_in_bytes(&self) -> io::Result<u32> {
        Ok(match self {
            Value::Bool(_) => 0,
            Value::Byte(_) => 1,
            Value::Int16(_) => 2,
            Value::Int32(_) => 4,
            Value::Int64(_) => 8,
            Value::ByteBuffer(bytes) => bytes
                .len()
                .try_into()
                .map(|len: u32| len + 2)
                .map_err(|_| io::ErrorKind::InvalidInput)?,
            Value::String(s) => s
                .len()
                .try_into()
                .map(|len: u32| len + 2)
                .map_err(|_| io::ErrorKind::InvalidInput)?,
            Value::Timestamp(_) => 8,
            Value::Uuid(_) => 16,
        })
    }
}

impl<'v> Value<'v> {
    pub fn from_bytes(bytes: &mut &'v [u8]) -> Result<Self, &'static str> {
        let r#type = bytes
            .get(0)
            .copied()
            .ok_or("Invalid header value: missing type")?;
        *bytes = &bytes[1..];

        match r#type {
            0 => Ok(Value::Bool(false)),
            1 => Ok(Value::Bool(true)),
            2 => bytes
                .read_u8(endi::Endian::Big)
                .map(Value::Byte)
                .map_err(|_| "Invalid header value: missing byte"),
            3 => bytes
                .read_i16(endi::Endian::Big)
                .map(Value::Int16)
                .map_err(|_| "Invalid header value: missing int16"),
            4 => bytes
                .read_i32(endi::Endian::Big)
                .map(Value::Int32)
                .map_err(|_| "Invalid header value: missing int32"),
            5 => bytes
                .read_i64(endi::Endian::Big)
                .map(Value::Int64)
                .map_err(|_| "Invalid header value: missing int64"),
            6 => {
                let len = bytes
                    .read_u16(endi::Endian::Big)
                    .map_err(|_| "Invalid header value: missing byte buffer length")?
                    as usize;

                let value = bytes
                    .get(..len)
                    .ok_or("Invalid header value: missing byte buffer")
                    .map(Cow::Borrowed)
                    .map(Value::ByteBuffer)?;
                *bytes = &bytes[len..];
                Ok(value)
            }
            7 => {
                let len = bytes
                    .read_u16(endi::Endian::Big)
                    .map_err(|_| "Invalid header value: missing string length")?
                    as usize;

                let value = bytes
                    .get(..len)
                    .ok_or("Invalid header value: missing string")
                    .map(|slice| {
                        let string = std::str::from_utf8(slice)
                            .map_err(|_| "Invalid header value: invalid string")?;
                        Ok(Value::String(Cow::Borrowed(string)))
                    })?;
                *bytes = &bytes[len..];
                value
            }
            8 => bytes
                .read_i64(endi::Endian::Big)
                .map(Value::Timestamp)
                .map_err(|_| "Invalid header value: missing timestamp"),
            9 => {
                let array = bytes
                    .get(..16)
                    .ok_or("Invalid header value: missing UUID")
                    .map(|slice| slice.try_into().unwrap())?;
                let value = uuid::Uuid::from_slice(array)
                    .map(Value::Uuid)
                    .map_err(|_| "Invalid header value: invalid UUID")?;
                *bytes = &bytes[16..];
                Ok(value)
            }
            _ => Err("Invalid header value: unknown type"),
        }
    }
}
