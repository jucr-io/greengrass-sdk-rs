#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _ = Greengrass::new_greengrass_client();
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
