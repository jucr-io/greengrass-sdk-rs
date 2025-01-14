#![deny(missing_debug_implementations, nonstandard_style, rust_2018_idioms, missing_docs)]
#![warn(unreachable_pub, clippy::std_instead_of_core)]
#![doc = include_str!("../README.md")]

mod connection;
mod error;
pub mod protocol;
pub use error::{Error, Result};
#[doc(hidden)]
pub mod env;
mod lifecycle_state;
pub use lifecycle_state::LifecycleState;
mod paused_updates;

pub use connection::Connection;
use tokio::task::JoinHandle;

#[derive(Debug)]
/// The IPC client for interacting with the Greengrass Nucleus.
pub struct IpcClient {
    conn: Connection,
    component_update_task: Option<JoinHandle<()>>,
}

impl IpcClient {
    /// Create a new IPC client.
    pub async fn new(socket_path: &'static str, auth_token: &'static str) -> Result<Self> {
        let conn = Connection::new(socket_path, auth_token).await?;

        Ok(Self { conn, component_update_task: None })
    }

    /// Create a new IPC client, fetching the socket path and auth token from the environment.
    pub async fn from_env() -> Result<Self> {
        let conn = Connection::from_env().await?;

        Ok(Self { conn, component_update_task: None })
    }

    /// Pause component updates.
    ///
    /// After this call the, the component updates will be paused until
    /// [`IpcClient::resume_component_update`] is called.
    pub async fn pause_component_update(&mut self) -> Result<()> {
        if self.component_update_task.is_some() {
            return Ok(());
        }

        let paused_updates =
            paused_updates::PausedUpdates::new(self.conn.socket_path(), self.conn.auth_token())
                .await?;
        let component_update_task = tokio::spawn(paused_updates.keep_paused());
        assert!(self.component_update_task.replace(component_update_task).is_none());

        Ok(())
    }

    /// Resume component updates.
    ///
    /// This will resume the component updates that were paused by calling
    /// [`IpcClient::pause_component_update`].
    pub async fn resume_component_update(&mut self) -> Result<()> {
        if let Some(handle) = self.component_update_task.take() {
            handle.abort();
        }

        Ok(())
    }

    /// Manually update the state of the component.
    ///
    /// This is useful when you want to update the state of the component manually.
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
