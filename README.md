# greengrass-sdk

This is a pure Rust crate for communicating with the [AWS IoT Greengrass][aig] nucleus runtime.

It uses `tokio` for async I/O and this means that currently it is tokio-specific.

## Usage

While the low-level message (de)serialization and connection API is provided, it is recommended to
use the high-level [`IpcClient`] struct for most use cases.

```rust,no_run
use greengrass_sdk::{IpcClient, LifecycleState, Result};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Connect to the nucleus runtime.
    let mut client = IpcClient::from_env().await?;

    // You only want to do this if nucleus is not directly managing the lifecycle of your component.
    client.update_state(LifecycleState::Running).await?;

    client.pause_component_update().await?;

    // Do some work that should not be interrupted by component updates.

    client.resume_component_update().await?;

    Ok(())
}
```

**Note:** The [`IpcClient::from_env`] method used in the example above, requires `SVCUID` and
`AWS_GG_NUCLEUS_DOMAIN_SOCKET_FILEPATH_FOR_COMPONENT` environment variables to be set
appropriately. The nucleus runtime sets them for components it launches directly.

## Status

The low-level API is complete but the high-level API currently only addresses the most common use
cases. PRs to add more functionality are most welcome.

## License

This project is licensed under the MIT license.

[aig]: https://aws.amazon.com/greengrass/
[`IpcClient`]: https:/docs.rs/greengrass_sdk/struct.IpcClient.html
[`IpcClient::from_env`]: https:/docs.rs/greengrass_sdk/struct.IpcClient.html#method.from_env