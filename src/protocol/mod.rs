use crc::{Crc, CRC_32_ISO_HDLC};
use endi::{ReadBytes, WriteBytes};
use serde::Serialize;
use std::io::Write;
use tracing::trace;

use headers::{Headers, MessageFlags, MessageType};
use prelude::Prelude;
use serde_json::{from_slice, json, to_value, to_vec, Value};

pub mod headers;
pub mod prelude;

use crate::{env, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'m> {
    headers: Headers<'m>,
    payload: Option<Value>,
}

impl<'m> Message<'m> {
    pub fn new(headers: Headers<'m>, payload: Option<Value>) -> Self {
        Self { headers, payload }
    }

    pub fn connect_request() -> Result<Self> {
        let mut headers = Headers::new(0, MessageType::Connect, MessageFlags::none());
        headers.insert(":version", headers::Value::String("0.1.0".into()));
        headers.insert(":content-type", headers::Value::String("application/json".into()));
        let auth_token = env::auth_token()?;
        let payload = json!({ "authToken": auth_token });

        Ok(Self::new(headers, Some(payload)))
    }

    pub fn ipc_call(
        service_model_type: &'static str,
        operation: &'static str,
        stream_id: i32,
        payload: Option<Value>,
    ) -> Self {
        let mut headers = Headers::new(stream_id, MessageType::Application, MessageFlags::none());
        headers.insert("service-model-type", headers::Value::String(service_model_type.into()));
        headers.insert("operation", headers::Value::String(operation.into()));

        Self::new(headers, payload)
    }

    pub fn component_updates_subcription_request(stream_id: i32) -> Self {
        Self::ipc_call(
            "aws.greengrass#SubscribeToComponentUpdatesRequest",
            "aws.greengrass#SubscribeToComponentUpdates",
            stream_id,
            None,
        )
    }

    pub fn defer_component_update(
        stream_id: i32,
        deployment_id: &str,
        component_name: Option<&str>,
        recheck_after_ms: Option<u64>,
    ) -> Self {
        #[derive(Serialize)]
        struct DeferComponentUpdateRequest<'a> {
            #[serde(rename = "deploymentId")]
            deployment_id: &'a str,
            #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
            message: Option<&'a str>,
            #[serde(rename = "recheckAfterMs", skip_serializing_if = "Option::is_none")]
            recheck_after_ms: Option<u64>,
        }

        let payload = to_value(DeferComponentUpdateRequest {
            deployment_id,
            message: component_name,
            recheck_after_ms,
        })
        .unwrap();

        Self::ipc_call(
            "aws.greengrass#DeferComponentUpdateRequest",
            "aws.greengrass#DeferComponentUpdate",
            stream_id,
            Some(payload),
        )
    }

    pub fn update_state(stream_id: i32, state: crate::LifecycleState) -> Self {
        #[derive(Serialize)]
        struct UpdateStateRequest {
            #[serde(rename = "state")]
            state: crate::LifecycleState,
        }
        let payload = to_value(UpdateStateRequest { state }).unwrap();

        Self::ipc_call(
            "aws.greengrass#UpdateStateRequest",
            "aws.greengrass#UpdateState",
            stream_id,
            Some(payload),
        )
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(1024);

        // First the prelude.
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

    pub fn headers(&self) -> &Headers<'m> {
        &self.headers
    }

    pub fn payload(&self) -> &Option<Value> {
        &self.payload
    }

    pub fn to_owned(&self) -> Message<'static> {
        Message { headers: self.headers.to_owned(), payload: self.payload.clone() }
    }
}

#[cfg(test)]
mod tests {
    use pretty_hex::PrettyHex;

    use super::*;

    #[test]
    fn message_to_n_from_bytes() {
        let headers = Headers::new(0, MessageType::Connect, MessageFlags::none());
        let payload = Some(json!({"hello": "world"}));
        let message = Message::new(headers, payload.clone());
        let bytes = message.to_bytes().unwrap();
        // Printing the bytes make debugging easier if the parsing fails because of a regression.
        println!("{:?}", bytes.hex_dump());

        let message = Message::from_bytes(&mut &bytes[..]).unwrap();
        assert_eq!(message.payload(), &payload);
    }
}
