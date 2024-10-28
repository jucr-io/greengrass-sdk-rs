#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _ = Greengrass::new_greengrass_client();
        //let _ = Allocator::Allocator();
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

/*#[cxx::bridge(namespace = "Aws::Crt::Io")]
pub mod Bootstrap {
    unsafe extern "C++" {
        include!("aws/crt/io/Bootstrap.h");

        type ClientBootstrap;

        fn ClientBootstrap(allocator: &crate::Allocator::Allocator) -> UniquePtr<ClientBootstrap>;
    }
}

#[cxx::bridge(namespace = "Aws::Crt")]
pub mod Crt {
    unsafe extern "C++" {
        include!("aws/crt/Api.h");

        type Allocator;

        fn DefaultAllocator() -> Allocator;
    }
}*/
