use std::{env, io};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::protocol::{
    headers::{MessageFlags, MessageType, Value},
    prelude::{Prelude, PRELUDE_SIZE},
    Message,
};

#[derive(Debug)]
pub struct Connection {
    socket: UnixStream,
    next_stream_id: i32,
    buffer: Vec<u8>,
}

impl Connection {
    pub async fn new() -> Result<Self, io::Error> {
        let socket_path = env::var("AWS_GG_NUCLEUS_DOMAIN_SOCKET_FILEPATH_FOR_COMPONENT")
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Env not set"))?;
        let stream = UnixStream::connect(&socket_path).await?;

        let mut conn = Self { socket: stream, next_stream_id: 1, buffer: Vec::with_capacity(1024) };

        // Handshake
        let message = Message::connect_request()?;
        conn.send_message(message).await?;
        let response = conn.read_message().await?;
        let headers = response.headers();
        if headers.get(":stream-id") != Some(&Value::Int32(0))
            || headers.get(":message-type") != Some(&Value::Int32(MessageType::ConnectAck.into()))
            || headers.get(":message-flags")
                != Some(&Value::Int32(MessageFlags::ConnectionAccepted as i32))
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid connection response"));
        }

        Ok(conn)
    }

    pub async fn subscribe_to_component_updates(&mut self) -> Result<i32, io::Error> {
        let id = self.next_stream_id();
        let message = Message::component_updates_subcription_request(id);
        self.send_message(message).await?;
        let _ = self.read_response(id).await?;

        Ok(id)
    }

    pub async fn defer_component_update(
        &mut self,
        deployment_id: &str,
        component_name: Option<&str>,
        recheck_after_ms: Option<u64>,
    ) -> Result<i32, io::Error> {
        let id = self.next_stream_id();
        let message =
            Message::defer_component_update(id, deployment_id, component_name, recheck_after_ms);
        self.send_message(message).await?;
        let _ = self.read_response(id).await?;

        Ok(id)
    }

    pub async fn send_message(&mut self, message: Message<'_>) -> Result<(), io::Error> {
        let buf = message.to_bytes()?;

        self.socket.write_all(&buf).await
    }

    /// Reads messages until it receives a message with the specified stream ID.
    pub async fn read_response(&mut self, stream_id: i32) -> Result<Message, io::Error> {
        loop {
            let message = self.read_message().await?;
            let headers = message.headers();
            if headers.get(":stream-id") == Some(&Value::Int32(stream_id)) {
                return Ok(message.to_owned());
            }

            if headers.get(":message-type") != Some(&Value::Int32(MessageType::Application.into()))
            {
                return Err(io::Error::new(io::ErrorKind::Other, "Received error response"));
            }
        }
    }

    pub async fn read_message(&mut self) -> Result<Message<'_>, io::Error> {
        self.socket.read_exact(&mut self.buffer[0..PRELUDE_SIZE]).await?;
        let prelude = Prelude::from_bytes(&mut &self.buffer[0..PRELUDE_SIZE])?;
        if prelude.total_len() > self.buffer.len() {
            self.buffer.resize(prelude.total_len(), 0);
        }

        self.socket.read_exact(&mut self.buffer[PRELUDE_SIZE..prelude.total_len()]).await?;

        Message::from_bytes(&mut &self.buffer[0..prelude.total_len()])
    }

    fn next_stream_id(&mut self) -> i32 {
        let stream_id = self.next_stream_id;
        self.next_stream_id += 1;

        stream_id
    }
}
