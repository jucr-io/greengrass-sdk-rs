#include "include/aws.h"
#include <aws/greengrass/GreengrassCoreIpcClient.h>
#include <aws/crt/Api.h>

std::unique_ptr<GreengrassCoreIpcClient> new_greengrass_client()
{
    Aws::Crt::Allocator *allocator = Aws::Crt::DefaultAllocator();
    Aws::Crt::Io::ClientBootstrap bootstrap(allocator);

    return std::make_unique<GreengrassCoreIpcClient>(bootstrap, allocator);
}