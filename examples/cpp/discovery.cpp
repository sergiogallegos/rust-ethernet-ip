#include "rust_ethernet_ip.h"

#include <array>
#include <iostream>
#include <string>

static std::string last_error(int client_id)
{
    std::array<char, 1024> buffer {};
    const int written = eip_get_last_error(client_id, buffer.data(), static_cast<int>(buffer.size()));
    return written > 0 ? std::string(buffer.data(), static_cast<std::size_t>(written)) : std::string {};
}

int main(int argc, char **argv)
{
    if (argc != 2) {
        std::cerr << "usage: cpp_discovery PLC_IP:44818\n";
        return 2;
    }

    const int client_id = eip_connect(argv[1]);
    if (client_id <= 0) {
        std::cerr << "connect failed: " << last_error(client_id) << "\n";
        return 1;
    }

    EipTagDiscoveryResult discovery {};
    const int rc = eip_discover_tags_detailed_by_id(client_id, &discovery);
    if (rc != 0 || !discovery.success) {
        std::cerr << "discovery failed: "
                  << (discovery.error_message != nullptr ? discovery.error_message : last_error(client_id))
                  << "\n";
        eip_free_tag_discovery_result(&discovery);
        eip_disconnect(client_id);
        return 1;
    }

    for (int i = 0; i < discovery.tag_count; ++i) {
        const EipTagAttributes &tag = discovery.tags[i];
        std::cout << (tag.name != nullptr ? tag.name : "") << "\t"
                  << (tag.data_type_name != nullptr ? tag.data_type_name : "") << "\t"
                  << tag.size << " bytes\n";
    }

    eip_free_tag_discovery_result(&discovery);
    return eip_disconnect(client_id) == 0 ? 0 : 1;
}
