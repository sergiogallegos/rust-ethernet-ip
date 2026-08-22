#include "rust_ethernet_ip.h"

#include <array>
#include <cstdint>
#include <cstdlib>
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
    if (argc != 3) {
        std::cerr << "usage: cpp_route_and_diagnostics MODULE_IP:44818 CPU_SLOT\n";
        return 2;
    }

    const auto slot = static_cast<std::uint8_t>(std::strtoul(argv[2], nullptr, 10));
    const std::uint8_t hop_types[] = { 1 }; // 1 = backplane
    const std::uint8_t ports[] = { 1 };
    const std::uint8_t slots[] = { slot };
    const char *addresses[] = { nullptr };

    const int client_id = eip_connect_with_route_hops(
        argv[1], hop_types, ports, slots, addresses, 1);
    if (client_id <= 0) {
        std::cerr << "connect failed: " << last_error(client_id) << "\n";
        return 1;
    }

    int count = 0;
    if (eip_read_dint(client_id, "ProductionCount", &count) != 0) {
        std::cerr << "read failed: " << last_error(client_id) << "\n";
        eip_disconnect(client_id);
        return 1;
    }
    std::cout << "ProductionCount = " << count << "\n";

    char *diagnostics = nullptr;
    if (eip_get_diagnostics_json(client_id, 1, &diagnostics) == 0 && diagnostics != nullptr) {
        std::cout << diagnostics << "\n";
        eip_free_string(diagnostics);
    } else {
        std::cerr << "diagnostics failed: " << last_error(client_id) << "\n";
    }

    return eip_disconnect(client_id) == 0 ? 0 : 1;
}
