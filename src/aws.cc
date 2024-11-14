#include <iostream>
#include <string>
#include <string_view>
#include "include/aws.h"
#include "greengrass-sdk-rs/src/ffi.rs"

using namespace Aws::Crt::Io;
using namespace Aws::Greengrass;

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

class ComponentUpdateResponseHandler : public SubscribeToComponentUpdatesStreamHandler
{
public:
    ComponentUpdateResponseHandler(rust::Box<UpdateNotifier> update_notifier)
        : update_notifier(std::move(update_notifier))
    {
    }

    virtual ~ComponentUpdateResponseHandler() {}

private:
    void OnStreamEvent(ComponentUpdatePolicyEvents *response) override
    {
        auto event = response->GetPreUpdateEvent();
        if (!event.has_value())
        {
            return;
        }

        this->update_notifier.into_raw()->notify();
    }

    bool OnStreamError(OperationError *error) override
    {
        // Handle error.
        (void)error;
        return false; // Return true to close stream, false to keep stream open.
    }

    void OnStreamClosed() override
    {
        // Handle close.
    }

    rust::Box<UpdateNotifier> update_notifier;
};

rust::String IpcClient::connect(rust::Box<UpdateNotifier> update_notifier)
{
    IpcClientLifecycleHandler lifecycleHandler;
    auto connectionStatus = this->client->Connect(lifecycleHandler).get();
    if (!connectionStatus)
    {
        auto str = std::string(connectionStatus.StatusToString());
        return rust::String(str);
    }
    this->connected = true;

    SubscribeToComponentUpdatesRequest request;
    auto handler = std::make_shared<ComponentUpdateResponseHandler>(std::move(update_notifier));
    auto operation = this->client->NewSubscribeToComponentUpdates(handler);
    auto activate = operation->Activate(request);
    activate.wait();

    auto responseFuture = operation->GetResult();
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

    // FIXME: Find a way to return a null string.
    return rust::String("");
}

rust::String IpcClient::deferComponentUpdate(uint64_t recheck_timeout_ms)
{
    auto deferComponentUpdate = this->client->NewDeferComponentUpdate();
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

    // FIXME: Find a way to return a null string.
    return rust::String("");
}

/// @brief  Create a new Greengrass IPC client and connect.
/// @return A unique pointer to the Greengrass IPC client that is connected.
std::unique_ptr<IpcClient>
new_greengrass_client()
{
    return std::make_unique<IpcClient>();
}