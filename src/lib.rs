#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _ = ffi::GreengrassCoreIpcClient(8);
    }
}

#[cxx::bridge(namespace = "aws::Greengrass")]
pub mod ffi {
    unsafe extern "C++" {
        include!("aws/greengrass/GreengrassCoreIpcClient.h");

        type GreengrassCoreIpcClient;

        fn GreengrassCoreIpcClient(blah: u8) -> UniquePtr<GreengrassCoreIpcClient>;
    }
}
