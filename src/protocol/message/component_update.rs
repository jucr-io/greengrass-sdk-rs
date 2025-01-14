use super::Message;
use core::num::NonZeroU64;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A request to defer a component update.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct DeferComponentUpdateRequest<'a> {
    #[serde(rename = "deploymentId")]
    deployment_id: Uuid,
    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    message: Option<&'a str>,
    #[serde(rename = "recheckAfterMs")]
    recheck_after_ms: RecheckAfterMs,
}

impl<'m> DeferComponentUpdateRequest<'m> {
    /// Creates a new `DeferComponentUpdateRequest`.
    pub fn new(
        stream_id: i32,
        deployment_id: Uuid,
        component_name: Option<&'m str>,
        recheck_after_ms: RecheckAfterMs,
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

    /// Returns the deployment ID parameter.
    pub fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }

    /// Returns the component name parameter.
    pub fn component_name(&self) -> Option<&str> {
        self.message
    }

    /// Returns the recheck after milliseconds parameter.
    pub fn recheck_after_ms(&self) -> RecheckAfterMs {
        self.recheck_after_ms
    }
}

/// The number of milliseconds to defer a component update.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RecheckAfterMs {
    /// Do not defer the component update.
    DontDefer,
    /// Defer the component update for the specified number of milliseconds.
    Defer(NonZeroU64),
}

impl Serialize for RecheckAfterMs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let value = match self {
            RecheckAfterMs::DontDefer => 0,
            RecheckAfterMs::Defer(ms) => ms.get(),
        };

        serializer.serialize_u64(value)
    }
}

impl<'de> Deserialize<'de> for RecheckAfterMs {
    fn deserialize<D>(deserializer: D) -> Result<RecheckAfterMs, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        u64::deserialize(deserializer).map(|v| match v {
            0 => RecheckAfterMs::DontDefer,
            v => RecheckAfterMs::Defer(NonZeroU64::new(v).unwrap()),
        })
    }
}

/// A response to a `DeferComponentUpdateRequest`.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct DeferComponentUpdateResponse {}

/// A request to update the state of a component.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ComponentUpdateSubscriptionRequest {}

impl ComponentUpdateSubscriptionRequest {
    /// Creates a new `ComponentUpdateSubscriptionRequest`.
    pub fn new(stream_id: i32) -> Message<'static, Self> {
        Message::ipc_call(
            "aws.greengrass#SubscribeToComponentUpdatesRequest",
            "aws.greengrass#SubscribeToComponentUpdates",
            stream_id,
            None,
        )
    }
}

/// A response to a `ComponentUpdateSubscriptionRequest`.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ComponentUpdateSubscriptionResponse {
    #[serde(rename = "preUpdateEvent", skip_serializing_if = "Option::is_none")]
    pre_update_event: Option<PreComponentUpdateEvent>,
    #[serde(rename = "postUpdateEvent", skip_serializing_if = "Option::is_none")]
    post_update_event: Option<PostComponentUpdateEvent>,
}

impl ComponentUpdateSubscriptionResponse {
    /// Creates a new `ComponentUpdateSubscriptionResponse`.
    pub fn new(
        pre_update_event: Option<PreComponentUpdateEvent>,
        post_update_event: Option<PostComponentUpdateEvent>,
    ) -> Self {
        Self { pre_update_event, post_update_event }
    }

    /// the pre-update event.
    pub fn pre_update_event(&self) -> Option<&PreComponentUpdateEvent> {
        self.pre_update_event.as_ref()
    }

    /// the post-update event.
    pub fn post_update_event(&self) -> Option<&PostComponentUpdateEvent> {
        self.post_update_event.as_ref()
    }
}

/// An event that occurs before a component update.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PreComponentUpdateEvent {
    #[serde(rename = "deploymentId")]
    deployment_id: Uuid,
    #[serde(rename = "isGgcRestarting")]
    is_ggc_restarting: bool,
}

impl PreComponentUpdateEvent {
    /// Creates a new `PreComponentUpdateEvent`.
    pub fn new(deployment_id: Uuid, is_ggc_restarting: bool) -> Self {
        Self { deployment_id, is_ggc_restarting }
    }

    /// the deployment ID parameter.
    pub fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }

    /// Is the Nucleus restarting.
    pub fn is_ggc_restarting(&self) -> bool {
        self.is_ggc_restarting
    }
}

/// An event that occurs after a component update.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PostComponentUpdateEvent {
    #[serde(rename = "deploymentId")]
    deployment_id: Uuid,
}

impl PostComponentUpdateEvent {
    /// Creates a new `PostComponentUpdateEvent`.
    pub fn new(deployment_id: Uuid) -> Self {
        Self { deployment_id }
    }

    /// Returns the deployment ID parameter.
    pub fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }
}
