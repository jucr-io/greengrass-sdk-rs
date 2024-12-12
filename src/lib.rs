mod connection;
mod error;
pub mod protocol;
pub use error::{Error, Result};
mod env;
mod lifecycle_state;
pub use lifecycle_state::LifecycleState;

use connection::Connection;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::task::JoinHandle;
use tracing::{debug, error, trace, warn};

pub struct IpcClient {
    conn: Connection,
    component_update_task: JoinHandle<()>,
    paused: Arc<AtomicBool>,
}

impl IpcClient {
    pub async fn new() -> Result<Self> {
        let conn = Connection::new().await?;
        let paused = Arc::new(AtomicBool::new(false));

        // Create a separate connection for component update subscription.
        let mut stream_conn = Connection::new().await?;
        let stream_id = stream_conn.subscribe_to_component_updates().await?;
        let paused_clone = paused.clone();
        let component_update_task = tokio::spawn(async move {
            loop {
                trace!("Waiting for the next component update event..");
                let update = match stream_conn.read_response(stream_id, false).await {
                    Ok(update) => update,
                    Err(e) => {
                        // FIXME: We need better error handling here and better way to differentiate
                        //        between recoverable and non-recoverable errors.
                        error!("{e}");
                        break;
                    }
                };
                trace!("Received component update: {update:?}");

                let paused = paused_clone.load(Ordering::Relaxed);
                if !paused {
                    debug!("component update not paused. Not deferring it..");
                    continue;
                }

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
                        continue;
                    }
                };
                drop(update);

                if let Err(e) = stream_conn
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
        });

        Ok(Self { conn, component_update_task, paused })
    }

    pub fn pause_component_update(&mut self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume_component_update(&mut self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub async fn update_state(&mut self, state: LifecycleState) -> Result<()> {
        self.conn.update_state(state).await
    }
}

impl Drop for IpcClient {
    fn drop(&mut self) {
        self.component_update_task.abort();
    }
}

// 1 second.
const DEFER_COMPONENT_UPDATE_TIMEOUT_MS: u64 = 1000;
