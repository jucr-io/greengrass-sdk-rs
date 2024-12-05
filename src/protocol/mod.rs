use std::{
    env,
    io::{self, Write},
};
use crc::{Crc, CRC_32_ISO_HDLC};
use endi::{ReadBytes, WriteBytes};

use headers::{Headers, MessageFlags, MessageType};
use serde_json::{from_slice, json, to_vec, Map, Value};

pub mod headers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'m> {
    headers: Headers<'m>,
    payload: Option<Value>,
}

impl<'m> Message<'m> {
    pub fn new(headers: Headers<'m>, payload: Option<Value>) -> Self
    {
        Self { headers, payload }
    }

    pub fn connect_request() -> io::Result<Self> {
        let mut headers = Headers::new(0, MessageType::Connect, MessageFlags::None);
        headers.insert(":version", headers::Value::String("0.1.0".into()));
        headers.insert(":content-type", headers::Value::String("application/json".into()));
        let auth_token = env::var("SVCUID").map_err(|_| io::ErrorKind::NotFound)?;
        let payload = json!({ "authToken": auth_token });

        Ok(Self::new(headers, Some(payload)))
    }

    pub fn ipc_call(
        service_model_type: &'static str,
        operation: &'static str,
        stream_id: i32,
        payload: Option<Value>,
    ) -> Self
    {
        let mut headers = Headers::new(stream_id, MessageType::Application, MessageFlags::None);
        headers.insert(
            "service-model-type",
            headers::Value::String(service_model_type.into()),
        );
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
        let mut payload = Map::new();
        payload.insert("deploymentId".into(), Value::String(deployment_id.into()));

        if let Some(name) = component_name {
            payload.insert("message".into(), Value::String(name.into()));
        };
        if let Some(recheck_after_ms) = recheck_after_ms {
            payload.insert("recheckAfterMs".into(), Value::Number(recheck_after_ms.into()));
        }

        Self::ipc_call(
            "aws.greengrass#DeferComponentUpdateRequest",
            "aws.greengrass#DeferComponentUpdate",
            stream_id,
            Some(Value::Object(payload)),
        )
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(1024);

        // First the prelude.
        let headers_len = self.headers.size_in_bytes()?;
        let payload = self.payload.as_ref().map(|p| to_vec(p)).transpose()?.unwrap_or_default();
        let payload_len: u32 = payload.len().try_into().map_err(|_| io::ErrorKind::InvalidInput)?;
        let total_len =
            // 8 bytes prelude + 4 bytes CRC checksum of prelude.
            12 + 
            headers_len + 
            payload_len + 
            // 4 bytes CRC checksum of the whole message.
            4;

        bytes.write_u32(endi::Endian::Big, total_len)?;
        bytes.write_u32(endi::Endian::Big, headers_len)?;
        let crc32 = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let checksum = crc32.checksum(&bytes);
        bytes.write_u32(endi::Endian::Big, checksum)?;

        // Then the headers and payload.
        self.headers.write_as_bytes(&mut bytes)?;
        bytes.write(&payload)?;

        // Finally the CRC checksum of the whole message.
        let checksum = crc32.checksum(&bytes);
        bytes.write_u32(endi::Endian::Big, checksum)?;

        Ok(bytes)
    }

    pub fn from_bytes(bytes: &mut &'m [u8]) -> Result<Self, &'static str> {
        let crc32 = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let msg_checksum = crc32.checksum(&bytes[..bytes.len() - 4]);
        let prelude_checksum = crc32.checksum(&bytes[..8]);
        let total_len = bytes.read_u32(endi::Endian::Big).map_err(|_| "invalid encoding")? as usize;
        let headers_len = bytes.read_u32(endi::Endian::Big).map_err(|_| "invalid encoding")? as usize;
        if total_len > u32::MAX as usize || headers_len > total_len {
            return Err("Invalid length");
        }
        if prelude_checksum != bytes.read_u32(endi::Endian::Big).map_err(|_| "invalid encoding")? {
            return Err("Invalid prelude checksum");
        }

        let headers = Headers::from_bytes(&mut &bytes[..headers_len])?;
        // The `unwrap` call here can only panic if the headers length is > `u32::MAX` and `from_bytes`
        // ensures that it is not.
        if headers.size_in_bytes().unwrap() as usize != headers_len {
            return Err("Invalid headers length");
        }
        *bytes = &bytes[headers_len..];

        // 8 bytes prelude + 4 bytes CRC checksum of prelude + header bytes already parsed.
        let msg_crc_offset = total_len - 12 - headers_len - 4;
        let payload = from_slice(&bytes[..dbg!(msg_crc_offset)])
            .map_err(|_| "Invalid payload")?;
        *bytes = &bytes[msg_crc_offset..];
        if msg_checksum != bytes.read_u32(endi::Endian::Big).map_err(|_| "invalid encoding")? {
            return Err("Invalid message checksum");
        }

        Ok(Self::new(headers, payload))
    }

    pub fn headers(&self) -> &Headers<'m> {
        &self.headers
    }

    pub fn payload(&self) -> &Option<Value> {
        &self.payload
    }
}

#[cfg(test)]
mod tests {
    use pretty_hex::PrettyHex;

    use super::*;

    #[test]
    fn message_to_n_from_bytes() {
        let headers = Headers::new(0, MessageType::Connect, MessageFlags::None);
        let payload = Some(json!({"hello": "world"}));
        let message = Message::new(headers, payload.clone());
        let bytes = message.to_bytes().unwrap();
        // Printing the bytes make debugging easier if the parsing fails because of a regression.
        println!("{:?}", bytes.hex_dump());

        let message = Message::from_bytes(&mut &bytes[..]).unwrap();
        assert_eq!(message.payload(), &payload);
    }
}