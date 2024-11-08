#pragma once
#include "rust/cxx.h"
#include <aws/greengrass/GreengrassCoreIpcClient.h>
#include <aws/crt/Api.h>

using Aws::Greengrass::GreengrassCoreIpcClient;

class IpcClient
{
public:
    GreengrassCoreIpcClient *client;
    bool connected = false;
    bool defer_updates = false;

    IpcClient();
};

std::unique_ptr<IpcClient>
new_greengrass_client();
rust::String client_connect(IpcClient &client);