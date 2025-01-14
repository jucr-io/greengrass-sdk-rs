use core::num::NonZeroU64;

use crate::{
    connection::Connection,
    protocol::{ComponentUpdateSubscriptionResponse, RecheckAfterMs},
    Error, Result,
};

use tracing::{debug, error, trace, warn};

pub(crate) struct PausedUpdates {
    conn: Connection,
    stream_id: i32,
}

impl PausedUpdates {
    pub(crate) async fn new(socket_path: &'static str, auth_token: &'static str) -> Result<Self> {
        let mut conn = Connection::new(socket_path, auth_token).await?;
        let stream_id = conn.subscribe_to_component_updates().await?;

        Ok(Self { conn, stream_id })
    }

    pub(crate) async fn keep_paused(mut self) {
        loop {
            trace!("Waiting for the next component update event..");
            let res = self
                .conn
                .read_response::<ComponentUpdateSubscriptionResponse>(self.stream_id, false)
                .await;
            let update = match res {
                Ok(update) => update,
                Err(e @ Error::Io(_))
                | Err(e @ Error::InternalServer(_))
                | Err(e @ Error::Protocol(_)) => {
                    error!("{e}");

                    break;
                }
                Err(e) => {
                    warn!("{e}");

                    continue;
                }
            };
            trace!("Received component update: {update:?}");

            let response = match update.payload() {
                Some(p) => p,
                None => {
                    warn!("Received component update without (expected) payload");

                    continue;
                }
            };
            let deployment_id = match response.pre_update_event().map(|e| e.deployment_id()) {
                Some(d) => d,
                None => {
                    debug!("No `preUpdateEvent` in the update, ignoring..");

                    continue;
                }
            };

            if let Err(e) = self
                .conn
                .defer_component_update(deployment_id, None, DEFER_COMPONENT_UPDATE_TIMEOUT_MS)
                .await
            {
                error!("Error deferring component update: {:?}", e);
            }
        }
    }
}

// 1 minute.
const DEFER_COMPONENT_UPDATE_TIMEOUT_MS: RecheckAfterMs =
    RecheckAfterMs::Defer(NonZeroU64::new(60_000).unwrap());
