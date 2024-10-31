#include <iostream>
#include "include/aws.h"
#include <aws/greengrass/GreengrassCoreIpcClient.h>
#include <aws/crt/Api.h>

// FIXME: What do we do in the handlers here?
class IpcClientLifecycleHandler : public ConnectionLifecycleHandler
{
    void OnConnectCallback() override
    {
        // Handle connection to IPC service.
    }

    void OnDisconnectCallback(RpcError error) override
    {
        // Handle disconnection from IPC service.
        (void)error;
    }

    bool OnErrorCallback(RpcError error) override
    {
        // Handle IPC service connection error.
        (void)error;
        return true;
    }
};

/// @brief  Create a new Greengrass IPC client and connect.
/// @return A unique pointer to the Greengrass IPC client that is connected.
std::unique_ptr<GreengrassCoreIpcClient> new_greengrass_client()
{
    Aws::Crt::Allocator *allocator = Aws::Crt::DefaultAllocator();
    Aws::Crt::Io::ClientBootstrap bootstrap(allocator);
    IpcClientLifecycleHandler lifecycleHandler;
    auto client = std::make_unique<GreengrassCoreIpcClient>(bootstrap);

    auto connectionStatus = client.get()->Connect(lifecycleHandler).get();
    if (!connectionStatus)
    {
        std::cerr << "Failed to establish IPC connection: " << connectionStatus.StatusToString() << std::endl;

        return nullptr;
    }

    return client;
}