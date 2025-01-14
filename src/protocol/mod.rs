//! The protocol module contains the definitions of the protocol messages and the (de)serialization
//! logic for them.

pub mod headers;
pub mod message;
pub mod prelude;

#[cfg(test)]
mod tests {
    use pretty_hex::PrettyHex;
    use serde_json::json;

    use super::{
        headers::{Headers, MessageFlags, MessageType},
        message::{
            component_update::ComponentUpdateSubscriptionResponse, handshake::ConnectResponse,
            Message,
        },
    };

    #[test]
    fn message_to_n_from_bytes() {
        let headers = Headers::new(0, MessageType::Connect, MessageFlags::none());
        let payload = Some(json!({"hello": "world"}));
        let message = Message::new(headers, payload.clone());
        let bytes = message.to_bytes().unwrap();
        // Printing the bytes make debugging easier if the parsing fails because of a regression.
        println!("{:?}", bytes.hex_dump());

        let message = Message::from_bytes(&mut &bytes[..]).unwrap();
        assert_eq!(message.payload(), payload.as_ref());
    }

    #[test]
    fn parse_connect_refusal_without_panicking() {
        let bytes = [
            0x00, 0x00, 0x00, 0x47, 0x00, 0x00, 0x00, 0x37, 0x37, 0x4C, 0xFE, 0x3F, 0x0D, 0x3A,
            0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2D, 0x74, 0x79, 0x70, 0x65, 0x04, 0x00,
            0x00, 0x00, 0x05, 0x0E, 0x3A, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2D, 0x66,
            0x6C, 0x61, 0x67, 0x73, 0x04, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x3A, 0x73, 0x74, 0x72,
            0x65, 0x61, 0x6D, 0x2D, 0x69, 0x64, 0x04, 0x00, 0x00, 0x00, 0x00, 0x65, 0x91, 0x29,
            0x3E,
        ];
        let message = Message::<ConnectResponse>::from_bytes(&mut &bytes[..]).unwrap();
        assert_eq!(message.headers().message_type(), MessageType::ConnectAck);
        assert!(message.headers().message_flags().is_empty());
        assert!(message.payload().is_none());
    }

