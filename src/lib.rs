mod ffi;

use ffi::Greengrass;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _client = IpcClient::new_connected().unwrap();
    }
}

pub struct IpcClient {
    client: cxx::UniquePtr<Greengrass::IpcClient>,
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
