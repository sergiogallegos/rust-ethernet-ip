#include "rust_ethernet_ip.h"

#include <array>
#include <iostream>
#include <stdexcept>
#include <string>

namespace {

std::string last_error(int client_id)
{
    std::array<char, 1024> buffer {};
    const int written = eip_get_last_error(
        client_id, buffer.data(), static_cast<int>(buffer.size()));
    return written > 0
        ? std::string(buffer.data(), static_cast<std::size_t>(written))
        : std::string {};
}

void require_ok(int client_id, int rc, const char *operation)
{
    if (rc != 0)
        throw std::runtime_error(std::string(operation) + ": " + last_error(client_id));
}

} // namespace

int main(int argc, char **argv)
{
    if (argc < 2 || argc > 3) {
        std::cerr << "usage: cpp_udt_and_scope HOST:PORT [PROGRAM_NAME]\n";
        return 2;
    }

    const std::string program = argc == 3 ? argv[2] : "MainProgram";
    const int client_id = eip_connect(argv[1]);
    if (client_id <= 0) {
        std::cerr << "connect failed\n";
        return 1;
    }

    try {
        int controller_count = 0;
        int program_count = 0;
        const std::string program_tag = "Program:" + program + ".ProductionCount";
        require_ok(client_id, eip_read_dint(client_id, "ProductionCount", &controller_count), "read controller tag");
        require_ok(client_id, eip_read_dint(client_id, program_tag.c_str(), &program_count), "read program tag");

        // Read a whole UDT for one logical snapshot.
        std::array<char, 8192> udt_json {};
        require_ok(client_id, eip_read_udt_chunked(client_id, "Mixer", udt_json.data(), static_cast<int>(udt_json.size())), "read whole UDT");
        std::cout << "Mixer snapshot: " << udt_json.data() << "\n";

        // Prefer complete member paths for ordinary application writes.
        double speed = 0.0;
        require_ok(client_id, eip_read_real(client_id, "Mixer.SpeedFeedback", &speed), "read UDT member");
        require_ok(client_id, eip_write_real(client_id, "Mixer.SpeedSetpoint", 60.0), "write UDT member");
        require_ok(client_id, eip_write_bool(client_id, "Mixer.Enabled", 1), "write BOOL member");
        require_ok(client_id, eip_write_string(client_id, "Mixer.Description", "Primary mixer"), "write STRING member");

        // Whole array-element reads work. Write its members individually;
        // writing Motors[0] as one structure is unsupported in 1.2.0.
        require_ok(client_id, eip_read_udt_chunked(client_id, "Motors[0]", udt_json.data(), static_cast<int>(udt_json.size())), "read UDT array element");
        require_ok(client_id, eip_write_dint(client_id, "Motors[0].CommandSpeed", 1250), "write UDT array member");

        std::cout << "controller=" << controller_count
                  << " program=" << program_count
                  << " speed=" << speed << "\n";
    } catch (const std::exception &error) {
        std::cerr << error.what() << "\n";
        eip_disconnect(client_id);
        return 1;
    }

    eip_disconnect(client_id);
    return 0;
}
