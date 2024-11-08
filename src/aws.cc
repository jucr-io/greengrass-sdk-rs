#include <iostream>
#include <string>
#include <string_view>
#include "include/aws.h"

using namespace Aws::Crt::Io;

IpcClient::IpcClient()
{
    EventLoopGroup eventLoopGroup(1);
    DefaultHostResolver socketResolver(eventLoopGroup, 64, 30);
    ClientBootstrap bootstrap(eventLoopGroup, socketResolver);

    this->client = new GreengrassCoreIpcClient(bootstrap);
}

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
std::unique_ptr<IpcClient>
new_greengrass_client()
{
    return std::make_unique<IpcClient>();
}

rust::String client_connect(IpcClient &client)
{
    IpcClientLifecycleHandler lifecycleHandler;
    auto connectionStatus = client.client->Connect(lifecycleHandler).get();
    if (!connectionStatus)
    {
        auto str = std::string(connectionStatus.StatusToString());
        return rust::String(str);
    }
    client.connected = true;

    // FIXME: Find a way to return a null string.
    return rust::String("");
}