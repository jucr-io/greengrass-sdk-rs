#pragma once
#include "rust/cxx.h"
#include <aws/greengrass/GreengrassCoreIpcClient.h>

using Aws::Greengrass::GreengrassCoreIpcClient;

std::unique_ptr<GreengrassCoreIpcClient> new_greengrass_client();
rust::String client_connect(GreengrassCoreIpcClient &client);