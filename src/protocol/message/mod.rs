use crc::{Crc, CRC_32_ISO_HDLC};
use endi::{ReadBytes, WriteBytes};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, io::Write};
use tracing::trace;

use serde_json::{from_slice, to_vec};

pub mod component_update;
pub mod handshake;
pub mod state;

use crate::{Error, Result};

use super::{
    headers::{self, Headers, MessageFlags, MessageType},
    prelude::Prelude,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'m, Payload> {
    headers: Headers<'m>,
    payload: Option<Payload>,
}

impl<'m, Payload> Message<'m, Payload> {
    pub fn new(headers: Headers<'m>, payload: Option<Payload>) -> Self {
        Self { headers, payload }
    }

    pub fn ipc_call(
        service_model_type: &'static str,
        operation: &'static str,
        stream_id: i32,
        payload: Option<Payload>,
    ) -> Self {
        let mut headers = Headers::new(stream_id, MessageType::Application, MessageFlags::none());
        headers.insert("service-model-type", headers::Value::String(service_model_type.into()));
        headers.insert("operation", headers::Value::String(operation.into()));

        Self::new(headers, payload)
    }

    pub fn headers(&self) -> &Headers<'m> {
        &self.headers
    }

    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }
}

impl<'m, Payload> Message<'m, Payload>
where
    Payload: Serialize + Debug,
{
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(1024);

        let headers_len = self.headers.size_in_bytes()?;
        let payload = self.payload.as_ref().map(|p| to_vec(p)).transpose()?.unwrap_or_default();
        let payload_len: u32 = payload.len().try_into().map_err(|_| Error::BufferTooLarge {
            size: payload.len(),
            max_size: u32::MAX as usize,
        })?;
        let total_len =
            // 8 bytes prelude + 4 bytes CRC checksum of prelude.
            12 +
            headers_len +
            payload_len +
            // 4 bytes CRC checksum of the whole message.
            4;

        let prelude = Prelude::new(total_len as usize, headers_len as usize)?;
        prelude.write_as_bytes(&mut bytes)?;

        // Then the headers and payload.
        self.headers.write_as_bytes(&mut bytes)?;
        bytes.write(&payload)?;

        // Finally the CRC checksum of the whole message.
        let crc32 = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let checksum = crc32.checksum(&bytes);
        bytes.write_u32(endi::Endian::Big, checksum)?;

        Ok(bytes)
    }
}

impl<'m, Payload> Message<'m, Payload>
where
    Payload: Deserialize<'m> + Debug,
{
    pub fn from_bytes(bytes: &mut &'m [u8]) -> Result<Self> {
        trace!("Parsing message from bytes: {:02X?}", bytes);
        let crc32 = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let msg_checksum = crc32.checksum(&bytes[..bytes.len() - 4]);
        let prelude =
            Prelude::from_bytes(bytes).map_err(|_| Error::Protocol("Invalid prelude".into()))?;

        trace!("Prelede: {:?}", prelude);
        let headers = Headers::from_bytes(&mut &bytes[..prelude.headers_len()])?;
        // The `unwrap` call here can only panic if the headers length is > `u32::MAX` and
        // `from_bytes` ensures that it is not.
        if headers.size_in_bytes().unwrap() as usize != prelude.headers_len() {
            return Err(Error::Protocol("Incorrect header length".into()));
        }

        trace!("Headers: {:?}", headers);
        *bytes = &bytes[prelude.headers_len()..];

        // 8 bytes prelude + 4 bytes CRC checksum of prelude + header bytes already parsed.
        let msg_crc_offset = prelude.total_len() - 12 - prelude.headers_len() - 4;
        if matches!(headers.message_type(), MessageType::ApplicationError) {
            let err_msg = if msg_crc_offset != 0 {
                from_slice(&bytes[..dbg!(msg_crc_offset)])
                    .map_err(|e| Error::Protocol(format!("Invalid payload: {e}")))?
            } else {
                String::new()
            };

            return Err(Error::Application(err_msg));
        }
        let payload = if msg_crc_offset != 0 {
            from_slice(&bytes[..dbg!(msg_crc_offset)])
                .map_err(|e| Error::Protocol(format!("Invalid payload: {e}")))?
        } else {
            None
        };
        trace!("Payload: {:?}", payload);
        *bytes = &bytes[msg_crc_offset..];
        if msg_checksum
            != bytes
                .read_u32(endi::Endian::Big)
                .map_err(|_| Error::Protocol("Invalid encoding".into()))?
        {
            return Err(Error::ChecksumMismatch);
        }
        trace!("Succesfully parsed message");

        Ok(Self::new(headers, payload))
    }
}

impl<'m, Payload> Message<'m, Payload>
where
    Payload: ToOwned,
{
    pub fn to_owned(&self) -> Message<'static, Payload::Owned> {
        Message {
            headers: self.headers.to_owned(),
            payload: self.payload.as_ref().map(ToOwned::to_owned),
        }
    }
}
