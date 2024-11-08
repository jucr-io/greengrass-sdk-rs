#[cxx::bridge]
pub(crate) mod Greengrass {
    unsafe extern "C++" {
        include!("include/aws.h");

        type IpcClient;

        fn new_greengrass_client() -> UniquePtr<IpcClient>;

        fn connect(self: Pin<&mut IpcClient>) -> String;
        #[cxx_name = "deferComponentUpdate"]
        fn defer_component_update(self: Pin<&mut IpcClient>, recheck_timeout_ms: u64) -> String;
    }
}
