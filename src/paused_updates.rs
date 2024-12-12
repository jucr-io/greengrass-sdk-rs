pub use crate::{connection::Connection, Error, Result};

use tracing::{debug, error, trace, warn};

pub struct PausedUpdates {
    conn: Connection,
    stream_id: i32,
}

impl PausedUpdates {
    pub async fn new() -> Result<Self> {
        let mut conn = Connection::new().await?;
        let stream_id = conn.subscribe_to_component_updates().await?;

        Ok(Self { conn, stream_id })
    }

    pub async fn keep_paused(mut self) {
        loop {
            trace!("Waiting for the next component update event..");
            let res = self.conn.read_response(self.stream_id, false).await;
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

            let messages = match update
                .payload()
                .as_ref()
                .and_then(|p| p.get("messages"))
                .and_then(|m| m.as_object())
            {
                Some(m) => m,
                None => {
                    warn!("Received component update without (expected) payload");

                    continue;
                }
            };
            let deployment_id = match messages
                .get("preUpdateEvent")
                .and_then(|m| m.as_object())
                .and_then(|e| e.get("deploymentId"))
                .and_then(|d| d.as_str())
            {
                Some(d) => d.to_string(),
                None => {
                    debug!("No `preUpdateEvent` in the update, ignoring..");

                    continue;
                }
            };
            drop(update);

            if let Err(e) = self
                .conn
                .defer_component_update(
                    &deployment_id,
                    None,
                    Some(DEFER_COMPONENT_UPDATE_TIMEOUT_MS),
                )
                .await
            {
                error!("Error deferring component update: {:?}", e);
            }
        }
    }
}

// 1 second.
const DEFER_COMPONENT_UPDATE_TIMEOUT_MS: u64 = 1000;
