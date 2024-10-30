#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let client = Greengrass::new_greengrass_client();
        assert!(!client.is_null());
    }
}

#[cxx::bridge]
pub mod Greengrass {
    unsafe extern "C++" {
        include!("include/aws.h");

        type GreengrassCoreIpcClient;

        fn new_greengrass_client() -> UniquePtr<GreengrassCoreIpcClient>;
    }
}
