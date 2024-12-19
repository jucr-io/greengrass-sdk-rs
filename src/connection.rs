use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use tracing::{trace, warn};

use crate::{
    env,
    protocol::{
        headers::{MessageFlags, MessageType},
        message::{
            component_update::{
                ComponentUpdateSubscriptionRequest, ComponentUpdateSubscriptionResponse,
                DeferComponentUpdateRequest, DeferComponentUpdateResponse, RecheckAfterMs,
            },
            handshake::{ConnectRequest, ConnectResponse},
            state::{UpdateStateRequest, UpdateStateResponse},
            Message,
        },
        prelude::{Prelude, PRELUDE_SIZE},
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

        let mut conn = Self { socket: stream, next_stream_id: 1, buffer: Vec::from([0; 1024]) };

        // Handshake
        let message = ConnectRequest::new()?;
        let response = conn.call::<_, ConnectResponse>(message, true).await?;
        let headers = response.headers();
        if headers.message_type() != MessageType::ConnectAck {
            return Err(Error::Protocol("Invalid connection response".into()));
        }
        if !headers.message_flags().contains(MessageFlags::ConnectionAccepted) {
            return Err(Error::ConnectionRefused);
        }

        Ok(conn)
    }

    pub async fn subscribe_to_component_updates(&mut self) -> Result<i32> {
        let id = self.next_stream_id();
        let message = ComponentUpdateSubscriptionRequest::new(id);
        let _ = self.call::<_, ComponentUpdateSubscriptionResponse>(message, false).await?;

        Ok(id)
    }

    pub async fn defer_component_update(
        &mut self,
        deployment_id: &str,
        component_name: Option<&str>,
        recheck_after_ms: RecheckAfterMs,
    ) -> Result<()> {
        let id = self.next_stream_id();
        let message =
            DeferComponentUpdateRequest::new(id, deployment_id, component_name, recheck_after_ms);
        let _ = self.call::<_, DeferComponentUpdateResponse>(message, true).await?;

        Ok(())
    }

    pub async fn update_state(&mut self, state: crate::LifecycleState) -> Result<()> {
        let id = self.next_stream_id();
        let message = UpdateStateRequest::new(id, state);
        let _ = self.call::<_, UpdateStateResponse>(message, true).await?;

        Ok(())
    }

    pub async fn call<'c, RequestPayload, ResponsePayload>(
        &'c mut self,
        request: Message<'_, RequestPayload>,
        last_response: bool,
    ) -> Result<Message<'c, ResponsePayload>>
    where
        RequestPayload: Serialize + Debug,
        ResponsePayload: Deserialize<'c> + Debug,
    {
        let stream_id = request.headers().stream_id();
        self.send_message(request).await?;

        self.read_response(stream_id, last_response).await
    }

    pub async fn send_message<Payload>(&mut self, message: Message<'_, Payload>) -> Result<()>
    where
        Payload: Serialize + Debug,
    {
        let buf = message.to_bytes()?;

        self.socket.write_all(&buf).await.map_err(Error::Io)?;
        trace!("Sent message: {message:?}");

        Ok(())
    }

    /// Reads messages until it receives a message with the specified stream ID.
    pub async fn read_response<'c, Payload>(
        &'c mut self,
        stream_id: i32,
        last_response: bool,
    ) -> Result<Message<'c, Payload>>
    where
        Payload: Deserialize<'c> + Debug,
    {
        trace!("Waiting for response with stream ID {stream_id}");
        let message = self.read_message::<Payload>().await?;
        let headers = message.headers();
        let received_stream_id = headers.stream_id();
        if received_stream_id != stream_id {
            // Since we use separate connections for each subscriptions, we shouldn't receive
            // messsages with unexpected stream IDs. If for any reason, this turns out not to be the
            // case, we probably should make the stream ID and this check optional.
            return Err(Error::UnexpectedStreamId {
                expected: stream_id,
                received: received_stream_id,
            });
        }
        trace!("Received response with stream ID {stream_id}");

        let message_type = headers.message_type();
        match message_type {
            MessageType::Application => (),
            // We already established above that the message belong to the stream ID we're
            // interested in so the message type must match here.
            _ => {
                return Err(Error::UnexpectedMessageType {
                    expected: MessageType::Application,
                    received: message_type,
                })
            }
        }
        let stream_terminated = headers.message_flags().contains(MessageFlags::TerminateStream);
        // Should we return errors here? 🤔
        if last_response && !stream_terminated {
            warn!("Response unexpectedly not marked as end of stream");
        } else if !last_response && stream_terminated {
            warn!("Unexpected end of stream");
        }

        return Ok(message);
    }

    pub async fn read_message<'c, Payload>(&'c mut self) -> Result<Message<'c, Payload>>
    where
        Payload: Deserialize<'c> + Debug,
    {
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
