use super::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UpdateStateRequest {
    #[serde(rename = "state")]
    state: crate::LifecycleState,
}

impl UpdateStateRequest {
    pub fn new(stream_id: i32, state: crate::LifecycleState) -> Message<'static, Self> {
        let payload = UpdateStateRequest { state };

        Message::ipc_call(
            "aws.greengrass#UpdateStateRequest",
            "aws.greengrass#UpdateState",
            stream_id,
            Some(payload),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UpdateStateResponse {}
