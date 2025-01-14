use core::fmt::Debug;

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use tracing::{trace, warn};
use uuid::Uuid;

use crate::{
    env,
    protocol::{
        prelude::{Prelude, SIZE},
        ComponentUpdateSubscriptionRequest, ComponentUpdateSubscriptionResponse, ConnectRequest,
        ConnectResponse, DeferComponentUpdateRequest, DeferComponentUpdateResponse, Message,
        MessageFlags, MessageType, RecheckAfterMs, UpdateStateRequest, UpdateStateResponse,
    },
    Error, Result,
};

/// A connection to the server.
///
/// This is the low-level API to directly interact with the server. Typically you would use the
/// [`crate::IpcClient`] instead.
#[derive(Debug)]
pub struct Connection {
    socket: UnixStream,
    next_stream_id: i32,
    buffer: Vec<u8>,
    socket_path: &'static str,
    auth_token: &'static str,
}

impl Connection {
    /// Creates a new connection to the server.
    pub async fn new(socket_path: &'static str, auth_token: &'static str) -> Result<Self> {
        let stream = UnixStream::connect(&socket_path).await?;

        let mut conn = Self {
            socket: stream,
            next_stream_id: 1,
            buffer: Vec::from([0; 1024]),
            socket_path,
            auth_token,
        };

        // Handshake
        let message = ConnectRequest::new(auth_token)?;
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

    /// Creates a new connection to the server, fetching the socket path and auth token from the
    /// environment.
    pub async fn from_env() -> Result<Self> {
        let socket_path = env::socket_path()?;
        let auth_token = env::auth_token()?;

        Self::new(socket_path, auth_token).await
    }

    /// Subscribes to component updates.
    pub async fn subscribe_to_component_updates(&mut self) -> Result<i32> {
        let id = self.next_stream_id();
        let message = ComponentUpdateSubscriptionRequest::new(id);
        let _ = self.call::<_, ComponentUpdateSubscriptionResponse>(message, false).await?;

        Ok(id)
    }

    /// Defers a component update.
    pub async fn defer_component_update(
        &mut self,
        deployment_id: Uuid,
        component_name: Option<&str>,
        recheck_after_ms: RecheckAfterMs,
    ) -> Result<()> {
        let id = self.next_stream_id();
        let message =
            DeferComponentUpdateRequest::new(id, deployment_id, component_name, recheck_after_ms);
        let _ = self.call::<_, DeferComponentUpdateResponse>(message, true).await?;

        Ok(())
    }

    /// Updates the state of the lifecycle.
    pub async fn update_state(&mut self, state: crate::LifecycleState) -> Result<()> {
        let id = self.next_stream_id();
        let message = UpdateStateRequest::new(id, state);
        let _ = self.call::<_, UpdateStateResponse>(message, true).await?;

        Ok(())
    }

    /// Calls a method on a service.
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

    /// Sends a message.
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
        loop {
            // Rationale for `unsafe` use: The current borrow-checker (as of 2024-12-20) is unable
            // to infer that we don't keep the borrow from the mutable reference around after each
            // loop and hence it's safe to borrow from `self` multiple times and gives us an error
            // if we can use `self` directly here. We can do that once Polonius is stable:
            //
            // https://github.com/rust-lang/rust/issues/134554
            //
            // Safety: We're just converting to a pointer (safe) & back again (unsafe).
            let conn = unsafe { &mut *(self as *mut Self) };
            let message = conn.read_message::<Payload>().await?;
            let headers = message.headers();
            let received_stream_id = headers.stream_id();
            if received_stream_id != stream_id {
                trace!(
                    "Received message with stream ID {received_stream_id}, expected {stream_id}",
                );

                continue;
            }

            trace!("Received response with stream ID {stream_id}");
            let message_type = headers.message_type();
            let expected_type =
                if stream_id == 0 { MessageType::ConnectAck } else { MessageType::Application };
            if message_type != expected_type {
                // We already established above that the message belong to the stream ID we're
                // interested in so the message type must match here.
                return Err(Error::UnexpectedMessageType {
                    expected: expected_type,
                    received: message_type,
                });
            }
            let stream_terminated = headers.message_flags().contains(MessageFlags::TerminateStream);
            if stream_id != 0 {
                // Should we return errors here? 🤔
                if last_response && !stream_terminated {
                    warn!("Response unexpectedly not marked as end of stream");
                } else if !last_response && stream_terminated {
                    warn!("Unexpected end of stream");
                }
            }

            break Ok(message);
        }
    }

    /// Reads a message.
    pub async fn read_message<'c, Payload>(&'c mut self) -> Result<Message<'c, Payload>>
    where
        Payload: Deserialize<'c> + Debug,
    {
        self.socket.read_exact(&mut self.buffer[0..SIZE]).await?;
        let prelude = Prelude::from_bytes(&mut &self.buffer[0..SIZE])?;
        if prelude.total_len() > self.buffer.len() {
            self.buffer.resize(prelude.total_len(), 0);
        }

        self.socket.read_exact(&mut self.buffer[SIZE..prelude.total_len()]).await?;

        Message::from_bytes(&mut &self.buffer[0..prelude.total_len()])
    }

    /// The path of the socket.
    pub fn socket_path(&self) -> &'static str {
        self.socket_path
    }

    /// The authentication token.
    pub fn auth_token(&self) -> &'static str {
        self.auth_token
    }

    fn next_stream_id(&mut self) -> i32 {
        let stream_id = self.next_stream_id;
        self.next_stream_id += 1;

        stream_id
    }
}
