#include <iostream>
#include <string>
#include <string_view>
#include <format>
#include "include/aws.h"
#include <aws/crt/Api.h>

using namespace Aws::Crt::Io;

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
    EventLoopGroup eventLoopGroup(1);
    DefaultHostResolver socketResolver(eventLoopGroup, 64, 30);
    ClientBootstrap bootstrap(eventLoopGroup, socketResolver);

    return std::make_unique<GreengrassCoreIpcClient>(bootstrap);
}

rust::String client_connect(GreengrassCoreIpcClient &client)
{
    IpcClientLifecycleHandler lifecycleHandler;
    auto connectionStatus = client.Connect(lifecycleHandler).get();
    if (!connectionStatus)
    {
        auto str = std::format("{}", connectionStatus.StatusToString());
        return rust::String(str);
    }

    // FIXME: Find a way to return a null string.
    return rust::String("");
}