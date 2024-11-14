#pragma once
#include "rust/cxx.h"
#include <aws/greengrass/GreengrassCoreIpcClient.h>
#include <aws/crt/Api.h>

using Aws::Greengrass::GreengrassCoreIpcClient;

// FIXME: We shouldn't need to define this here but if we include the generated `ffi.rs.h`,
//        it leads to some strange issue about `IpcClient` declaration.
struct UpdateNotifier;

class IpcClient
{
public:
    IpcClient();

    rust::String connect(rust::Box<UpdateNotifier> update_notifier);
    rust::String deferComponentUpdate(uint64_t recheck_timeout_ms);

private:
    GreengrassCoreIpcClient *client;
    bool connected = false;
};

std::unique_ptr<IpcClient>
new_greengrass_client();