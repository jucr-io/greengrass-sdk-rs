use std::{
    env::{self, temp_dir},
    future::pending,
};

use enumflags2::BitFlags;
use greengrass_sdk_rs::{
    env::{AUTH_TOKEN_ENV, SOCKET_PATH_ENV},
    protocol::{
        headers::{Headers, MessageFlags, MessageType},
        ComponentUpdateSubscriptionRequest, ComponentUpdateSubscriptionResponse, ConnectRequest,
        ConnectResponse, DeferComponentUpdateRequest, DeferComponentUpdateResponse, Message,
        PreComponentUpdateEvent, RecheckAfterMs, UpdateStateRequest, UpdateStateResponse,
    },
    IpcClient, LifecycleState,
};
use test_log::test;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    spawn,
    sync::broadcast::{channel, Sender},
    task::JoinHandle,
};

fn mock_greengrass_server(deferred_notifier: Sender<()>) -> JoinHandle<()> {
    let filename = format!("greengrass-sdk-rs-{}", fastrand::f64());
    let path = temp_dir().join(filename);
    env::set_var(SOCKET_PATH_ENV, path.as_os_str());
    env::set_var(AUTH_TOKEN_ENV, "whatever");

    let listener = UnixListener::bind(path).unwrap();
    spawn(async move {
        let mut client_num = 0;
        while let Ok((mut stream, _)) = listener.accept().await {
            client_num += 1;
            let deferred_notifier = deferred_notifier.clone();
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
                        response_headers.clone(),
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

                    // Now send out a component pre-update event.
                    let deployment_id = "77d00c6b-f0c6-4e14-86cb-d476f0016044".parse().unwrap();
                    let pre_update_event = PreComponentUpdateEvent::new(deployment_id, false);
                    let response = Message::new(
                        response_headers,
                        Some(ComponentUpdateSubscriptionResponse::new(
                            Some(pre_update_event),
                            None,
                        )),
                    );
                    let _ = stream.write_all(&response.to_bytes().unwrap()).await;

                    // Now receive a defer component update request and respond with success.
                    let response_headers = Headers::new(
                        2,
                        MessageType::Application,
                        MessageFlags::TerminateStream.into(),
                    );
                    let mut buf = [0; 1024];

                    let n = stream.read(&mut buf).await.unwrap();
                    let msg: Message<DeferComponentUpdateRequest> =
                        Message::from_bytes(&mut &buf[..n]).unwrap();
                    assert_eq!(msg.headers().message_type(), MessageType::Application);
                    assert_eq!(msg.headers().message_flags(), MessageFlags::none());
                    assert_eq!(msg.headers().stream_id(), 2);
                    let request = msg.payload().unwrap();
                    assert_eq!(request.deployment_id(), deployment_id);
                    assert_eq!(request.component_name(), None);
                    assert_eq!(
                        request.recheck_after_ms(),
                        RecheckAfterMs::Defer(60_000.try_into().unwrap())
                    );

                    let response =
                        Message::new(response_headers, Some(DeferComponentUpdateResponse {}));
                    let _ = stream.write_all(&response.to_bytes().unwrap()).await;

                    deferred_notifier.send(()).unwrap();
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
    let (sender, mut receiver) = channel(1);
    mock_greengrass_server(sender);

    let mut client = IpcClient::new().await.unwrap();

    client.update_state(LifecycleState::Running).await.unwrap();

    client.pause_component_update().await.unwrap();
    receiver.recv().await.unwrap();
    client.resume_component_update().await.unwrap();
}