    #[test]
    fn parse_component_updates_response() {
        // A pre-component update event.
        let bytes = [
            0x00, 0x00, 0x01, 0x0A, 0x00, 0x00, 0x00, 0x98, 0x1A, 0x1A, 0x0F, 0x1B, 0x0D, 0x3A,
            0x63, 0x6F, 0x6E, 0x74, 0x65, 0x6E, 0x74, 0x2D, 0x74, 0x79, 0x70, 0x65, 0x07, 0x00,
            0x10, 0x61, 0x70, 0x70, 0x6C, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6F, 0x6E, 0x2F, 0x6A,
            0x73, 0x6F, 0x6E, 0x12, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x2D, 0x6D, 0x6F,
            0x64, 0x65, 0x6C, 0x2D, 0x74, 0x79, 0x70, 0x65, 0x07, 0x00, 0x2A, 0x61, 0x77, 0x73,
            0x2E, 0x67, 0x72, 0x65, 0x65, 0x6E, 0x67, 0x72, 0x61, 0x73, 0x73, 0x23, 0x43, 0x6F,
            0x6D, 0x70, 0x6F, 0x6E, 0x65, 0x6E, 0x74, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x50,
            0x6F, 0x6C, 0x69, 0x63, 0x79, 0x45, 0x76, 0x65, 0x6E, 0x74, 0x73, 0x0D, 0x3A, 0x6D,
            0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2D, 0x74, 0x79, 0x70, 0x65, 0x04, 0x00, 0x00,
            0x00, 0x00, 0x0E, 0x3A, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2D, 0x66, 0x6C,
            0x61, 0x67, 0x73, 0x04, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x3A, 0x73, 0x74, 0x72, 0x65,
            0x61, 0x6D, 0x2D, 0x69, 0x64, 0x04, 0x00, 0x00, 0x00, 0x01, 0x7B, 0x22, 0x70, 0x72,
            0x65, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x45, 0x76, 0x65, 0x6E, 0x74, 0x22, 0x3A,
            0x7B, 0x22, 0x64, 0x65, 0x70, 0x6C, 0x6F, 0x79, 0x6D, 0x65, 0x6E, 0x74, 0x49, 0x64,
            0x22, 0x3A, 0x22, 0x37, 0x37, 0x64, 0x30, 0x30, 0x63, 0x36, 0x62, 0x2D, 0x66, 0x30,
            0x63, 0x36, 0x2D, 0x34, 0x65, 0x31, 0x34, 0x2D, 0x38, 0x36, 0x63, 0x62, 0x2D, 0x64,
            0x34, 0x37, 0x36, 0x66, 0x30, 0x30, 0x31, 0x36, 0x30, 0x34, 0x34, 0x22, 0x2C, 0x22,
            0x69, 0x73, 0x47, 0x67, 0x63, 0x52, 0x65, 0x73, 0x74, 0x61, 0x72, 0x74, 0x69, 0x6E,
            0x67, 0x22, 0x3A, 0x66, 0x61, 0x6C, 0x73, 0x65, 0x7D, 0x7D, 0xA7, 0x68, 0x41, 0xFE,
        ];
        let message =
            Message::<ComponentUpdateSubscriptionResponse>::from_bytes(&mut &bytes[..]).unwrap();
        assert_eq!(message.headers().message_type(), MessageType::Application);
        assert!(message.headers().message_flags().is_empty());
        let payload = message.payload().unwrap();
        let pre_update_event = payload.pre_update_event().unwrap();
        assert_eq!(
            pre_update_event.deployment_id(),
            "77d00c6b-f0c6-4e14-86cb-d476f0016044".try_into().unwrap()
        );
        assert_eq!(pre_update_event.is_ggc_restarting(), false);
        assert!(payload.post_update_event().is_none());

        // A post-component update event.
        let bytes = [
            0x00, 0x00, 0x00, 0xF3, 0x00, 0x00, 0x00, 0x98, 0x94, 0x62, 0xEE, 0x92, 0x0D, 0x3A,
            0x63, 0x6F, 0x6E, 0x74, 0x65, 0x6E, 0x74, 0x2D, 0x74, 0x79, 0x70, 0x65, 0x07, 0x00,
            0x10, 0x61, 0x70, 0x70, 0x6C, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6F, 0x6E, 0x2F, 0x6A,
            0x73, 0x6F, 0x6E, 0x12, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x2D, 0x6D, 0x6F,
            0x64, 0x65, 0x6C, 0x2D, 0x74, 0x79, 0x70, 0x65, 0x07, 0x00, 0x2A, 0x61, 0x77, 0x73,
            0x2E, 0x67, 0x72, 0x65, 0x65, 0x6E, 0x67, 0x72, 0x61, 0x73, 0x73, 0x23, 0x43, 0x6F,
            0x6D, 0x70, 0x6F, 0x6E, 0x65, 0x6E, 0x74, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x50,
            0x6F, 0x6C, 0x69, 0x63, 0x79, 0x45, 0x76, 0x65, 0x6E, 0x74, 0x73, 0x0D, 0x3A, 0x6D,
            0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2D, 0x74, 0x79, 0x70, 0x65, 0x04, 0x00, 0x00,
            0x00, 0x00, 0x0E, 0x3A, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x2D, 0x66, 0x6C,
            0x61, 0x67, 0x73, 0x04, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x3A, 0x73, 0x74, 0x72, 0x65,
            0x61, 0x6D, 0x2D, 0x69, 0x64, 0x04, 0x00, 0x00, 0x00, 0x01, 0x7B, 0x22, 0x70, 0x6F,
            0x73, 0x74, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x45, 0x76, 0x65, 0x6E, 0x74, 0x22,
            0x3A, 0x7B, 0x22, 0x64, 0x65, 0x70, 0x6C, 0x6F, 0x79, 0x6D, 0x65, 0x6E, 0x74, 0x49,
            0x64, 0x22, 0x3A, 0x22, 0x38, 0x66, 0x35, 0x35, 0x37, 0x37, 0x35, 0x65, 0x2D, 0x38,
            0x64, 0x62, 0x31, 0x2D, 0x34, 0x62, 0x38, 0x32, 0x2D, 0x61, 0x35, 0x34, 0x38, 0x2D,
            0x31, 0x39, 0x31, 0x36, 0x66, 0x37, 0x62, 0x33, 0x36, 0x31, 0x66, 0x62, 0x22, 0x7D,
            0x7D, 0x43, 0x2C, 0x0A, 0x1C,
        ];
        let message =
            Message::<ComponentUpdateSubscriptionResponse>::from_bytes(&mut &bytes[..]).unwrap();
        assert_eq!(message.headers().message_type(), MessageType::Application);
        assert!(message.headers().message_flags().is_empty());
        let payload = message.payload().unwrap();
        let post_update_event = payload.post_update_event().unwrap();
        assert_eq!(
            post_update_event.deployment_id(),
            "8f55775e-8db1-4b82-a548-1916f7b361fb".try_into().unwrap()
        );
        assert!(payload.pre_update_event().is_none());
    }
}
