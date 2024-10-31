#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _client = IpcClient::new_connected().unwrap();
    }
}

#[cxx::bridge]
mod Greengrass {
    unsafe extern "C++" {
        include!("include/aws.h");

        type GreengrassCoreIpcClient;

        fn new_greengrass_client() -> UniquePtr<GreengrassCoreIpcClient>;
        fn client_connect(client: Pin<&mut GreengrassCoreIpcClient>) -> String;
    }
}

pub struct IpcClient {
    client: cxx::UniquePtr<Greengrass::GreengrassCoreIpcClient>,
}

impl IpcClient {
    pub fn new_connected() -> Result<IpcClient, String> {
        let mut client = IpcClient::new();

        client.connect().map(|_| client)
    }

    fn new() -> IpcClient {
        let client = Greengrass::new_greengrass_client();
        if client.is_null() {
            // Don't think this happens, but just in case.
            panic!("Failed to create IPC client");
        }

        IpcClient { client }
    }

    fn connect(&mut self) -> Result<(), String> {
        match Greengrass::client_connect(self.client.pin_mut()).as_str() {
            "" => Ok(()),
            err => Err(err.to_string()),
        }
    }
}
