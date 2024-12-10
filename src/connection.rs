use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::{
    env,
    protocol::{
        headers::{MessageFlags, MessageType},
        prelude::{Prelude, PRELUDE_SIZE},
        Message,
    },
    Error, Result,
};

#[derive(Debug)]
pub struct Connection {
    socket: UnixStream,
    next_stream_id: i32,
    buffer: Vec<u8>,
}

impl Connection {
    pub async fn new() -> Result<Self> {
        let socket_path = env::socket_path()?;
        let stream = UnixStream::connect(&socket_path).await?;

        let mut conn = Self { socket: stream, next_stream_id: 1, buffer: Vec::with_capacity(1024) };

        // Handshake
        let message = Message::connect_request()?;
        conn.send_message(message).await?;
        let response = conn.read_message().await?;
        let headers = response.headers();
        if headers.stream_id() != 0
            || headers.message_type() != MessageType::ConnectAck
            || headers.message_flags() != MessageFlags::ConnectionAccepted
        {
            return Err(Error::Protocol("Invalid connection response".into()));
        }

        Ok(conn)
    }

    pub async fn subscribe_to_component_updates(&mut self) -> Result<i32> {
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
    ) -> Result<()> {
        let id = self.next_stream_id();
        let message =
            Message::defer_component_update(id, deployment_id, component_name, recheck_after_ms);
        self.send_message(message).await?;
        let _ = self.read_response(id).await?;

        Ok(())
    }

    pub async fn update_state(&mut self, state: crate::LifecycleState) -> Result<()> {
        let id = self.next_stream_id();
        let message = Message::update_state(id, state);
        self.send_message(message).await?;
        let _ = self.read_response(id).await?;

        Ok(())
    }

    pub async fn send_message(&mut self, message: Message<'_>) -> Result<()> {
        let buf = message.to_bytes()?;

        self.socket.write_all(&buf).await.map_err(Error::Io)
    }

    /// Reads messages until it receives a message with the specified stream ID.
    pub async fn read_response(&mut self, stream_id: i32) -> Result<Message> {
        loop {
            let message = self.read_message().await?;
            let headers = message.headers();
            if headers.stream_id() == stream_id {
                return Ok(message.to_owned());
            }

            let message_type = headers.message_type();
            match message_type {
                MessageType::Application => (),
                MessageType::ApplicationError => {
                    return Err(Error::Application(
                        message.payload().as_ref().map(ToString::to_string).unwrap_or_default(),
                    ))
                }
                // We already established above that the message belong to the stream ID we're
                // interested in so the message type must match here.
                _ => {
                    return Err(Error::UnexpectedMessageType {
                        expected: MessageType::Application,
                        received: message_type,
                    })
                }
            }
        }
    }

    pub async fn read_message(&mut self) -> Result<Message<'_>> {
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
