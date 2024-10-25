#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        /*include!("cxx-demo/include/blobstore.h");

        type BlobstoreClient;

        fn new_blobstore_client() -> UniquePtr<BlobstoreClient>;*/
    }
}
