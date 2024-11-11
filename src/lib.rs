mod ffi;

use ffi::Greengrass;
use std::sync::{Arc, Mutex};
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Mutex as AsyncMutex,
    },
    task::JoinHandle,
};
use tracing::error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _client = IpcClient::new_connected().unwrap();
    }
}

pub struct IpcClient {
    inner: Arc<IpcClientInner>,
}

struct IpcClientInner {
    client: Mutex<cxx::UniquePtr<Greengrass::IpcClient>>,
    component_update_event_tx: Sender<()>,
    component_update_event_rx: AsyncMutex<Receiver<()>>,
    pause_component_update_task: Mutex<Option<JoinHandle<()>>>,
}

impl IpcClient {
    pub fn new_connected() -> Result<IpcClient, String> {
        let client = IpcClient::new();

        client.connect().map(|_| client)
    }

    pub fn pause_component_update(&self) {
        let inner = self.inner.clone();
        self.inner
            .pause_component_update_task
            .lock()
            .unwrap()
            .get_or_insert(tokio::spawn(async move {
                loop {
                    let _ = inner.component_update_event_rx.lock().await.recv().await;

                    match inner
                        .client
                        .lock()
                        .unwrap()
                        .pin_mut()
                        .defer_component_update(DEFER_COMPONENT_UPDATE_TIMEOUT_MS)
                        .as_str()
                    {
                        "" => (),
                        err => error!("{}", err.to_string()),
                    }
                }
            }));
    }

    pub fn resume_component_update(&self) {
        if let Some(handle) = self
            .inner
            .pause_component_update_task
            .lock()
            .unwrap()
            .take()
        {
            handle.abort();
        }
    }

    fn new() -> Self {
        let client = Greengrass::new_greengrass_client();
        if client.is_null() {
            // Don't think this happens, but just in case.
            panic!("Failed to create IPC client");
        }

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        Self {
            inner: Arc::new(IpcClientInner {
                client: Mutex::new(client),
                component_update_event_tx: tx,
                component_update_event_rx: AsyncMutex::new(rx),
                pause_component_update_task: Mutex::new(None),
            }),
        }
    }

    fn connect(&self) -> Result<(), String> {
        let update_notifier =
            ffi::UpdateNotifier::new(self.inner.component_update_event_tx.clone()).into();
        match self
            .inner
            .client
            .lock()
            .unwrap()
            .pin_mut()
            .connect(update_notifier)
            .as_str()
        {
            "" => Ok(()),
            err => Err(err.to_string()),
        }
    }
}

// 1 second.
const DEFER_COMPONENT_UPDATE_TIMEOUT_MS: u64 = 1000;
