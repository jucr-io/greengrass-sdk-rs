use super::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DeferComponentUpdateRequest<'a> {
    #[serde(rename = "deploymentId")]
    deployment_id: &'a str,
    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    message: Option<&'a str>,
    #[serde(rename = "recheckAfterMs", skip_serializing_if = "Option::is_none")]
    recheck_after_ms: Option<u64>,
}

impl<'m> DeferComponentUpdateRequest<'m> {
    pub fn new(
        stream_id: i32,
        deployment_id: &'m str,
        component_name: Option<&'m str>,
        recheck_after_ms: Option<u64>,
    ) -> Message<'m, Self> {
        let payload = DeferComponentUpdateRequest {
            deployment_id,
            message: component_name,
            recheck_after_ms,
        };

        Message::ipc_call(
            "aws.greengrass#DeferComponentUpdateRequest",
            "aws.greengrass#DeferComponentUpdate",
            stream_id,
            Some(payload),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeferComponentUpdateResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComponentUpdateSubscriptionRequest {}

impl<'m> ComponentUpdateSubscriptionRequest {
    pub fn new(stream_id: i32) -> Message<'m, Self> {
        Message::ipc_call(
            "aws.greengrass#SubscribeToComponentUpdatesRequest",
            "aws.greengrass#SubscribeToComponentUpdates",
            stream_id,
            None,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComponentUpdateSubscriptionResponse<'c> {
    #[serde(borrow, rename = "preUpdateEvent")]
    pre_update_event: Option<PreComponentUpdateEvent<'c>>,
    #[serde(rename = "postUpdateEvent")]
    post_update_event: Option<PostComponentUpdateEvent<'c>>,
}

impl ComponentUpdateSubscriptionResponse<'_> {
    pub fn pre_update_event(&self) -> Option<&PreComponentUpdateEvent> {
        self.pre_update_event.as_ref()
    }

    pub fn post_update_event(&self) -> Option<&PostComponentUpdateEvent> {
        self.post_update_event.as_ref()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreComponentUpdateEvent<'p> {
    #[serde(rename = "deploymentId")]
    deployment_id: &'p str,
    #[serde(rename = "isGgcRestarting")]
    is_ggc_restarting: bool,
}

impl PreComponentUpdateEvent<'_> {
    pub fn deployment_id(&self) -> &str {
        self.deployment_id
    }

    pub fn is_ggc_restarting(&self) -> bool {
        self.is_ggc_restarting
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostComponentUpdateEvent<'p> {
    #[serde(rename = "deploymentId")]
    deployment_id: &'p str,
}

impl PostComponentUpdateEvent<'_> {
    pub fn deployment_id(&self) -> &str {
        self.deployment_id
    }
}
