use super::{
    super::headers::{self, Headers, MessageFlags, MessageType},
    Message,
};
use crate::{env, Result};
use serde::{Deserialize, Serialize};

impl ConnectRequest {
    pub fn new() -> Result<Message<'static, Self>> {
        let mut headers = Headers::new(0, MessageType::Connect, MessageFlags::none());
        headers.insert(":version", headers::Value::String("0.1.0".into()));
        headers.insert(":content-type", headers::Value::String("application/json".into()));
        // TODO: Cache the env in a static variable and then `ConnectRequest` can use `&'static str`.
        let auth_token = env::auth_token()?;

        Ok(Message::new(headers, Some(ConnectRequest { auth_token })))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectRequest {
    #[serde(rename = "authToken")]
    auth_token: String,
}
