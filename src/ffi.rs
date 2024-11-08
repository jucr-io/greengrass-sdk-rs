#[cxx::bridge]
pub(crate) mod Greengrass {
    unsafe extern "C++" {
        include!("include/aws.h");

        type IpcClient;

        fn new_greengrass_client() -> UniquePtr<IpcClient>;
        fn client_connect(client: Pin<&mut IpcClient>) -> String;
    }
}
