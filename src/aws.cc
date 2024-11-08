#include <iostream>
#include <string>
#include <string_view>
#include "include/aws.h"

using namespace Aws::Crt::Io;
using Aws::Greengrass::DeferComponentUpdateRequest;

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

rust::String client_defer_component_update(IpcClient &client, uint64_t recheck_timeout_ms)
{
    auto deferComponentUpdate = client.client->NewDeferComponentUpdate();
    DeferComponentUpdateRequest deferComponentUpdateRequest;
    deferComponentUpdateRequest.SetRecheckAfterMs(recheck_timeout_ms);

    auto activate = deferComponentUpdate->Activate(deferComponentUpdateRequest);
    activate.wait();

    auto responseFuture = deferComponentUpdate->GetResult();
    if (responseFuture.wait_for(std::chrono::seconds(5)) == std::future_status::timeout)
    {
        return rust::String("Operation timed out.");
    }
    auto response = responseFuture.get();
    if (!response)
    {
        // Handle error.
        auto errorType = response.GetResultType();
        if (errorType == OPERATION_ERROR)
        {
            auto msg = response.GetOperationError()->GetMessage();
            if (!msg.has_value())
            {
                return rust::String("Unknown error.");
            }
            return rust::String(std::string(msg.value()));
        }
        else
        {
            auto msg = response.GetRpcError().StatusToString();
            return rust::String(std::string(msg));
        }
    }

    client.defer_updates = true;

    // FIXME: Find a way to return a null string.
    return rust::String("");
}