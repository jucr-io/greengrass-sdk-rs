mod connection;
pub mod protocol;

use std::sync::{Arc, Mutex};
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Mutex as AsyncMutex,
    },
    task::JoinHandle,
};

pub struct IpcClient {
    inner: Arc<IpcClientInner>,
}

struct IpcClientInner {
    _component_update_event_tx: Sender<()>,
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

                    // TODO: Implement deferring of component update.
                    let _ = DEFER_COMPONENT_UPDATE_TIMEOUT_MS;
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
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        Self {
            inner: Arc::new(IpcClientInner {
                _component_update_event_tx: tx,
                component_update_event_rx: AsyncMutex::new(rx),
                pause_component_update_task: Mutex::new(None),
            }),
        }
    }

    fn connect(&self) -> Result<(), String> {
        // TODO:
        // 1. Connection to the IPC server over the unix socket.
        // 2. Subscribe to the component update event in a task, passing it the sender.

        unimplemented!();
    }
}

// 1 second.
const DEFER_COMPONENT_UPDATE_TIMEOUT_MS: u64 = 1000;
