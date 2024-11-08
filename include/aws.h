#pragma once
#include "rust/cxx.h"
#include <aws/greengrass/GreengrassCoreIpcClient.h>
#include <aws/crt/Api.h>

using Aws::Greengrass::GreengrassCoreIpcClient;

class IpcClient
{
public:
    IpcClient();

    rust::String connect();
    rust::String deferComponentUpdate(uint64_t recheck_timeout_ms);

private:
    GreengrassCoreIpcClient *client;
    bool connected = false;
    bool defer_updates = false;
};

std::unique_ptr<IpcClient>
new_greengrass_client();