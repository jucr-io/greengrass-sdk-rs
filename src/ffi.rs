#[cxx::bridge]
pub(crate) mod Greengrass {
    unsafe extern "C++" {
        include!("include/aws.h");

        type IpcClient;

        fn new_greengrass_client() -> UniquePtr<IpcClient>;
        fn client_connect(client: Pin<&mut IpcClient>) -> String;
        fn client_defer_component_update(
            client: Pin<&mut IpcClient>,
            recheck_timeout_ms: u64,
        ) -> String;
    }
}
