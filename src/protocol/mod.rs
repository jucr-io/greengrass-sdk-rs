pub mod headers;
pub mod message;
pub mod prelude;

#[cfg(test)]
mod tests {
    use pretty_hex::PrettyHex;
    use serde_json::json;

    use super::{
        headers::{Headers, MessageFlags, MessageType},
        message::Message,
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
        assert_eq!(message.payload(), &payload);
    }
}
