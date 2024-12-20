use std::{
    env::{self, temp_dir},
    future::pending,
};

use enumflags2::BitFlags;
use greengrass_sdk_rs::{
    env::{AUTH_TOKEN_ENV, SOCKET_PATH_ENV},
    protocol::{
        headers::{Headers, MessageFlags, MessageType},
        message::{
            component_update::{
                ComponentUpdateSubscriptionRequest, ComponentUpdateSubscriptionResponse,
            },
            handshake::{ConnectRequest, ConnectResponse},
            state::{UpdateStateRequest, UpdateStateResponse},
            Message,
        },
    },
    IpcClient, LifecycleState,
};
use test_log::test;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    spawn,
    task::JoinHandle,
};

fn mock_greengrass_server() -> JoinHandle<()> {
    let filename = format!("greengrass-sdk-rs-{}", fastrand::f64());
    let path = temp_dir().join(filename);
    env::set_var(SOCKET_PATH_ENV, path.as_os_str());
    env::set_var(AUTH_TOKEN_ENV, "whatever");

    let listener = UnixListener::bind(path).unwrap();
    spawn(async move {
        let mut client_num = 0;
        while let Ok((mut stream, _)) = listener.accept().await {
            client_num += 1;
            spawn(async move {
                // Handshake
                let response_headers = Headers::new(
                    0,
                    MessageType::ConnectAck,
                    MessageFlags::ConnectionAccepted.into(),
                );
                let response = Message::new(response_headers, None::<ConnectResponse>);
                mock_greengrass_server_response(
                    &mut stream,
                    0,
                    MessageType::Connect,
                    MessageFlags::none(),
                    ConnectRequest::new().unwrap().payload(),
                    &response.to_bytes().unwrap(),
                )
                .await;

                if client_num == 1 {
                    // Receive set component state request.
                    let response_headers = Headers::new(
                        1,
                        MessageType::Application,
                        MessageFlags::TerminateStream.into(),
                    );
                    let response = Message::new(response_headers, Some(UpdateStateResponse {}));
                    mock_greengrass_server_response(
                        &mut stream,
                        1,
                        MessageType::Application,
                        MessageFlags::none(),
                        UpdateStateRequest::new(1, LifecycleState::Running).payload(),
                        &response.to_bytes().unwrap(),
                    )
                    .await;
                } else {
                    // Component update subscription is received from a second connection.
                    let response_headers =
                        Headers::new(1, MessageType::Application, MessageFlags::none());
                    let response = Message::new(
                        response_headers,
                        Some(ComponentUpdateSubscriptionResponse::new(None, None)),
                    );
                    mock_greengrass_server_response(
                        &mut stream,
                        1,
                        MessageType::Application,
                        MessageFlags::none(),
                        ComponentUpdateSubscriptionRequest::new(1).payload(),
                        &response.to_bytes().unwrap(),
                    )
                    .await;
                }

                // Not to drop the stream immediately.
                pending::<()>().await;
            });
        }
    })
}

async fn mock_greengrass_server_response<ReqPayload>(
    stream: &mut UnixStream,
    stream_id: i32,
    request_message_type: MessageType,
    request_message_flags: BitFlags<MessageFlags>,
    request_payload: Option<&ReqPayload>,
    response_bytes: &[u8],
) where
    for<'p> ReqPayload: PartialEq + std::fmt::Debug + serde::Deserialize<'p>,
{
    let mut buf = [0; 1024];

    // Handshake
    let n = stream.read(&mut buf).await.unwrap();
    let msg: Message<ReqPayload> = Message::from_bytes(&mut &buf[..n]).unwrap();
    assert_eq!(msg.headers().message_type(), request_message_type);
    assert_eq!(msg.headers().message_flags(), request_message_flags);
    assert_eq!(msg.headers().stream_id(), stream_id);
    assert_eq!(msg.payload(), request_payload);

    let _ = stream.write_all(response_bytes).await;
}

#[test(tokio::test)]
async fn test_ipc_client() {
    mock_greengrass_server();

    let mut client = IpcClient::new().await.unwrap();

    client.update_state(LifecycleState::Running).await.unwrap();

    client.pause_component_update().await.unwrap();
    // TODO: Add a test for defer_component_update.
    client.resume_component_update().await.unwrap();
}
