mod connection;
mod error;
pub mod protocol;
pub use error::{Error, Result};
mod env;
mod lifecycle_state;
pub use lifecycle_state::LifecycleState;
mod paused_updates;

use connection::Connection;
use tokio::task::JoinHandle;

pub struct IpcClient {
    conn: Connection,
    component_update_task: Option<JoinHandle<()>>,
}

impl IpcClient {
    pub async fn new() -> Result<Self> {
        let conn = Connection::new().await?;

        Ok(Self { conn, component_update_task: None })
    }

    pub async fn pause_component_update(&mut self) -> Result<()> {
        if self.component_update_task.is_some() {
            return Ok(());
        }

        let paused_updates = paused_updates::PausedUpdates::new().await?;
        let component_update_task = tokio::spawn(paused_updates.keep_paused());
        assert!(self.component_update_task.replace(component_update_task).is_none());

        Ok(())
    }

    pub async fn resume_component_update(&mut self) -> Result<()> {
        if let Some(handle) = self.component_update_task.take() {
            handle.abort();
        }

        Ok(())
    }

    pub async fn update_state(&mut self, state: LifecycleState) -> Result<()> {
        self.conn.update_state(state).await
    }
}

impl Drop for IpcClient {
    fn drop(&mut self) {
        if let Some(handle) = self.component_update_task.take() {
            handle.abort();
        }
    }
}
