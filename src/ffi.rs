#[cxx::bridge]
pub(crate) mod Greengrass {
    unsafe extern "C++" {
        include!("include/aws.h");

        type IpcClient;

        fn new_greengrass_client() -> UniquePtr<IpcClient>;

        fn connect(self: Pin<&mut IpcClient>, update_notifier: Box<UpdateNotifier>) -> String;
        #[cxx_name = "deferComponentUpdate"]
        fn defer_component_update(self: Pin<&mut IpcClient>, recheck_timeout_ms: u64) -> String;
    }

    extern "Rust" {
        type UpdateNotifier;

        fn notify(self: &UpdateNotifier);
    }
}

// SAFETY: On the Rust side, we keep the IpcClient in a Mutex and hence ensure no 2 threads can
// access it simultaneously. Can't really say much for sure about the C++ side. We'll just have to
// test it thoroughly and find out the hard way if there are any issues.
unsafe impl Send for Greengrass::IpcClient {}

pub struct UpdateNotifier {
    sender: tokio::sync::mpsc::Sender<()>,
}

impl UpdateNotifier {
    pub(crate) fn new(sender: tokio::sync::mpsc::Sender<()>) -> Self {
        Self { sender }
    }

    pub fn notify(&self) {
        let _ = self.sender.blocking_send(());
    }
}
