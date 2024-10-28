#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _ = Greengrass::GreengrassCoreIpcClient();
    }
}

#[cxx::bridge(namespace = "Aws::Greengrass")]
pub mod Greengrass {
    unsafe extern "C++" {
        include!("aws/greengrass/GreengrassCoreIpcClient.h");

        type GreengrassCoreIpcClient;

        fn GreengrassCoreIpcClient() -> UniquePtr<GreengrassCoreIpcClient>;
    }
}

#[cxx::bridge(namespace = "Aws::Crt::Io")]
pub mod Bootstrap {
    unsafe extern "C++" {
        include!("aws/crt/io/Bootstrap.h");

        type ClientBootstrap;

        fn ClientBootstrap(
            allocator: UniquePtr<Allocator::Allocator>,
        ) -> UniquePtr<ClientBootstrap>;
    }
}

#[cxx::bridge(namespace = "Aws::Crt::Allocator")]
pub mod Allocator {
    unsafe extern "C++" {
        include!("aws/crt/Allocator.h");

        type Allocator;

        fn DefaultAllocator() -> UniquePtr<Allocator>;
    }
}
