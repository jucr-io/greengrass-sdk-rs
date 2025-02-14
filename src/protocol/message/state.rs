use super::Message;
use serde::{Deserialize, Serialize};

/// A request to update the state of a component.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UpdateStateRequest {
    #[serde(rename = "state")]
    state: crate::LifecycleState,
}

impl UpdateStateRequest {
    /// Creates a new `UpdateStateRequest`.
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

/// A response to a state update request.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UpdateStateResponse {}
