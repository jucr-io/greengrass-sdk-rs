use super::{
    super::headers::{self, Headers, MessageFlags, MessageType},
    Message,
};
use crate::Result;
use serde::{Deserialize, Serialize};

impl<'c> ConnectRequest<'c> {
    /// Creates a new `ConnectRequest`.
    pub fn new(auth_token: &'c str) -> Result<Message<'static, Self>> {
        let mut headers = Headers::new(0, MessageType::Connect, MessageFlags::none());
        headers.insert(":version", headers::Value::String("0.1.0".into()));
        headers.insert(":content-type", headers::Value::String("application/json".into()));

        Ok(Message::new(headers, Some(ConnectRequest { auth_token })))
    }
}

/// A request to connect to the server.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ConnectRequest<'c> {
    #[serde(rename = "authToken", borrow)]
    auth_token: &'c str,
}

impl ConnectRequest<'_> {
    /// The authentication token.
    pub fn auth_token(&self) -> &str {
        self.auth_token
    }
}

/// A response to a connection request.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ConnectResponse {}
