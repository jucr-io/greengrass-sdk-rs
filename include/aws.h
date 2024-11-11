#pragma once
#include "rust/cxx.h"
#include <aws/greengrass/GreengrassCoreIpcClient.h>
#include <aws/crt/Api.h>

using Aws::Greengrass::GreengrassCoreIpcClient;

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