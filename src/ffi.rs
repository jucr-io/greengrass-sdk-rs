#[cxx::bridge]
pub(crate) mod Greengrass {
    unsafe extern "C++" {
        include!("include/aws.h");

        type GreengrassCoreIpcClient;

        fn new_greengrass_client() -> UniquePtr<GreengrassCoreIpcClient>;
        fn client_connect(client: Pin<&mut GreengrassCoreIpcClient>) -> String;
    }
}
